import { useEffect, useMemo, useState } from "react"
import {
  KeyRound,
  Lock,
  LogOut,
  Pencil,
  RefreshCw,
  Save,
  Search,
  Trash2,
  Unlock,
  Wand2
} from "lucide-react"

import {
  HostApiError,
  createHostRequest,
  sendHostRequest
} from "./lib/native-client"
import { logoSrcFromDataBase64Import } from "./lib/logo"
import logoDataUrl from "data-base64:./assets/icon.png"

const logoSrc = logoSrcFromDataBase64Import(logoDataUrl)

type VaultEntry = {
  platform: string
  userId: string
  pinned: boolean
  createdAt: number
  updatedAt: number
}

type DetectedForm = {
  platformSuggestion: string
  username: string
  password: string
}

type Status = "checking" | "locked" | "unlocked" | "setup_required" | "error"

function Popup() {
  const [status, setStatus] = useState<Status>("checking")
  const [message, setMessage] = useState("")
  const [masterPassword, setMasterPassword] = useState("")
  const [query, setQuery] = useState("")
  const [entries, setEntries] = useState<VaultEntry[]>([])
  const [selected, setSelected] = useState<VaultEntry | null>(null)
  const [platform, setPlatform] = useState("")
  const [userId, setUserId] = useState("")
  const [password, setPassword] = useState("")
  const [form, setForm] = useState<DetectedForm | null>(null)

  const canSave = platform.trim() && userId.trim() && password

  useEffect(() => {
    void refreshStatus()
  }, [])

  useEffect(() => {
    if (status === "unlocked") {
      void refreshEntries()
      void refreshDetectedForm()
    }
  }, [query, status])

  async function refreshStatus() {
    setMessage("")
    try {
      const data = await sendHostRequest<{ locked: boolean; setupRequired?: boolean }>({
        type: "status"
      })
      if (data.locked) {
        clearUnlockedState()
      }
      setStatus(data.locked ? "locked" : "unlocked")
    } catch (error) {
      clearUnlockedState()
      if (error instanceof HostApiError && error.code === "setup_required") {
        setStatus("setup_required")
      } else {
        setStatus("error")
        setMessage(error instanceof Error ? error.message : "RVault is unavailable.")
      }
    }
  }

  async function unlockVault() {
    setMessage("")
    try {
      await sendHostRequest<{ locked: boolean }>(
        createHostRequest("unlock", { masterPassword })
      )
      setMasterPassword("")
      setStatus("unlocked")
      await refreshEntries()
    } catch (error) {
      setStatus("locked")
      setMessage(error instanceof Error ? error.message : "Unlock failed.")
    }
  }

  async function lockVault() {
    setMessage("")
    try {
      await sendHostRequest<{ locked: boolean }>({ type: "lock" })
    } catch (error) {
      setMessage(error instanceof Error ? error.message : "Lock failed.")
    } finally {
      clearUnlockedState()
      setStatus("locked")
    }
  }

  async function quitExtension() {
    setMessage("")
    try {
      await sendHostRequest<{ locked: boolean }>({ type: "quit" })
    } finally {
      clearUnlockedState()
      setStatus("locked")
      window.close()
    }
  }

  async function refreshEntries() {
    const data = await sendHostRequest<{ entries: VaultEntry[] }>({
      type: "list",
      query: query.trim() || undefined
    })
    setEntries(data.entries)
  }

  async function saveEntry() {
    if (!canSave) {
      return
    }

    if (selected) {
      await sendHostRequest<{ saved: boolean }>(
        createHostRequest("update", {
          platform: selected.platform,
          oldUserId: selected.userId,
          newUserId: userId.trim(),
          password
        })
      )
    } else {
      await sendHostRequest<{ saved: boolean }>(
        createHostRequest("create", {
          platform: platform.trim(),
          userId: userId.trim(),
          password
        })
      )
    }

    setMessage("Saved.")
    clearEditor()
    await refreshEntries()
  }

  async function deleteEntry(entry: VaultEntry) {
    await sendHostRequest<{ deleted: boolean }>(
      createHostRequest("delete", {
        platform: entry.platform,
        userId: entry.userId
      })
    )
    if (selected?.platform === entry.platform && selected.userId === entry.userId) {
      clearEditor()
    }
    await refreshEntries()
  }

  async function generatePassword() {
    const data = await sendHostRequest<{ password: string }>({
      type: "generate",
      length: 20,
      specialCharacters: true
    })
    setPassword(data.password)
  }

  async function fillEntry(entry: VaultEntry) {
    const data = await sendHostRequest<{ password: string }>(
      createHostRequest("get", {
        platform: entry.platform,
        userId: entry.userId
      })
    )
    const result = await sendActiveTabMessage<{ filled: boolean }>({
      kind: "rvault:fill",
      username: entry.userId,
      password: data.password
    })
    setMessage(result?.filled ? "Filled current form." : "No login form found on this page.")
  }

  async function refreshDetectedForm() {
    const detected = await sendActiveTabMessage<DetectedForm>({
      kind: "rvault:get-detected-form"
    })
    setForm(detected)
    if (detected) {
      setPlatform((current) => current || detected.platformSuggestion)
      setUserId((current) => current || detected.username)
      setPassword((current) => current || detected.password)
    }
  }

  function editEntry(entry: VaultEntry) {
    setSelected(entry)
    setPlatform(entry.platform)
    setUserId(entry.userId)
    setPassword("")
  }

  function clearEditor() {
    setSelected(null)
    setPlatform("")
    setUserId("")
    setPassword("")
  }

  function clearUnlockedState() {
    setMasterPassword("")
    setEntries([])
    setSelected(null)
    setPlatform("")
    setUserId("")
    setPassword("")
    setForm(null)
  }

  const title = useMemo(() => {
    if (status === "unlocked") return "RVault"
    if (status === "setup_required") return "Setup Required"
    return "Unlock RVault"
  }, [status])

  return (
    <main className="rvault-popup">
      <style>{styles}</style>
      <header>
        <div className="brand-lockup">
          <img className="brand-logo" src={logoSrc} alt="" />
          <div>
            <h1>{title}</h1>
            <p>{status === "unlocked" ? "Helium native messaging" : "Local vault access"}</p>
          </div>
        </div>
        {status === "unlocked" ? (
          <div className="header-actions">
            <button className="icon-button" title="Quit extension" onClick={() => void quitExtension()}>
              <LogOut size={16} />
            </button>
            <button className="icon-button" title="Lock vault" onClick={() => void lockVault()}>
              <Lock size={16} />
            </button>
          </div>
        ) : (
          <button className="icon-button" title="Refresh" onClick={refreshStatus}>
            <RefreshCw size={16} />
          </button>
        )}
      </header>

      {message ? <div className="notice">{message}</div> : null}

      {status === "setup_required" ? (
        <section className="empty">Run <code>rvault setup</code> before using the extension.</section>
      ) : null}

      {(status === "locked" || status === "error" || status === "checking") ? (
        <section className="unlock-panel">
          <label>
            Master password
            <input
              autoFocus
              type="password"
              value={masterPassword}
              onChange={(event) => setMasterPassword(event.currentTarget.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") void unlockVault()
              }}
            />
          </label>
          <button className="primary" disabled={!masterPassword} onClick={unlockVault}>
            <Unlock size={16} />
            Unlock
          </button>
        </section>
      ) : null}

      {status === "unlocked" ? (
        <>
          <section className="search-row">
            <Search size={16} />
            <input
              placeholder="Search entries"
              value={query}
              onChange={(event) => setQuery(event.currentTarget.value)}
            />
            <button className="icon-button" title="Refresh entries" onClick={refreshEntries}>
              <RefreshCw size={15} />
            </button>
          </section>

          <section className="entry-list">
            {entries.length === 0 ? (
              <div className="empty">No matching entries.</div>
            ) : (
              entries.map((entry) => (
                <article key={`${entry.platform}:${entry.userId}`} className="entry">
                  <button className="entry-main" onClick={() => void fillEntry(entry)}>
                    <KeyRound size={16} />
                    <span>
                      <strong>{entry.platform}</strong>
                      <small>{entry.userId}</small>
                    </span>
                  </button>
                  <button className="icon-button" title="Edit" onClick={() => editEntry(entry)}>
                    <Pencil size={14} />
                  </button>
                  <button className="icon-button danger" title="Delete" onClick={() => void deleteEntry(entry)}>
                    <Trash2 size={14} />
                  </button>
                </article>
              ))
            )}
          </section>

          <section className="editor">
            <div className="editor-title">
              <strong>{selected ? "Edit Entry" : "Save Entry"}</strong>
              <button className="link-button" onClick={() => void refreshDetectedForm()}>
                Current form
              </button>
            </div>
            {form ? (
              <div className="form-hint">
                Detected {form.username ? form.username : "a login form"} on {form.platformSuggestion}
              </div>
            ) : null}
            <label>
              Platform
              <input value={platform} onChange={(event) => setPlatform(event.currentTarget.value)} />
            </label>
            <label>
              Username
              <input value={userId} onChange={(event) => setUserId(event.currentTarget.value)} />
            </label>
            <label>
              Password
              <div className="password-row">
                <input
                  type="password"
                  value={password}
                  onChange={(event) => setPassword(event.currentTarget.value)}
                />
                <button className="icon-button" title="Generate password" onClick={() => void generatePassword()}>
                  <Wand2 size={15} />
                </button>
              </div>
            </label>
            <div className="actions">
              <button className="primary" disabled={!canSave} onClick={() => void saveEntry()}>
                <Save size={16} />
                Save
              </button>
              <button onClick={clearEditor}>Clear</button>
            </div>
          </section>
        </>
      ) : null}
    </main>
  )
}

