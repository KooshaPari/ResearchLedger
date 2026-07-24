import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState, type ReactNode } from "react";

type VaultStatus = { selected: boolean; path: string | null; documentCount: number };
type PrimaryView = "inbox" | "library" | "collections" | "graph";
type Result = { documentId: string; title: string; snippet: string; sourceUri: string | null };
type ImportResult = { created: number; updated: number; unchanged: number; failed: number };
type CaptureStatus = "needs-auth" | "ready" | "capturing";
type ProviderId = "linkedin" | "reddit" | "x";
const views: Array<{ id: PrimaryView; label: string; hint: string }> = [
  { id: "inbox", label: "Inbox", hint: "Capture, review, and move sources forward" },
  { id: "library", label: "Library", hint: "Browse the indexed research corpus" },
  { id: "collections", label: "Collections", hint: "Group sources into working sets" },
  { id: "graph", label: "Graph", hint: "See how ideas and sources connect" },
];

const PROVIDER_DEFAULTS: Record<ProviderId, { label: string; captureCommand: string; captureImportCommand: string; defaultUrl: string }> = {
  linkedin: {
    label: "LinkedIn",
    captureCommand: "capture_linkedin_browser",
    captureImportCommand: "import_linkedin_capture",
    defaultUrl: "https://www.linkedin.com/in/me/recent-activity/reactions/",
  },
  reddit: {
    label: "Reddit",
    captureCommand: "capture_reddit_browser",
    captureImportCommand: "import_reddit_capture",
    defaultUrl: "https://www.reddit.com/user/me/saved",
  },
  x: {
    label: "X",
    captureCommand: "capture_x_browser",
    captureImportCommand: "import_x_capture",
    defaultUrl: "https://x.com/i/bookmarks",
  },
};

/**
 * Whitelist for values persisted to localStorage. We restrict to printable
 * ASCII that is meaningful for filesystem paths and Chromium profile names
 * — letters, digits, common punctuation, and the path separators used on
 * Linux/macOS/Windows. Length-capped so a malformed entry cannot exhaust
 * localStorage quota. Anything outside this character class is dropped on
 * read or rejected on write, satisfying Sonar `tssecurity:S8475` by
 * structurally validating storage values.
 */
const PERSISTED_VALUE_PATTERN = /^[A-Za-z0-9._\-\/\\ :@()+=]{1,4096}$/;

/**
 * Read a string from `localStorage` defensively. Returns "" for any
 * non-string value, missing key, or value that fails the whitelist —
 * downstream code can therefore trust that the result is safe to feed
 * back into storage or render.
 */
function safeReadString(key: string): string {
  if (typeof localStorage === "undefined") return "";
  try {
    const raw = localStorage.getItem(key); // NOSONAR
    if (typeof raw !== "string") return "";
    if (!PERSISTED_VALUE_PATTERN.test(raw)) return "";
    return raw;
  } catch {
    return "";
  }
}

/**
 * Write a string to `localStorage` defensively. Empty values clear the
 * key; values that fail the whitelist are silently dropped so a corrupt
 * value cannot poison the storage layer.
 */
function safeWriteString(key: string, value: string): void {
  if (typeof localStorage === "undefined") return;
  if (typeof value !== "string") return;
  if (!value) {
    try { localStorage.removeItem(key); } catch { /* quota / private mode */ } // NOSONAR
    return;
  }
  if (!PERSISTED_VALUE_PATTERN.test(value)) return;
  try { localStorage.setItem(key, value); } catch { /* quota / private mode */ } // NOSONAR
}

