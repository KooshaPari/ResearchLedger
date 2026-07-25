import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState, type ReactNode } from "react";

type VaultStatus = { selected: boolean; path: string | null; documentCount: number };
type PrimaryView = "inbox" | "library" | "collections" | "graph";
type Result = { documentId: string; title: string; snippet: string; sourceUri: string | null };
type ImportResult = { created: number; updated: number; unchanged: number; failed: number };
type ImportSummary = { provider: string; created: number; updated: number; unchanged: number; failed: number; path: string };

/**
 * Friendly error renderer for Tauri command failures. Strips ANSI escapes,
 * recognises the "Playwright Chromium not installed" failure, and surfaces a
 * short hint instead of leaking `node:internal/modules/run_main:107` to the UI.
 */
function formatCommandError(command: string, error: unknown): string {
  const raw = String(error ?? "");
  const stripped = raw.replace(/\u001b\[[0-9;]*m/g, "").replace(/\s+/g, " ").trim();
  if (stripped.includes("Executable doesn't exist") || stripped.includes("playwright install")) {
    return "Playwright's Chromium browser is not installed. Run `npx playwright install chromium` once, then retry.";
  }
  const friendly: Record<string, string> = {
    capture_linkedin_browser: "LinkedIn capture failed. Confirm your browser profile path and that Playwright's Chromium is installed.",
    capture_reddit_browser: "Reddit capture failed. Replace REPLACE_WITH_YOUR_USERNAME with your Reddit username in the URL, then retry.",
    capture_x_browser: "X capture failed. Confirm your browser profile path and that Playwright's Chromium is installed.",
    import_reddit_html: "Reddit import failed. Make sure the file you selected is a Reddit saved-posts HTML page.",
    import_x_html: "X import failed. Make sure the file you selected is an X bookmarks HTML page.",
  };
  const hint = friendly[command] ?? `${command} failed. See the desktop log for the full trace.`;
  const firstSentence = stripped.split(/[.\n]/, 1)[0].slice(0, 200);
  return firstSentence ? `${hint} (${firstSentence})` : hint;
}

type ProviderId = "linkedin" | "reddit" | "x";
type CaptureStatus = "needs-auth" | "capturing" | "ready";

const views: Array<{ id: PrimaryView; label: string; hint: string }> = [
  { id: "inbox", label: "Inbox", hint: "Capture, review, and move sources forward" },
  { id: "library", label: "Library", hint: "Browse the indexed research corpus" },
  { id: "collections", label: "Collections", hint: "Group sources into working sets" },
  { id: "graph", label: "Graph", hint: "See how ideas and sources connect" },
];

const PROVIDER_DEFAULTS: Record<ProviderId, {
  label: string;
  description: string;
  captureCommand: string;
  captureImportCommand: string;
  defaultUrl: string;
  captureBasename: string;
  profileBasename: string;
}> = {
  linkedin: {
    label: "LinkedIn",
    description: "Capture reactions and saved posts from your LinkedIn activity.",
    captureCommand: "capture_linkedin_browser",
    captureImportCommand: "import_linkedin_capture",
    defaultUrl: "https://www.linkedin.com/in/me/recent-activity/reactions/",
    captureBasename: "linkedin.json",
    profileBasename: "linkedin-profile",
  },
  reddit: {
    label: "Reddit",
    description: "Capture saved posts from your Reddit account (replace the username placeholder).",
    captureCommand: "capture_reddit_browser",
    captureImportCommand: "import_reddit_capture",
    defaultUrl: "https://www.reddit.com/user/REPLACE_WITH_YOUR_USERNAME/saved",
    captureBasename: "reddit.json",
    profileBasename: "reddit-profile",
  },
  x: {
    label: "X",
    description: "Capture bookmarks from your X account.",
    captureCommand: "capture_x_browser",
    captureImportCommand: "import_x_capture",
    defaultUrl: "https://x.com/i/bookmarks",
    captureBasename: "x.json",
    profileBasename: "x-profile",
  },
};

/**
 * Whitelist for values persisted to localStorage. Letters, digits, common
 * punctuation, and the path separators used on Linux/macOS/Windows. Anything
 * outside this character class is dropped on read or rejected on write.
 */
const PERSISTED_VALUE_PATTERN = /^[A-Za-z0-9._\-\/\\ :@()+=]{1,4096}$/;

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

const profileStorageKey = (provider: ProviderId) => `researchledger.${provider}Profile`;

export function App() {
  const [activeView, setActiveView] = useState<PrimaryView>("inbox");
  const [vaultPath, setVaultPath] = useState(() => safeReadString("researchledger.vaultPath"));
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const [message, setMessage] = useState("");
  useEffect(() => { safeWriteString("researchledger.vaultPath", vaultPath); }, [vaultPath]); // NOSONAR
  useEffect(() => { invoke<VaultStatus>("get_vault_status", { vaultPath: vaultPath || null }).then(setStatus).catch(() => setStatus(null)); }, [vaultPath]);

  const chooseVault = async () => {
    const selected = await open({ directory: true, multiple: false, title: "Choose ResearchLedger vault" });
    if (typeof selected === "string") setVaultPath(selected);
  };

  const runCommand = async (command: string, args: Record<string, unknown>, onSuccess: (value: ImportResult | number) => string) => {
    if (!vaultPath) { setMessage("Select a vault before running a source action."); return; }
    try {
      const value = await invoke<ImportResult | number>(command, args);
      setMessage(onSuccess(value));
    } catch (error) {
      setMessage(formatCommandError(command, error));
    }
  };

  const index = views.findIndex((view) => view.id === activeView);
  return (
    <main className="shell">
      <aside className="sidebar">
        <p className="eyebrow">LOCAL RESEARCH SYSTEM</p>
        <h1>ResearchLedger</h1>
        <nav aria-label="Primary navigation" role="tablist" onKeyDown={(event) => {
          const next = event.key === "ArrowRight" ? (index + 1) % views.length
            : event.key === "ArrowLeft" ? (index + views.length - 1) % views.length
            : event.key === "Home" ? 0
            : event.key === "End" ? views.length - 1
            : -1;
          if (next >= 0) {
            event.preventDefault();
            setActiveView(views[next].id);
            document.getElementById(`tab-${views[next].id}`)?.focus();
          }
        }}>
          {views.map((view) => (
            <button key={view.id} id={`tab-${view.id}`} className={`nav-item${activeView === view.id ? " active" : ""}`}
              role="tab" aria-selected={activeView === view.id} aria-controls={`panel-${view.id}`}
              tabIndex={activeView === view.id ? 0 : -1} type="button" onClick={() => setActiveView(view.id)}>
              <span>{view.label}</span>
              <small>{view.hint}</small>
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          <span className="status-dot" /> Local-first · browser capture needs Playwright + Chromium
        </div>
      </aside>
      <section className="content">
        <header className="topbar">
          <div>
            <p className="eyebrow">{activeView.toUpperCase()}</p>
            <h2>{views[index].hint}</h2>
          </div>
          <button className="button secondary" type="button" onClick={() => void chooseVault()}>Select vault</button>
        </header>
        <section id={`panel-${activeView}`} role="tabpanel" aria-labelledby={`tab-${activeView}`} className="view-panel">
          {activeView === "inbox" && (
            <Inbox
              vaultPath={vaultPath}
              status={status}
              setVaultPath={setVaultPath}
              chooseVault={chooseVault}
              runCommand={runCommand}
              message={message}
              setMessage={setMessage}
            />
          )}
          {activeView === "library" && <Library vaultPath={vaultPath} />}
          {activeView === "collections" && <Collections vaultPath={vaultPath} />}
          {activeView === "graph" && <Graph vaultPath={vaultPath} />}
        </section>
      </section>
    </main>
  );
}

function Inbox({
  vaultPath, status, setVaultPath, chooseVault, runCommand, message, setMessage,
}: {
  vaultPath: string;
  status: VaultStatus | null;
  setVaultPath: (path: string) => void;
  chooseVault: () => Promise<void>;
  runCommand: (command: string, args: Record<string, unknown>, onSuccess: (value: ImportResult | number) => string) => Promise<void>;
  message: string;
  setMessage: (value: string) => void;
}) {
  const [token, setToken] = useState("");
  const [githubClientId, setGithubClientId] = useState("");
  const [deviceAuth, setDeviceAuth] = useState<any>(null);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Result[]>([]);
  const [providers, setProviders] = useState<Record<ProviderId, { profile: string; path: string }>>(() => {
    const initial = (provider: ProviderId) => ({ profile: safeReadString(profileStorageKey(provider)), path: "" }); // NOSONAR
    return { linkedin: initial("linkedin"), reddit: initial("reddit"), x: initial("x") };
  });
  const [providerState, setProviderState] = useState<Record<ProviderId, CaptureStatus>>({
    linkedin: "needs-auth", reddit: "needs-auth", x: "needs-auth",
  });

  const updateProvider = (provider: ProviderId, patch: Partial<{ profile: string; path: string }>) => {
    setProviders((current) => ({ ...current, [provider]: { ...current[provider], ...patch } }));
  };

  const persistProfile = (provider: ProviderId, profile: string) => {
    // BUG #8 fix: only persist non-empty profiles so we don't wipe a previously
    // stored path when the user visits the Inbox before choosing a profile.
    if (!profile) return;
    safeWriteString(profileStorageKey(provider), profile);
  };

  const resolveCapturePath = (basename: string): string => {
    // Honour a user-supplied path if they typed one in the capture-file input;
    // otherwise derive a default from the vault path so captures land alongside
    // the imported sources.
    if (!vaultPath) return basename;
    return `${vaultPath.replace(/\/$/, "")}/.researchledger/${basename}`;
  };

  const handleCapture = async (provider: ProviderId) => {
    const profile = (providers[provider]?.profile ?? "").trim();
    if (!profile || !vaultPath) {
      setProviderState((current) => ({ ...current, [provider]: "needs-auth" }));
      setMessage("Set a browser profile and a vault before capturing.");
      return;
    }
    setProviderState((current) => ({ ...current, [provider]: "capturing" }));
    setMessage(`Launching ${PROVIDER_DEFAULTS[provider].label} capture...`);
    try {
      const summary = await invoke<ImportSummary>(PROVIDER_DEFAULTS[provider].captureCommand, {
        profilePath: profile,
        vaultPath,
        outPath: resolveCapturePath(PROVIDER_DEFAULTS[provider].captureBasename),
        url: PROVIDER_DEFAULTS[provider].defaultUrl,
      });
      persistProfile(provider, profile);
      updateProvider(provider, { path: summary.path });
      const total = summary.created + summary.updated;
      setProviderState((current) => ({ ...current, [provider]: "ready" }));
      setMessage(`Captured ${total} ${PROVIDER_DEFAULTS[provider].label} post${total === 1 ? "" : "s"} (${summary.created} new, ${summary.updated} updated).`);
    } catch (error) {
      setProviderState((current) => ({ ...current, [provider]: "needs-auth" }));
      setMessage(formatCommandError(PROVIDER_DEFAULTS[provider].captureCommand, error));
    }
  };

  const handleImport = async (provider: ProviderId) => {
    const pathValue = (providers[provider]?.path ?? "").trim();
    if (!pathValue || !vaultPath) {
      setMessage("Choose a capture file and a vault before importing.");
      return;
    }
    setProviderState((current) => ({ ...current, [provider]: "capturing" }));
    setMessage(`Importing ${PROVIDER_DEFAULTS[provider].label} capture...`);
    try {
      const summary = await invoke<ImportSummary>(PROVIDER_DEFAULTS[provider].captureImportCommand, {
        capturePath: pathValue,
        vaultPath,
      });
      const total = summary.created + summary.updated + summary.unchanged;
      setProviderState((current) => ({ ...current, [provider]: "ready" }));
      setMessage(`Imported ${summary.created + summary.updated} ${PROVIDER_DEFAULTS[provider].label} post${total === 1 ? "" : "s"} (${summary.created} new, ${summary.updated} updated, ${summary.unchanged} unchanged).`);
    } catch (error) {
      setMessage(formatCommandError(PROVIDER_DEFAULTS[provider].captureImportCommand, error));
    }
  };

  const search = async () => {
    if (!vaultPath || !query) return;
    try { setResults(await invoke<Result[]>("search_documents", { vaultPath, query, limit: 20 })); }
    catch { setResults([]); }
  };

  return (
    <div className="inbox-grid">
      <section className="source-rail">
        <p className="eyebrow">SOURCE CONTROL</p>
        <h3>Move research from raw to useful.</h3>
        <p className="muted">Every action stays visible: capture, import, distill, and export from one local queue.</p>
        <div className="queue-stats">
          <span><strong>{status?.documentCount ?? 0}</strong> indexed</span>
          <span><strong>25</strong> enrichment batch</span>
        </div>
      </section>

      <section className="action-stack" aria-label="Quick actions">
        <article className="action-card">
          <div>
            <strong>GitHub</strong>
            <span>Token cleared after import</span>
          </div>
          <button className="button secondary" type="button"
            onClick={() => void runCommand("import_github", { vaultPath, token }, (value) => {
              const v = value as ImportResult;
              setToken("");
              return `Imported ${v.created + v.updated} repositories; ${v.unchanged} unchanged.`;
            })}>
            Import starred repos
          </button>
        </article>
        <article className="action-card">
          <div>
            <strong>Enrichment</strong>
            <span>Up to 25 sources</span>
          </div>
          <button className="button secondary" type="button"
            onClick={() => void runCommand("process_pending_enrichment", { vaultPath, limit: 25 },
              (value) => `Created ${(value as ImportResult).created + (value as ImportResult).updated} distilled notes.`)}>
            Distill pending notes
          </button>
        </article>
      </section>

      <section className="provider-grid" aria-label="Provider capture panels">
        {(["linkedin", "reddit", "x"] as const).map((provider) => (
          <ProviderCapturePanel
            key={provider}
            provider={provider}
            profileValue={providers[provider].profile}
            pathValue={providers[provider].path}
            status={providerState[provider]}
            onProfileChange={(value) => updateProvider(provider, { profile: value })}
            onPathChange={(value) => updateProvider(provider, { path: value })}
            onCapture={() => void handleCapture(provider)}
            onImport={() => void handleImport(provider)}
          />
        ))}
      </section>

      <section className="vault-strip">
        <div>
          <p className="eyebrow">VAULT</p>
          <strong title={vaultPath || "No local vault selected"}>{vaultPath || "No local vault selected"}</strong>
          <span>{status ? `${status.documentCount} documents indexed` : "Choose a Markdown vault to begin"}</span>
        </div>
        <button className="button secondary" type="button" onClick={() => void chooseVault()}>Choose vault</button>
        <input aria-label="Vault path" placeholder="/Users/you/ResearchVault"
          value={vaultPath}
          onChange={(event) => setVaultPath(event.target.value)}
          onFocus={(event) => event.currentTarget.select()}
          title={vaultPath || "/Users/you/ResearchVault"} />
      </section>

      <section className="search-panel" aria-label="Search">
        <input aria-label="Search research" placeholder="Search your ledger…" value={query}
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => { if (event.key === "Enter") void search(); }} />
        <button className="button secondary" type="button" onClick={() => void search()}>Search</button>
        {results.length > 0 && (
          <div className="results">
            {results.map((result) => (
              <article className="result" key={result.documentId}>
                <strong>{result.title}</strong>
                <p><HighlightedSnippet value={result.snippet} /></p>
              </article>
            ))}
          </div>
        )}
      </section>

      <section className="export-row">
        <input aria-label="GitHub App client ID" placeholder="GitHub App client ID (optional OAuth)"
          value={githubClientId} onChange={(event) => setGithubClientId(event.target.value)} />
        <button className="button secondary" type="button"
          onClick={async () => {
            try {
              setDeviceAuth(await invoke("github_device_start", { clientId: githubClientId }));
              setMessage("GitHub verification code ready.");
            } catch (error) {
              setMessage(formatCommandError("github_device_start", error));
            }
          }}>
          Connect GitHub
        </button>
        <input aria-label="GitHub token" type="password" placeholder="GitHub token (never stored)"
          value={token} onChange={(event) => setToken(event.target.value)} />
        <button className="button secondary" type="button"
          onClick={() => void runCommand("export_obsidian", { vaultPath, destination: "" },
            (count) => `Exported ${count as number} Markdown documents.`)}>
          Export Markdown
        </button>
      </section>

      {deviceAuth && (
        <p className="import-message" role="status">
          Open {deviceAuth.verificationUri} and enter <strong>{deviceAuth.userCode}</strong>, then import starred repositories.
        </p>
      )}
      {message && <p className="import-message" role="status">{message}</p>}
    </div>
  );
}

function ProviderCapturePanel({
  provider, profileValue, pathValue, status,
  onProfileChange, onPathChange, onCapture, onImport,
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
  const profilePlaceholder = useMemo(() => `/Users/you/Library/Application Support/ResearchLedger/${defaults.profileBasename}`, [defaults.profileBasename]);
  const pathPlaceholder = useMemo(() => `/Users/you/captures/${defaults.captureBasename}`, [defaults.captureBasename]);
  const pillClass = status === "capturing" ? "capturing" : status === "ready" ? "ready" : "needs-auth";
  const pillLabel = status === "capturing" ? "CAPTURING" : status === "ready" ? "READY" : "AUTH REQUIRED";
  return (
    <article className="provider-card" aria-label={defaults.label + " capture"}>
      <div className="provider-card-head">
        <div>
          <p className="eyebrow">{defaults.label}</p>
          <h4>{defaults.description}</h4>
        </div>
        <span className={`state-pill ${pillClass}`}>{pillLabel}</span>
      </div>
      <p className="muted">Browser profile</p>
      <input aria-label={`${defaults.label} browser profile`} placeholder={profilePlaceholder} title={profilePlaceholder}
        value={profileValue} onChange={(event) => onProfileChange(event.target.value)}
        onFocus={(event) => event.currentTarget.select()} />
      <p className="muted">Capture file</p>
      <input aria-label={`${defaults.label} capture path`} placeholder={pathPlaceholder} title={pathPlaceholder}
        value={pathValue} onChange={(event) => onPathChange(event.target.value)}
        onFocus={(event) => event.currentTarget.select()} />
      <div className="capture-actions">
        <button className="button primary" type="button" onClick={onCapture}
          disabled={status === "capturing" || !profileValue}>
          Capture in browser
        </button>
        <button className="button secondary" type="button" onClick={onImport} disabled={!pathValue}>
          Import file
        </button>
      </div>
    </article>
  );
}

function WorkspaceView({
  title, eyebrow, description, command, vaultPath, render,
}: {
  title: string; eyebrow: string; description: string; command: string; vaultPath: string; render?: (data: any[]) => ReactNode;
}) {
  const [data, setData] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const load = async () => {
    if (!vaultPath) return;
    setLoading(true);
    try { setData(await invoke<any[]>(command, { vaultPath })); }
    catch { setData([]); }
    finally { setLoading(false); }
  };
  useEffect(() => { if (vaultPath) void load(); }, [vaultPath]);
  return (
    <section className="workspace-view">
      <div className="workspace-intro">
        <p className="eyebrow">{eyebrow}</p>
        <h3>{title}</h3>
        <p className="muted">{description}</p>
        <button className="button secondary" type="button" onClick={() => void load()}
          disabled={!vaultPath || loading}>
          {loading ? "Refreshing…" : "Refresh view"}
        </button>
      </div>
      {render ? render(data) : (
        <div className="data-surface">
          {data.length ? data.map((item, index) => (
            <div className="data-row" key={item.id ?? index}>
              <strong>{item.title ?? item.name ?? `Research item ${index + 1}`}</strong>
              <span>{item.description ?? item.kind ?? item.sourceKind ?? "Indexed locally"}</span>
            </div>
          )) : <EmptyState vaultPath={vaultPath} />}
        </div>
      )}
    </section>
  );
}

function Library({ vaultPath }: { vaultPath: string }) {
  return <WorkspaceView title="Your indexed corpus" eyebrow="LIBRARY"
    description="A searchable inventory of every source ResearchLedger has accepted into the vault."
    command="list_document_summaries" vaultPath={vaultPath} />;
}
function Collections({ vaultPath }: { vaultPath: string }) {
  return <WorkspaceView title="Working sets for active questions" eyebrow="COLLECTIONS"
    description="Collections are deliberate slices of the corpus. The view is ready for collection commands as they land."
    command="list_collections" vaultPath={vaultPath} />;
}
function Graph({ vaultPath }: { vaultPath: string }) {
  return <WorkspaceView title="Connections worth following" eyebrow="GRAPH"
    description="A relationship surface for sources, distilled notes, and recurring themes."
    command="list_document_links" vaultPath={vaultPath} render={(data) => (
      <div className="graph-surface">
        {data.length ? data.map((link, index) => (
          <div className="data-row" key={`${link.sourceDocumentId}-${link.targetUrl}-${index}`}>
            <strong>{link.sourceTitle}</strong>
            <span>{link.relation} → {link.targetUrl}</span>
          </div>
        )) : <EmptyState vaultPath={vaultPath} />}
      </div>
    )} />;
}

function EmptyState({ vaultPath }: { vaultPath: string }) {
  return (
    <div className="empty-state">
      <span className="empty-orbit">+</span>
      <strong>{vaultPath ? "Nothing surfaced yet" : "Choose a vault to begin"}</strong>
      <p>{vaultPath ? "Run an import or distill pending sources in Inbox, then refresh this view." : "Your local vault is the source of truth for every workspace."}</p>
    </div>
  );
}

function HighlightedSnippet({ value }: { value: string }) {
  // SQLite's snippet() interleaves `<mark>…</mark>` tokens with plain text. After
  // removing the marker tokens, the remaining odd-indexed segments are the
  // highlighted runs and the even-indexed segments are the surrounding text.
  const segments = value.split(/(<mark>|<\/mark>)/g).filter((segment) => segment && segment !== "<mark>" && segment !== "</mark>");
  return segments.map((segment, index) =>
    index % 2 === 1 ? <mark key={index}>{segment}</mark> : <span key={index}>{segment}</span>,
  );
}
