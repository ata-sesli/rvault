// keystore_bin.rs
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng, Payload},
    ChaCha20Poly1305, Key, Nonce,
};
use directories::ProjectDirs;
use rand::RngCore;
use zeroize::Zeroize;

const MAGIC: &[u8; 8] = b"RVAULT\0\x01"; // magic + minor, tweak as you like
const AAD: &[u8] = b"rvault-keystore-v1";
const FILE_NAME: &str = "keystore.rvault";
const EK_LEN: usize = 32;

#[derive(Clone, Copy)]
pub struct KdfParams {
    pub t: u32, // time
    pub m: u32, // memory KiB
    pub p: u32, // lanes
}

fn cfg_dir() -> Result<PathBuf, String> {
    let pd = ProjectDirs::from("io.github", "ata-sesli", "RVault")
        .ok_or("Could not find project directories".to_string())?;
    fs::create_dir_all(pd.config_dir()).map_err(|e| format!("mkdir: {e}"))?;
    Ok(pd.config_dir().to_path_buf())
}
fn file_path() -> Result<PathBuf, String> { Ok(cfg_dir()?.join(FILE_NAME)) }

fn derive_kek(mp: &[u8], salt: &[u8], k: KdfParams) -> Result<Key<ChaCha20Poly1305>, String> {
    let params = Params::new(k.m, k.t, k.p, Some(EK_LEN))
        .map_err(|e| format!("Argon2 params: {e}"))?;
    let a2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; EK_LEN];
    a2.hash_password_into(mp, salt, &mut out)
        .map_err(|e| format!("Argon2 derive: {e}"))?;
    let key = Key::from_slice(&out).to_owned();
    out.zeroize();
    Ok(key)
}

fn crc32(bytes: &[u8]) -> u32 {
    use crc32fast::Hasher;
    let mut h = Hasher::new();
    h.update(bytes);
    h.finalize()
}

fn write_u32_le(v: u32, buf: &mut Vec<u8>) {
    buf.extend_from_slice(&v.to_le_bytes());
}
fn read_u32_le(src: &[u8], cur: &mut usize) -> Result<u32, String> {
    if *cur + 4 > src.len() { return Err("truncated u32".into()); }
    let mut arr = [0u8; 4];
    arr.copy_from_slice(&src[*cur..*cur+4]);
    *cur += 4;
    Ok(u32::from_le_bytes(arr))
}