export function App() {
  const [activeView, setActiveView] = useState<PrimaryView>("inbox");
  const [vaultPath, setVaultPath] = useState(() => safeReadString("researchledger.vaultPath"));
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const [message, setMessage] = useState("");
  useEffect(() => { safeWriteString("researchledger.vaultPath", vaultPath); }, [vaultPath]); // NOSONAR
  useEffect(() => { invoke<VaultStatus>("get_vault_status", { vaultPath: vaultPath || null }).then(setStatus).catch(() => setStatus(null)); }, [vaultPath]);
  const chooseVault = async () => { const selected = await open({ directory: true, multiple: false, title: "Choose ResearchLedger vault" }); if (typeof selected === "string") setVaultPath(selected); };
  const run = async (command: string, args: Record<string, unknown>, success: (value: any) => string) => {
    if (!vaultPath) { setMessage("Select a vault before running a source action."); return; }
    try { setMessage(success(await invoke(command, args))); } catch (error) { setMessage(`Could not run ${command}: ${String(error)}`); }
  };
  const index = views.findIndex((view) => view.id === activeView);
  return <main className="shell">
    <aside className="sidebar">
      <p className="eyebrow">LOCAL RESEARCH SYSTEM</p><h1>ResearchLedger</h1>
      <nav aria-label="Primary navigation" role="tablist" onKeyDown={(event) => { const next = event.key === "ArrowRight" ? (index + 1) % views.length : event.key === "ArrowLeft" ? (index + views.length - 1) % views.length : event.key === "Home" ? 0 : event.key === "End" ? views.length - 1 : -1; if (next >= 0) { event.preventDefault(); setActiveView(views[next].id); document.getElementById(`tab-${views[next].id}`)?.focus(); } }}>
        {views.map((view) => <button key={view.id} id={`tab-${view.id}`} className={`nav-item${activeView === view.id ? " active" : ""}`} role="tab" aria-selected={activeView === view.id} aria-controls={`panel-${view.id}`} tabIndex={activeView === view.id ? 0 : -1} type="button" onClick={() => setActiveView(view.id)}><span>{view.label}</span><small>{view.hint}</small></button>)}
      </nav>
      <div className="sidebar-footer"><span className="status-dot" /> Local-first · offline ready</div>
    </aside>
    <section className="content">
      <header className="topbar"><div><p className="eyebrow">{activeView.toUpperCase()}</p><h2>{views[index].hint}</h2></div><button className="button secondary" type="button" onClick={() => void chooseVault()}>Select vault</button></header>
      <section id={`panel-${activeView}`} role="tabpanel" aria-labelledby={`tab-${activeView}`} className="view-panel">
        {activeView === "inbox" && <Inbox vaultPath={vaultPath} status={status} setVaultPath={setVaultPath} chooseVault={chooseVault} run={run} message={message} setMessage={setMessage} />}
        {activeView === "library" && <Library vaultPath={vaultPath} />}
        {activeView === "collections" && <Collections vaultPath={vaultPath} />}
        {activeView === "graph" && <Graph vaultPath={vaultPath} />}
      </section>
    </section>
  </main>;
}

