import { useEffect, useMemo, useState } from "react"
import {
  ArrowDown,
  ArrowUp,
  CalendarClock,
  CheckCircle2,
  Copy,
  KeyRound,
  Lock,
  LogOut,
  MoreHorizontal,
  Pencil,
  Plus,
  RefreshCw,
  Save,
  Search,
  Trash2,
  Unlock,
  UserRound,
  Wand2,
  X
} from "lucide-react"

import {
  HostApiError,
  createHostRequest,
  sendHostRequest
} from "./lib/native-client"
import { copyTextToClipboard } from "./lib/clipboard"
import { logoSrcFromDataBase64Import } from "./lib/logo"
import {
  getDetailTitle,
  isDetailMode,
  nextSortState,
  sortEntries,
  type PopupMode,
  type SortKey,
  type SortState
} from "./lib/popup-state"
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
type Notice = { text: string; tone: "info" | "success" | "error" }

function Popup() {
  const [status, setStatus] = useState<Status>("checking")
  const [notice, setNotice] = useState<Notice | null>(null)
  const [masterPassword, setMasterPassword] = useState("")
  const [query, setQuery] = useState("")
  const [sort, setSort] = useState<SortState>({ key: "platform", direction: "asc" })
  const [entries, setEntries] = useState<VaultEntry[]>([])
  const [selected, setSelected] = useState<VaultEntry | null>(null)
  const [menuEntry, setMenuEntry] = useState<VaultEntry | null>(null)
  const [mode, setMode] = useState<PopupMode>("compact")
  const [platform, setPlatform] = useState("")
  const [userId, setUserId] = useState("")
  const [password, setPassword] = useState("")
  const [form, setForm] = useState<DetectedForm | null>(null)

  const canSave = platform.trim() && userId.trim() && password
  const sortedEntries = useMemo(() => sortEntries(entries, sort), [entries, sort])

  useEffect(() => {
    void refreshStatus()
  }, [])

  useEffect(() => {
    if (!notice || notice.tone === "error") {
      return
    }

    const timer = window.setTimeout(() => setNotice(null), 2200)
    return () => window.clearTimeout(timer)
  }, [notice])

  useEffect(() => {
    if (status === "unlocked") {
      void refreshEntries()
      void refreshDetectedForm()
    }
  }, [query, status])

  async function refreshStatus() {
    setNotice(null)
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
        showNotice(error instanceof Error ? error.message : "RVault is unavailable.", "error")
      }
    }
  }

  async function unlockVault() {
    setNotice(null)
    try {
      await sendHostRequest<{ locked: boolean }>(
        createHostRequest("unlock", { masterPassword })
      )
      setMasterPassword("")
      setStatus("unlocked")
      await refreshEntries()
    } catch (error) {
      setStatus("locked")
      showNotice(error instanceof Error ? error.message : "Unlock failed.", "error")
    }
  }

  async function lockVault() {
    setNotice(null)
    try {
      await sendHostRequest<{ locked: boolean }>({ type: "lock" })
    } catch (error) {
      showNotice(error instanceof Error ? error.message : "Lock failed.", "error")
    } finally {
      clearUnlockedState()
      setStatus("locked")
    }
  }

  async function quitExtension() {
    setNotice(null)
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

    showNotice("Saved", "success")
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
    setMenuEntry(null)
    showNotice("Deleted", "success")
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
    showNotice(
      result?.filled ? "Filled current form" : "No login form found on this page",
      result?.filled ? "success" : "info"
    )
  }

  async function copyPassword(entry: VaultEntry) {
    try {
      const data = await sendHostRequest<{ password: string }>(
        createHostRequest("get", {
          platform: entry.platform,
          userId: entry.userId
        })
      )
      await copyTextToClipboard(data.password)
      setMenuEntry(null)
      showNotice("Password copied", "success")
    } catch (error) {
      showNotice(error instanceof Error ? error.message : "Copy failed.", "error")
    }
  }

  async function copyUsername(entry: VaultEntry) {
    try {
      await copyTextToClipboard(entry.userId)
      setMenuEntry(null)
      showNotice("Username copied", "success")
    } catch (error) {
      showNotice(error instanceof Error ? error.message : "Copy failed.", "error")
    }
  }

  async function refreshDetectedForm() {
    const detected = await sendActiveTabMessage<DetectedForm>({
      kind: "rvault:get-detected-form"
    })
    setForm(detected)
  }

  function editEntry(entry: VaultEntry) {
    setSelected(entry)
    setMenuEntry(null)
    setMode("edit")
    setPlatform(entry.platform)
    setUserId(entry.userId)
    setPassword("")
  }

  function createEntryFromForm() {
    setSelected(null)
    setMenuEntry(null)
    setMode("create")
    setPlatform("")
    setUserId(form?.username ?? "")
    setPassword(form?.password ?? "")
  }

  function createBlankEntry() {
    setSelected(null)
    setMenuEntry(null)
    setMode("create")
    setPlatform("")
    setUserId("")
    setPassword("")
  }

  function clearEditor() {
    setSelected(null)
    setMenuEntry(null)
    setMode("compact")
    setPlatform("")
    setUserId("")
    setPassword("")
  }

  function clearUnlockedState() {
    setMasterPassword("")
    setSort({ key: "platform", direction: "asc" })
    setEntries([])
    setSelected(null)
    setMenuEntry(null)
    setMode("compact")
    setPlatform("")
    setUserId("")
    setPassword("")
    setForm(null)
  }

  function showNotice(text: string, tone: Notice["tone"] = "info") {
    setNotice({ text, tone })
  }

  function toggleEntryMenu(entry: VaultEntry) {
    setMenuEntry((current) =>
      current && entryKey(current) === entryKey(entry) ? null : entry
    )
  }

  function updateSort(key: SortKey) {
    setSort((current) => nextSortState(current, key))
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
            {status !== "unlocked" ? <p>Local vault access</p> : null}
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

      {notice ? (
        <div className={`notice ${notice.tone}`}>
          {notice.tone === "success" ? <CheckCircle2 size={14} /> : null}
          <span>{notice.text}</span>
        </div>
      ) : null}

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
          <section className="list-controls">
            <div className="search-row">
              <Search size={16} />
              <input
                placeholder="Search entries"
                value={query}
                onChange={(event) => setQuery(event.currentTarget.value)}
              />
              <button className="icon-button" title="Refresh entries" onClick={refreshEntries}>
                <RefreshCw size={15} />
              </button>
            </div>
            <div className="sort-buttons" aria-label="Sort entries">
              <button
                className={sort.key === "platform" ? "active" : ""}
                title="Sort by platform"
                onClick={() => updateSort("platform")}
              >
                <KeyRound size={13} />
                Platform
                <SortArrow active={sort.key === "platform"} direction={sort.direction} />
              </button>
              <button
                className={sort.key === "username" ? "active" : ""}
                title="Sort by username"
                onClick={() => updateSort("username")}
              >
                <UserRound size={13} />
                Username
                <SortArrow active={sort.key === "username"} direction={sort.direction} />
              </button>
              <button
                className={sort.key === "updatedAt" ? "active" : ""}
                title="Sort by date last updated"
                onClick={() => updateSort("updatedAt")}
              >
                <CalendarClock size={13} />
                Updated
                <SortArrow active={sort.key === "updatedAt"} direction={sort.direction} />
              </button>
            </div>
          </section>

          {form ? (
            <section className="form-strip">
              <div>
                <strong>Current form</strong>
                <span>{form.username || form.platformSuggestion || "Login form detected"}</span>
              </div>
              <button className="secondary small" onClick={createEntryFromForm}>
                <Plus size={13} />
                Save
              </button>
            </section>
          ) : null}

          <section className={`entry-list ${isDetailMode(mode) ? "compact" : ""}`}>
            {entries.length === 0 ? (
              <div className="empty">No matching entries.</div>
            ) : (
              sortedEntries.map((entry) => (
                <article key={`${entry.platform}:${entry.userId}`} className="entry">
                  <button className="entry-main" onClick={() => void fillEntry(entry)}>
                    <KeyRound size={15} />
                    <span>
                      <strong>{entry.platform}</strong>
                      <small>{entry.userId}</small>
                    </span>
                  </button>
                  <div className="entry-actions">
                    <button className="row-icon" title="Copy password" onClick={() => void copyPassword(entry)}>
                      <Copy size={13} />
                    </button>
                    <button className="row-icon" title="More actions" onClick={() => toggleEntryMenu(entry)}>
                      <MoreHorizontal size={14} />
                    </button>
                  </div>
                </article>
              ))
            )}
          </section>

          {menuEntry ? (
            <section className="entry-action-panel">
              <div>
                <strong>{menuEntry.platform}</strong>
                <span>{menuEntry.userId}</span>
              </div>
              <div className="entry-action-buttons">
                <button onClick={() => void copyUsername(menuEntry)}>
                  <UserRound size={13} />
                  Username
                </button>
                <button onClick={() => editEntry(menuEntry)}>
                  <Pencil size={13} />
                  Edit
                </button>
                <button className="danger" onClick={() => void deleteEntry(menuEntry)}>
                  <Trash2 size={13} />
                  Delete
                </button>
              </div>
            </section>
          ) : null}

          {isDetailMode(mode) ? (
            <section className="detail-panel">
              <div className="detail-title">
                <div>
                  <span>Selected</span>
                  <strong>{getDetailTitle(mode)}</strong>
                </div>
                <button className="row-icon" title="Close details" onClick={clearEditor}>
                  <X size={14} />
                </button>
              </div>
              {mode === "create" && form?.platformSuggestion ? (
                <div className="form-hint">
                  Suggested platform: {form.platformSuggestion}
                </div>
              ) : null}
              {selected ? (
                <div className="quick-actions">
                  <button className="secondary" onClick={() => void fillEntry(selected)}>
                    <KeyRound size={14} />
                    Fill
                  </button>
                  <button className="secondary" onClick={() => void copyPassword(selected)}>
                    <Copy size={14} />
                    Copy password
                  </button>
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
                    <Wand2 size={14} />
                  </button>
                </div>
              </label>
              <div className="actions">
                <button className="secondary" onClick={clearEditor}>Cancel</button>
                <button className="primary" disabled={!canSave} onClick={() => void saveEntry()}>
                  <Save size={15} />
                  Save
                </button>
              </div>
            </section>
          ) : (
            <section className="compact-actions">
              <button className="secondary" onClick={createBlankEntry}>
                <Plus size={14} />
                New entry
              </button>
            </section>
          )}
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

function entryKey(entry: VaultEntry): string {
  return `${entry.platform}:${entry.userId}`
}

function SortArrow({
  active,
  direction
}: {
  active: boolean
  direction: SortState["direction"]
}) {
  if (!active) {
    return <span className="sort-arrow-placeholder" />
  }

  return direction === "asc" ? <ArrowUp size={12} /> : <ArrowDown size={12} />
}

const styles = `
  :root {
    color-scheme: dark;
    font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    --rv-bg: #060a10;
    --rv-surface: #0c141e;
    --rv-surface-2: #121d2a;
    --rv-hover: #162536;
    --rv-border: rgba(124, 159, 190, .18);
    --rv-text: #e7f3f7;
    --rv-muted: #8ca4b9;
    --rv-muted-2: #60798e;
    --rv-accent: #58d6ff;
    --rv-danger: #ff6f75;
    --rv-success: #73e6a1;
  }
  body {
    margin: 0;
    background: var(--rv-bg);
    color: var(--rv-text);
  }
  .rvault-popup {
    width: 370px;
    min-height: 410px;
    padding: 13px;
    box-sizing: border-box;
    background: var(--rv-bg);
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    margin-bottom: 13px;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .brand-lockup {
    display: flex;
    align-items: center;
    gap: 11px;
    min-width: 0;
  }
  .brand-logo {
    width: 36px;
    height: 36px;
    border-radius: 8px;
    box-shadow: 0 0 0 1px rgba(88, 214, 255, .24);
  }
  h1 {
    margin: 0;
    font-size: 19px;
    line-height: 1.25;
    font-weight: 800;
  }
  p {
    margin: 2px 0 0;
    color: var(--rv-muted);
    font-size: 12px;
  }
  section {
    margin-bottom: 10px;
  }
  label {
    display: grid;
    gap: 6px;
    margin-bottom: 9px;
    font-size: 11px;
    font-weight: 700;
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
    color: var(--rv-muted-2);
  }
  input:focus {
    outline: 2px solid rgba(88, 214, 255, .2);
    border-color: var(--rv-accent);
  }
  button {
    height: 30px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--rv-text);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    font-size: 13px;
    cursor: pointer;
  }
  button:hover {
    background: var(--rv-hover);
  }
  button:disabled {
    opacity: .45;
    cursor: not-allowed;
  }
  .primary {
    background: var(--rv-accent);
    color: #03101a;
    padding: 0 12px;
    font-weight: 800;
  }
  .primary:hover {
    background: #79e0ff;
  }
  .secondary {
    border: 1px solid var(--rv-border);
    color: var(--rv-text);
    padding: 0 10px;
  }
  .secondary.small {
    height: 26px;
    font-size: 12px;
    padding: 0 8px;
  }
  .icon-button {
    width: 30px;
    min-width: 30px;
    padding: 0;
    border: 1px solid var(--rv-border);
    background: rgba(18, 29, 42, .8);
  }
  .danger {
    color: var(--rv-danger);
  }
  .notice {
    display: flex;
    align-items: center;
    gap: 7px;
    min-height: 28px;
    box-sizing: border-box;
    border-radius: 7px;
    background: rgba(16, 25, 35, .88);
    border: 1px solid rgba(88, 214, 255, .16);
    color: #d8e8f0;
    padding: 6px 9px;
    font-size: 12px;
    margin-bottom: 9px;
  }
  .notice.success {
    border-color: rgba(115, 230, 161, .26);
    color: #e3ffed;
  }
  .notice.error {
    border-color: rgba(255, 111, 117, .35);
    color: #ffd9dc;
  }
  .empty, .form-hint {
    color: var(--rv-muted);
    font-size: 12px;
    padding: 8px 0;
  }
  .unlock-panel {
    display: grid;
    gap: 10px;
  }
  .list-controls {
    display: grid;
    gap: 7px;
  }
  .search-row {
    display: grid;
    grid-template-columns: 16px 1fr 28px;
    align-items: center;
    gap: 7px;
    min-height: 34px;
    background: rgba(12, 20, 30, .72);
    border-radius: 8px;
    padding: 3px 5px 3px 9px;
    color: var(--rv-accent);
  }
  .search-row input {
    border: 0;
    height: 28px;
    padding: 0;
    background: transparent;
  }
  .search-row .icon-button {
    width: 28px;
    min-width: 28px;
    height: 28px;
    border: 0;
    background: transparent;
    color: var(--rv-muted);
  }
  .sort-buttons {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 6px;
  }
  .sort-buttons button {
    height: 28px;
    min-width: 0;
    padding: 0 7px;
    border: 1px solid var(--rv-border);
    color: var(--rv-muted);
    font-size: 11px;
    gap: 4px;
  }
  .sort-buttons button.active {
    border-color: rgba(88, 214, 255, .42);
    color: var(--rv-text);
    background: rgba(88, 214, 255, .08);
  }
  .sort-arrow-placeholder {
    width: 12px;
    height: 12px;
  }
  .form-strip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 7px 0 8px;
    border-bottom: 1px solid var(--rv-border);
  }
  .form-strip div {
    display: grid;
    gap: 2px;
    min-width: 0;
  }
  .form-strip strong {
    font-size: 12px;
  }
  .form-strip span {
    color: var(--rv-muted);
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .entry-list {
    display: grid;
    max-height: 238px;
    overflow: auto;
  }
  .entry-list.compact {
    max-height: 126px;
  }
  .entry {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
    align-items: center;
    border-bottom: 1px solid var(--rv-border);
  }
  .entry-main {
    min-width: 0;
    justify-content: flex-start;
    height: 49px;
    padding: 0 2px;
    color: var(--rv-text);
  }
  .entry-main:hover {
    background: transparent;
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
  .entry-actions {
    display: flex;
    align-items: center;
    gap: 3px;
    opacity: .78;
  }
  .entry:hover .entry-actions {
    opacity: 1;
  }
  .row-icon {
    width: 26px;
    min-width: 26px;
    height: 26px;
    padding: 0;
    color: var(--rv-muted);
  }
  .row-icon:hover {
    color: var(--rv-text);
    background: var(--rv-hover);
  }
  .row-icon.danger:hover {
    color: var(--rv-danger);
  }
  .entry-action-panel {
    display: grid;
    gap: 7px;
    padding: 8px 0 10px;
    border-bottom: 1px solid var(--rv-border);
  }
  .entry-action-panel > div:first-child {
    display: grid;
    min-width: 0;
  }
  .entry-action-panel strong, .entry-action-panel span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .entry-action-panel span {
    color: var(--rv-muted);
    font-size: 12px;
  }
  .entry-action-buttons {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }
  .entry-action-buttons button {
    border: 1px solid var(--rv-border);
    font-size: 12px;
  }
  .detail-panel {
    border-top: 1px solid var(--rv-border);
    padding-top: 11px;
    margin-top: 4px;
  }
  .detail-title, .actions, .password-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .detail-title {
    justify-content: space-between;
    margin-bottom: 9px;
  }
  .detail-title div {
    display: grid;
    gap: 1px;
  }
  .detail-title span {
    color: var(--rv-muted-2);
    font-size: 10px;
    font-weight: 800;
    text-transform: uppercase;
  }
  .detail-title strong {
    font-size: 14px;
  }
  .quick-actions {
    display: flex;
    gap: 7px;
    margin-bottom: 10px;
  }
  .password-row input {
    flex: 1;
  }
  .actions {
    justify-content: flex-end;
  }
  .compact-actions {
    display: flex;
    justify-content: flex-end;
    padding-top: 2px;
  }
  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  }
`

export default Popup