/// Create keystore and return freshly generated EK (32 bytes).
/// `banner`: any custom text you want to embed before the binary payload.
/// `kdf`: None → defaults (t=3, m≈205MiB, p=1)
pub fn create(master_password: &str, banner: &str, kdf: Option<KdfParams>) -> Result<[u8; EK_LEN], String> {
    let path = file_path()?;
    if path.exists() { return Err("keystore already exists".into()); }

    let kdf = kdf.unwrap_or(KdfParams { t: 3, m: 209_715, p: 1 });

    let mut ek = [0u8; EK_LEN];
    OsRng.fill_bytes(&mut ek);

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let kek  = derive_kek(master_password.as_bytes(), &salt, kdf)?;
    let aead = ChaCha20Poly1305::new(&kek);
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    let ct = aead.encrypt(&nonce, Payload { msg: &ek, aad: AAD })
        .map_err(|e| format!("AEAD encrypt: {e}"))?;

    // Build binary payload
    let mut payload = Vec::with_capacity(64 + ct.len());
    payload.extend_from_slice(MAGIC);                  // magic
    write_u32_le(1, &mut payload);                     // version
    write_u32_le(0, &mut payload);                     // flags (reserved)
    write_u32_le(kdf.t, &mut payload);
    write_u32_le(kdf.m, &mut payload);
    write_u32_le(kdf.p, &mut payload);
    payload.extend_from_slice(&salt);
    payload.extend_from_slice(&nonce);
    write_u32_le(ct.len() as u32, &mut payload);
    payload.extend_from_slice(&ct);
    let csum = crc32(&payload);
    write_u32_le(csum, &mut payload);

    // Compose file: banner (UTF-8) + random noise (optional) + payload
    let mut file_bytes = Vec::with_capacity(banner.len() + payload.len() + 16);
    if !banner.is_empty() {
        file_bytes.extend_from_slice(banner.as_bytes());
        if !banner.ends_with('\n') { file_bytes.push(b'\n'); }
    }
    // Add a little noise to make it less predictable (optional)
    let mut noise = [0u8; 8];
    OsRng.fill_bytes(&mut noise);
    file_bytes.extend_from_slice(&noise);

    file_bytes.extend_from_slice(&payload);

    // Atomic-ish write with 0600
    let tmp = path.with_extension("tmp");
    {
        let mut f = OpenOptions::new().create_new(true).write(true)
            .open(&tmp).map_err(|e| format!("open tmp: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perm = f.metadata().map_err(|e| format!("meta: {e}"))?.permissions();
            perm.set_mode(0o600);
            fs::set_permissions(&tmp, perm).map_err(|e| format!("chmod: {e}"))?;
        }
        f.write_all(&file_bytes).map_err(|e| format!("write: {e}"))?;
        f.sync_all().ok();
    }
    fs::rename(&tmp, &path).map_err(|e| format!("rename: {e}"))?;
    Ok(ek)
}

/// Load EK from keystore using master password.
pub fn load(master_password: &str) -> Result<[u8; EK_LEN], String> {
    let path = file_path()?;
    let mut buf = Vec::new();
    OpenOptions::new().read(true).open(&path)
        .map_err(|e| format!("open keystore: {e}"))?
        .read_to_end(&mut buf)
        .map_err(|e| format!("read keystore: {e}"))?;

    // Find MAGIC anywhere in the file (ignore banner/noise before it)
    let Some(start) = memchr::memmem::find(&buf, MAGIC) else {
        return Err("magic not found (corrupt file)".into());
    };
    let data = &buf[start..];

    let mut cur = 0usize;
    let take = |n: usize, cur: &mut usize, data: &[u8]| -> Result<&[u8], String> {
        if *cur + n > data.len() { return Err("truncated".into()); }
        let out = &data[*cur..*cur+n];
        *cur += n;
        Ok(out)
    };

    // MAGIC
    let _ = take(MAGIC.len(), &mut cur, data)?;
    // VERSION
    let version = read_u32_le(data, &mut cur)?;
    if version != 1 { return Err("unsupported version".into()); }
    // FLAGS
    let _flags = read_u32_le(data, &mut cur)?;
    // KDF t,m,p
    let t = read_u32_le(data, &mut cur)?;
    let m = read_u32_le(data, &mut cur)?;
    let p = read_u32_le(data, &mut cur)?;
    let kdf = KdfParams { t, m, p };
    // SALT, NONCE
    let salt = take(16, &mut cur, data)?;
    let nonce_bytes = take(12, &mut cur, data)?;
    let nonce = Nonce::from_slice(nonce_bytes);
    // CT
    let ct_len = read_u32_le(data, &mut cur)? as usize;
    let ct = take(ct_len, &mut cur, data)?;
    // CRC32 (over everything from MAGIC..end_of_ct)
    let saved_crc = read_u32_le(data, &mut cur)?;
    let crc_now = crc32(&data[..cur-4]); // exclude the stored crc field
    if saved_crc != crc_now { return Err("CRC mismatch (corrupt file)".into()); }

    // Derive KEK & decrypt EK
    let kek = derive_kek(master_password.as_bytes(), salt, kdf)?;
    let aead = ChaCha20Poly1305::new(&kek);

    let mut ek = aead
        .decrypt(nonce, Payload { msg: ct, aad: AAD })
        .map_err(|_| "invalid password or corrupted keystore".to_string())?;

    if ek.len() != EK_LEN { ek.zeroize(); return Err("bad EK len".into()); }
    let mut out = [0u8; EK_LEN];
    out.copy_from_slice(&ek);
    ek.zeroize();
    Ok(out)
}