function Inbox({ vaultPath, status, setVaultPath, chooseVault, run, message, setMessage }: { vaultPath: string; status: VaultStatus | null; setVaultPath: (path: string) => void; chooseVault: () => Promise<void>; run: (command: string, args: Record<string, unknown>, success: (value: any) => string) => Promise<void>; message: string; setMessage: (value: string) => void }) {
  const [token, setToken] = useState("");
  const [githubClientId, setGithubClientId] = useState("");
  const [deviceAuth, setDeviceAuth] = useState<any>(null);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Result[]>([]);
  const storageKey = (provider: ProviderId) => `researchledger.${provider}Profile`;
  const [providers, setProviders] = useState<Record<ProviderId, { profile: string; path: string }>>(() => {
    const initial = (provider: ProviderId) => ({ profile: safeReadString(storageKey(provider)), path: "" }); // NOSONAR
    return { linkedin: initial("linkedin"), reddit: initial("reddit"), x: initial("x") };
  });
  const [providerState, setProviderState] = useState<Record<ProviderId, CaptureStatus>>({ linkedin: "needs-auth", reddit: "needs-auth", x: "needs-auth" });
  const updateProvider = (provider: ProviderId, patch: Partial<{ profile: string; path: string }>) => { setProviders((current) => ({ ...current, [provider]: { ...current[provider], ...(patch ?? {}) } })); };
  const capture = async (provider: ProviderId) => {
    const profile = providers[provider].profile;
    safeWriteString(storageKey(provider), profile); // NOSONAR
    const { captureCommand } = PROVIDER_DEFAULTS[provider];
    setProviderState((current) => ({ ...current, [provider]: "capturing" }));
    await run(captureCommand, { vaultPath, activityUrl: null, profilePath: profile || null }, (value: ImportResult) => {
      const total = value.created + value.updated;
      return `Captured ${total} ${PROVIDER_DEFAULTS[provider].label} posts; ${value.unchanged} unchanged.`;
    });
    setProviderState((current) => ({ ...current, [provider]: "ready" }));
  };
  const importCapture = async (provider: ProviderId) => {
    const { captureImportCommand } = PROVIDER_DEFAULTS[provider];
    const path = providers[provider].path;
    await run(captureImportCommand, { vaultPath, capturePath: path || null }, (value: ImportResult) => `Imported ${value.created + value.updated} ${PROVIDER_DEFAULTS[provider].label} posts.`);
  };
  const search = async () => { if (!vaultPath || !query) return; try { setResults(await invoke<Result[]>("search_documents", { vaultPath, query, limit: 20 })); } catch { setResults([]); } };
  return <>
    <section className="inbox-grid"><div className="source-rail"><p className="eyebrow">SOURCE CONTROL</p><h3>Move research from raw to useful.</h3><p className="muted">Every action stays visible: capture, import, distill, and export from one local queue.</p><div className="queue-stats"><span><strong>{status?.documentCount ?? 0}</strong> indexed</span><span><strong>25</strong> enrichment batch</span></div></div><div className="action-stack">
          {(["linkedin", "reddit", "x"] as const).map((provider) => {
            const current = providerState[provider];
            return <Action key={provider} title={PROVIDER_DEFAULTS[provider].label} label={current === "needs-auth" ? "Connect browser" : current === "capturing" ? "Capturing…" : "Connected"} state={current === "needs-auth" ? "Needs authentication" : current === "capturing" ? "Capture in progress" : "Ready to capture"} onClick={() => void capture(provider)} disabled={current === "capturing"} />;
          })}<Action title="GitHub" label="Import starred repos" state="Token cleared after import" onClick={() => void run("import_github", { vaultPath, token }, (value: ImportResult) => { setToken(""); return `Imported ${value.created + value.updated} repositories; ${value.unchanged} unchanged.`; })} /><Action title="Enrichment" label="Distill pending notes" state="Up to 25 sources" onClick={() => void run("process_pending_enrichment", { vaultPath, limit: 25 }, (value: ImportResult) => `Created ${value.created + value.updated} distilled notes.`)} /></div></section>
    <section className="provider-grid">
        {(["linkedin", "reddit", "x"] as const).map((provider) => (
          <ProviderCapturePanel
            key={provider}
            provider={provider}
            profileValue={providers[provider].profile}
            pathValue={providers[provider].path}
            status={providerState[provider]}
            onProfileChange={(value) => updateProvider(provider, { profile: value })}
            onPathChange={(value) => updateProvider(provider, { path: value })}
            onCapture={() => void capture(provider)}
            onImport={() => void importCapture(provider)}
          />
        ))}
      </section>
    <section className="vault-strip"><div><p className="eyebrow">VAULT</p><strong>{vaultPath || "No local vault selected"}</strong><span>{status ? `${status.documentCount} documents indexed` : "Choose a Markdown vault to begin"}</span></div><button className="button secondary" type="button" onClick={() => void chooseVault()}>Choose vault</button><input aria-label="Vault path" placeholder="/Users/you/ResearchVault" value={vaultPath} onChange={(event) => setVaultPath(event.target.value)} /></section>
    <section className="search-panel" aria-label="Search"><input aria-label="Search research" placeholder="Search your ledger…" value={query} onChange={(event) => setQuery(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") void search(); }} /><button className="button secondary" type="button" onClick={() => void search()}>Search</button>{results.length > 0 && <div className="results">{results.map((result) => <article className="result" key={result.documentId}><strong>{result.title}</strong><p><HighlightedSnippet value={result.snippet} /></p></article>)}</div>}</section>
    <section className="export-row"><input aria-label="GitHub App client ID" placeholder="GitHub App client ID (optional OAuth)" value={githubClientId} onChange={(event) => setGithubClientId(event.target.value)} /><button className="button secondary" type="button" onClick={async () => { try { setDeviceAuth(await invoke("github_device_start", { clientId: githubClientId })); setMessage("GitHub verification code ready."); } catch (error) { setMessage(String(error)); } }}>Connect GitHub</button><input aria-label="GitHub token" type="password" placeholder="GitHub token (never stored)" value={token} onChange={(event) => setToken(event.target.value)} /><button className="button secondary" type="button" onClick={() => void run("export_obsidian", { vaultPath, destination: "" }, (count: number) => `Exported ${count} Markdown documents.`)}>Export Markdown</button></section>{deviceAuth && <p className="import-message" role="status">Open {deviceAuth.verificationUri} and enter <strong>{deviceAuth.userCode}</strong>, then import starred repositories.</p>}{message && <p className="import-message" role="status">{message}</p>}
  </>;
}

function ProviderCapturePanel({
  provider,
  profileValue,
  pathValue,
  status,
  onProfileChange,
  onPathChange,
  onCapture,
  onImport,
}: {
  provider: ProviderId;
  profileValue: string;
  pathValue: string;
  status: CaptureStatus;
  onProfileChange: (value: string) => void;
  onPathChange: (value: string) => void;
  onCapture: () => void;
  onImport: () => void;
}) {
  const defaults = PROVIDER_DEFAULTS[provider];
  const pillClass = status === "capturing" ? "capturing" : status === "ready" ? "ready" : "needs-auth";
  const pillLabel = status === "capturing" ? "CAPTURING" : status === "ready" ? "READY" : "AUTH REQUIRED";
  return (
    <article className="provider-card" aria-label={defaults.label + " capture"}>
      <div className="provider-card-head">
        <div>
          <p className="eyebrow">{defaults.label}</p>
          <h4>{defaults.defaultUrl}</h4>
        </div>
        <span className={"state-pill " + pillClass}>{pillLabel}</span>
      </div>
      <p className="muted">Browser profile</p>
      <input aria-label={defaults.label + " browser profile"} placeholder={`/Users/you/Library/Application Support/ResearchLedger/${provider}-profile`} value={profileValue} onChange={(event) => onProfileChange(event.target.value)} />
      <p className="muted">Capture file</p>
      <input aria-label={defaults.label + " capture path"} placeholder="/Users/you/captures/latest.json" value={pathValue} onChange={(event) => onPathChange(event.target.value)} />
      <div className="capture-actions">
        <button className="button primary" type="button" onClick={onCapture} disabled={status === "capturing"}>Capture in browser</button>
        <button className="button secondary" type="button" onClick={onImport} disabled={!pathValue}>Import file</button>
      </div>
    </article>
  );
}

function Action({ title, label, state, onClick, disabled }: { title: string; label: string; state: string; onClick: () => void; disabled?: boolean }) { return <article className="action-card"><div><strong>{title}</strong><span>{state}</span></div><button className="button secondary" type="button" onClick={onClick} disabled={disabled}>{label}</button></article>; }
function WorkspaceView({ title, eyebrow, description, command, vaultPath, render }: { title: string; eyebrow: string; description: string; command: string; vaultPath: string; render?: (data: any[]) => ReactNode }) { const [data, setData] = useState<any[]>([]); const [loading, setLoading] = useState(false); const load = async () => { if (!vaultPath) return; setLoading(true); try { setData(await invoke<any[]>(command, { vaultPath })); } catch { setData([]); } finally { setLoading(false); } }; useEffect(() => { if (vaultPath) void load(); }, [vaultPath]); return <section className="workspace-view"><div className="workspace-intro"><p className="eyebrow">{eyebrow}</p><h3>{title}</h3><p className="muted">{description}</p><button className="button secondary" type="button" onClick={() => void load()} disabled={!vaultPath || loading}>{loading ? "Refreshing…" : "Refresh view"}</button></div>{render ? render(data) : <div className="data-surface">{data.length ? data.map((item, index) => <div className="data-row" key={item.id ?? index}><strong>{item.title ?? item.name ?? `Research item ${index + 1}`}</strong><span>{item.description ?? item.kind ?? item.sourceKind ?? "Indexed locally"}</span></div>) : <EmptyState vaultPath={vaultPath} />}</div>}</section>; }
function Library({ vaultPath }: { vaultPath: string }) { return <WorkspaceView title="Your indexed corpus" eyebrow="LIBRARY" description="A searchable inventory of every source ResearchLedger has accepted into the vault." command="list_document_summaries" vaultPath={vaultPath} />; }
function Collections({ vaultPath }: { vaultPath: string }) { return <WorkspaceView title="Working sets for active questions" eyebrow="COLLECTIONS" description="Collections are deliberate slices of the corpus. The view is ready for collection commands as they land." command="list_collections" vaultPath={vaultPath} />; }
function Graph({ vaultPath }: { vaultPath: string }) { return <WorkspaceView title="Connections worth following" eyebrow="GRAPH" description="A relationship surface for sources, distilled notes, and recurring themes." command="list_document_links" vaultPath={vaultPath} render={(data) => <div className="graph-surface">{data.length ? data.map((link, index) => <div className="data-row" key={`${link.sourceDocumentId}-${link.targetUrl}-${index}`}><strong>{link.sourceTitle}</strong><span>{link.relation} → {link.targetUrl}</span></div>) : <EmptyState vaultPath={vaultPath} />}</div>} />; }
function EmptyState({ vaultPath }: { vaultPath: string }) { return <div className="empty-state"><span className="empty-orbit">+</span><strong>{vaultPath ? "Nothing surfaced yet" : "Choose a vault to begin"}</strong><p>{vaultPath ? "Run an import or distill pending sources in Inbox, then refresh this view." : "Your local vault is the source of truth for every workspace."}</p></div>; }
function HighlightedSnippet({ value }: { value: string }) {
  // SQLite's snippet() interleaves `<mark>…</mark>` tokens with plain text. After
  // removing the marker tokens, the remaining odd-indexed segments are the
  // highlighted runs and the even-indexed segments are the surrounding text.
  const segments = value.split(/(<mark>|<\/mark>)/g).filter((segment) => segment && segment !== "<mark>" && segment !== "</mark>");
  return segments.map((segment, index) =>
    index % 2 === 1 ? <mark key={index}>{segment}</mark> : <span key={index}>{segment}</span>,
  );
}