async function sendActiveTabMessage<T>(message: unknown): Promise<T | null> {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true })
  if (!tab?.id) {
    return null
  }

  return new Promise((resolve) => {
    chrome.tabs.sendMessage(tab.id!, message, (response) => {
      if (chrome.runtime.lastError) {
        resolve(null)
      } else {
        resolve((response ?? null) as T | null)
      }
    })
  })
}

const styles = `
  :root {
    color-scheme: dark;
    font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    --rv-bg: #070b12;
    --rv-surface: #0d1724;
    --rv-surface-2: #122238;
    --rv-border: #243b55;
    --rv-border-strong: #35627d;
    --rv-text: #e7f3f7;
    --rv-muted: #91a8bc;
    --rv-soft: #172b42;
    --rv-accent: #5dd4dc;
    --rv-accent-strong: #36a8d4;
    --rv-blue: #1d5a9b;
    --rv-danger: #ff7777;
  }
  body {
    margin: 0;
    background: var(--rv-bg);
    color: var(--rv-text);
  }
  .rvault-popup {
    width: 370px;
    min-height: 440px;
    padding: 14px;
    box-sizing: border-box;
    background: var(--rv-bg);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .brand-lockup {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .brand-logo {
    width: 42px;
    height: 42px;
    border-radius: 8px;
    box-shadow: 0 0 0 1px rgba(93, 212, 220, .35);
  }
  h1 {
    margin: 0;
    font-size: 18px;
    line-height: 1.25;
  }
  p {
    margin: 2px 0 0;
    color: var(--rv-muted);
    font-size: 12px;
  }
  section {
    margin-bottom: 12px;
  }
  label {
    display: grid;
    gap: 5px;
    margin-bottom: 9px;
    font-size: 12px;
    color: var(--rv-muted);
  }
  input {
    width: 100%;
    height: 34px;
    box-sizing: border-box;
    border: 1px solid var(--rv-border);
    border-radius: 7px;
    padding: 0 9px;
    background: #08111e;
    color: var(--rv-text);
    font-size: 13px;
  }
  input::placeholder {
    color: #668096;
  }
  input:focus {
    outline: 2px solid rgba(93, 212, 220, .25);
    border-color: var(--rv-accent);
  }
  button {
    height: 32px;
    border: 1px solid var(--rv-border);
    border-radius: 7px;
    background: var(--rv-surface-2);
    color: var(--rv-text);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    font-size: 13px;
    cursor: pointer;
  }
  button:disabled {
    opacity: .45;
    cursor: not-allowed;
  }
  .primary {
    background: var(--rv-accent-strong);
    border-color: var(--rv-accent);
    color: #03101a;
    padding: 0 12px;
    font-weight: 700;
  }
  .icon-button {
    width: 32px;
    min-width: 32px;
    padding: 0;
  }
  .danger {
    color: var(--rv-danger);
  }
  .link-button {
    border: 0;
    background: transparent;
    color: var(--rv-accent);
    height: auto;
    padding: 0;
  }
  .notice, .empty, .form-hint {
    border-radius: 7px;
    background: var(--rv-soft);
    border: 1px solid var(--rv-border);
    color: #c9dce7;
    padding: 9px;
    font-size: 12px;
  }
  .unlock-panel {
    display: grid;
    gap: 10px;
  }
  .search-row {
    display: grid;
    grid-template-columns: 18px 1fr 32px;
    align-items: center;
    gap: 8px;
    background: var(--rv-surface);
    border: 1px solid var(--rv-border);
    border-radius: 8px;
    padding: 7px;
    color: var(--rv-accent);
  }
  .search-row input {
    border: 0;
    height: 28px;
    padding: 0;
    background: transparent;
  }
  .entry-list {
    display: grid;
    gap: 7px;
    max-height: 170px;
    overflow: auto;
  }
  .entry {
    display: grid;
    grid-template-columns: 1fr 32px 32px;
    gap: 6px;
    align-items: center;
  }
  .entry-main {
    min-width: 0;
    justify-content: flex-start;
    height: 40px;
    padding: 0 9px;
  }
  .entry-main span {
    display: grid;
    min-width: 0;
    text-align: left;
  }
  .entry-main strong, .entry-main small {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .entry-main small {
    color: var(--rv-muted);
  }
  .editor {
    border-top: 1px solid var(--rv-border);
    padding-top: 12px;
  }
  .editor-title, .actions, .password-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .editor-title {
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .password-row input {
    flex: 1;
  }
  .actions {
    justify-content: flex-end;
  }
  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  }
`

export default Popup
