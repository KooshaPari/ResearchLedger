import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState, type ReactNode } from "react";
import { HackerNewsPanel } from "./HackerNewsPanel";

function formatCommandError(command: string, error: unknown): string {
  const raw = String(error ?? "");
  const stripped = raw.replace(/\x1b\[[0-9;]*m/g, "");
  const safe = stripped
    .replace(/\/Users\/[^\s'"]+/g, "[path]")
    .replace(/\/home\/[^\s'"]+/g, "[path]");
  if (
    /Executable doesn'?t exist|ERR_MODULE_NOT_FOUND|Cannot find package ['"]?playwright|chromium.*not installed/i.test(
      safe,
    )
  ) {
    return `Could not run ${command}: Playwright Chromium is not installed. Install the bundled browser runtime, then retry.`;
  }
  if (
    /AUTH[_ ]REQUIRED|not authenticated|sign(?:ed)? in|login required|login page/i.test(
      safe,
    )
  ) {
    return `Could not run ${command}: this provider needs an authenticated browser profile. Sign in, then retry.`;
  }
  if (/timed? ?out|timeout|deadline/i.test(safe)) {
    return `Could not run ${command}: the provider timed out. Check your connection and retry.`;
  }
  if (/rate limit|too many requests/i.test(safe)) {
    return `Could not run ${command}: the provider rate-limited this request. Wait a moment and retry.`;
  }
  if (/ENOENT|no such file|cannot find the path/i.test(safe)) {
    return `Could not run ${command}: the selected capture file or profile path could not be found.`;
  }
  return `Could not run ${command}: ${safe || "unknown provider error"}`;
}

type VaultStatus = {
  selected: boolean;
  path: string | null;
  documentCount: number;
};
type PrimaryView = "inbox" | "library" | "collections" | "graph";
type Result = {
  documentId: string;
  title: string;
  snippet: string;
  sourceUri: string | null;
};
type ImportResult = {
  created: number;
  updated: number;
  unchanged: number;
  failed: number;
};
type DeviceAuthorization = {
  deviceCode: string;
  userCode: string;
  verificationUri: string;
  expiresIn: number;
  interval: number;
};
const views: Array<{ id: PrimaryView; label: string; hint: string }> = [
  {
    id: "inbox",
    label: "Inbox",
    hint: "Capture, review, and move sources forward",
  },
  {
    id: "library",
    label: "Library",
    hint: "Browse the indexed research corpus",
  },
  {
    id: "collections",
    label: "Collections",
    hint: "Group sources into working sets",
  },
  { id: "graph", label: "Graph", hint: "See how ideas and sources connect" },
];

export function App() {
  const [activeView, setActiveView] = useState<PrimaryView>("inbox");
  const [vaultPath, setVaultPath] = useState(() =>
    typeof localStorage === "undefined"
      ? ""
      : (localStorage.getItem("researchledger.vaultPath") ?? ""),
  );
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const [message, setMessage] = useState("");
  useEffect(() => {
    if (vaultPath) localStorage.setItem("researchledger.vaultPath", vaultPath);
  }, [vaultPath]);
  useEffect(() => {
    invoke<VaultStatus>("get_vault_status", { vaultPath: vaultPath || null })
      .then(setStatus)
      .catch(() => setStatus(null));
  }, [vaultPath]);
  const chooseVault = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choose ResearchLedger vault",
    });
    if (typeof selected === "string") setVaultPath(selected);
  };
  const run = async (
    command: string,
    args: Record<string, unknown>,
    success: (value: any) => string,
  ) => {
    if (!vaultPath) {
      setMessage("Select a vault before running a source action.");
      return;
    }
    try {
      setMessage(success(await invoke(command, args)));
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
        <nav
          aria-label="Primary navigation"
          role="tablist"
          onKeyDown={(event) => {
            const next =
              event.key === "ArrowRight"
                ? (index + 1) % views.length
                : event.key === "ArrowLeft"
                  ? (index + views.length - 1) % views.length
                  : event.key === "Home"
                    ? 0
                    : event.key === "End"
                      ? views.length - 1
                      : -1;
            if (next >= 0) {
              event.preventDefault();
              setActiveView(views[next].id);
              document.getElementById(`tab-${views[next].id}`)?.focus();
            }
          }}
        >
          {views.map((view) => (
            <button
              key={view.id}
              id={`tab-${view.id}`}
              className={`nav-item${activeView === view.id ? " active" : ""}`}
              role="tab"
              aria-selected={activeView === view.id}
              aria-controls={`panel-${view.id}`}
              tabIndex={activeView === view.id ? 0 : -1}
              type="button"
              onClick={() => setActiveView(view.id)}
            >
              <span>{view.label}</span>
              <small>{view.hint}</small>
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          <span className="status-dot" /> Local-first · browser capture needs
          Playwright + Chromium
        </div>
      </aside>
      <section className="content">
        <header className="topbar">
          <div>
            <p className="eyebrow">{activeView.toUpperCase()}</p>
            <h2>{views[index].hint}</h2>
          </div>
          <button
            className="button secondary"
            type="button"
            onClick={() => void chooseVault()}
          >
            Select vault
          </button>
        </header>
        <section
          id="panel-inbox"
          role="tabpanel"
          aria-labelledby="tab-inbox"
          className="view-panel"
          hidden={activeView !== "inbox"}
        >
          <Inbox
            vaultPath={vaultPath}
            status={status}
            setVaultPath={setVaultPath}
            chooseVault={chooseVault}
            run={run}
            message={message}
            setMessage={setMessage}
          />
        </section>
        <section
          id="panel-library"
          role="tabpanel"
          aria-labelledby="tab-library"
          className="view-panel"
          hidden={activeView !== "library"}
        >
          <Library vaultPath={vaultPath} />
        </section>
        <section
          id="panel-collections"
          role="tabpanel"
          aria-labelledby="tab-collections"
          className="view-panel"
          hidden={activeView !== "collections"}
        >
          <Collections vaultPath={vaultPath} />
        </section>
        <section
          id="panel-graph"
          role="tabpanel"
          aria-labelledby="tab-graph"
          className="view-panel"
          hidden={activeView !== "graph"}
        >
          <Graph vaultPath={vaultPath} />
        </section>
      </section>
    </main>
  );
}

function Inbox({
  vaultPath,
  status,
  setVaultPath,
  chooseVault,
  run,
  message,
  setMessage,
}: {
  vaultPath: string;
  status: VaultStatus | null;
  setVaultPath: (path: string) => void;
  chooseVault: () => Promise<void>;
  run: (
    command: string,
    args: Record<string, unknown>,
    success: (value: any) => string,
  ) => Promise<void>;
  message: string;
  setMessage: (value: string) => void;
}) {
  const [token, setToken] = useState("");
  const [hackernewsProfile, setHackernewsProfile] = useState(() =>
    typeof localStorage === "undefined"
      ? ""
      : (localStorage.getItem("researchledger.hackernewsProfile") ?? ""),
  );
  const [hackernewsUsername, setHackernewsUsername] = useState(() =>
    typeof localStorage === "undefined"
      ? ""
      : (localStorage.getItem("researchledger.hackernewsUsername") ?? ""),
  );
  const [linkedinPath, setLinkedinPath] = useState("");
  const [linkedinProfile, setLinkedinProfile] = useState(() =>
    typeof localStorage === "undefined"
      ? ""
      : (localStorage.getItem("researchledger.linkedinProfile") ?? ""),
  );
  const [githubClientId, setGithubClientId] = useState("");
  const [deviceAuth, setDeviceAuth] = useState<DeviceAuthorization | null>(
    null,
  );
  const [githubState, setGithubState] = useState<
    "needs-config" | "waiting" | "ready"
  >("needs-config");
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Result[]>([]);
  const [linkedinState, setLinkedinState] = useState<
    "needs-auth" | "ready" | "capturing"
  >("needs-auth");
  const [hnState, setHnState] = useState<"needs-auth" | "ready" | "capturing">(
    "needs-auth",
  );
  const [redditState, setRedditState] = useState<
    "needs-auth" | "ready" | "capturing"
  >("needs-auth");
  const [xState, setXState] = useState<"needs-auth" | "ready" | "capturing">(
    "needs-auth",
  );
  const [redditPath, setRedditPath] = useState("");
  const [redditProfile, setRedditProfile] = useState(() =>
    typeof localStorage === "undefined"
      ? ""
      : (localStorage.getItem("researchledger.redditProfile") ?? ""),
  );
  const [redditUsername, setRedditUsername] = useState(() =>
    typeof localStorage === "undefined"
      ? ""
      : (localStorage.getItem("researchledger.redditUsername") ?? ""),
  );
  const [xPath, setXPath] = useState("");
  const [xProfile, setXProfile] = useState(() =>
    typeof localStorage === "undefined"
      ? ""
      : (localStorage.getItem("researchledger.xProfile") ?? ""),
  );
  useEffect(() => {
    if (!deviceAuth || !githubClientId.trim()) return;
    let cancelled = false;
    setGithubState("waiting");
    void invoke<string>("github_device_poll", {
      clientId: githubClientId.trim(),
      deviceCode: deviceAuth.deviceCode,
      interval: deviceAuth.interval,
      expiresIn: deviceAuth.expiresIn,
    })
      .then((githubToken) => {
        if (cancelled) return;
        setToken(githubToken);
        setGithubState("ready");
        setDeviceAuth(null);
        setMessage("GitHub connected. Import starred repositories when ready.");
      })
      .catch((error) => {
        if (cancelled) return;
        setGithubState("needs-config");
        setDeviceAuth(null);
        setMessage(formatCommandError("github_device_poll", error));
      });
    return () => {
      cancelled = true;
    };
  }, [deviceAuth, githubClientId, setMessage]);
  const captureHackerNews = async () => {
    if (hackernewsProfile && typeof localStorage !== "undefined")
      localStorage.setItem(
        "researchledger.hackernewsProfile",
        hackernewsProfile,
      );
    const username = hackernewsUsername ? hackernewsUsername.trim() : "";
    if (!username) {
      setHnState("needs-auth");
      setMessage(
        "Sign in to news.ycombinator.com first, then enter your Hacker News username.",
      );
      return;
    }
    setHnState("capturing");
    const url =
      "https://news.ycombinator.com/saved?id=" + encodeURIComponent(username);
    try {
      const value = await invoke("capture_hackernews_browser", {
        vaultPath,
        activityUrl: url,
        profilePath: hackernewsProfile || null,
      } as any);
      setMessage(
        "Captured " +
          ((value as any).created + (value as any).updated) +
          " Hacker News stories; " +
          (value as any).unchanged +
          " unchanged.",
      );
    } catch (error) {
      setMessage(formatCommandError("capture_hackernews_browser", error));
    }
    setHnState("ready");
  };
  const captureLinkedIn = async () => {
    if (linkedinProfile && typeof localStorage !== "undefined")
      localStorage.setItem("researchledger.linkedinProfile", linkedinProfile);
    setLinkedinState("capturing");
    await run(
      "capture_linkedin_browser",
      { vaultPath, activityUrl: null, profilePath: linkedinProfile || null },
      (value: ImportResult) =>
        `Captured ${value.created + value.updated} LinkedIn posts; ${value.unchanged} unchanged.`,
    );
    setLinkedinState("ready");
  };
  const openLinkedInSignin = async () => {
    if (linkedinProfile && typeof localStorage !== "undefined")
      localStorage.setItem("researchledger.linkedinProfile", linkedinProfile);
    try {
      setMessage(
        await invoke<string>("open_linkedin_signin", {
          profilePath: linkedinProfile || null,
        }),
      );
    } catch (error) {
      setMessage(formatCommandError("open_linkedin_signin", error));
    }
  };
  const captureReddit = async () => {
    if (redditProfile)
      localStorage.setItem("researchledger.redditProfile", redditProfile);
    const username = redditUsername.trim();
    if (!username) {
      setRedditState("needs-auth");
      setMessage("Enter your Reddit username first.");
      return;
    }
    setRedditState("capturing");
    try {
      const value = await invoke("capture_reddit_browser", {
        vaultPath,
        activityUrl: `https://www.reddit.com/user/${encodeURIComponent(username)}/saved`,
        profilePath: redditProfile || null,
      } as any);
      setMessage(
        "Captured " +
          ((value as any).created + (value as any).updated) +
          " Reddit posts; " +
          (value as any).unchanged +
          " unchanged.",
      );
    } catch (error) {
      setMessage(formatCommandError("capture_reddit_browser", error));
    }
    setRedditState("ready");
  };
  const captureX = async () => {
    if (xProfile) localStorage.setItem("researchledger.xProfile", xProfile);
    setXState("capturing");
    try {
      const value = await invoke("capture_x_browser", {
        vaultPath,
        activityUrl: "https://x.com/i/bookmarks",
        profilePath: xProfile || null,
      } as any);
      setMessage(
        "Captured " +
          ((value as any).created + (value as any).updated) +
          " X bookmarks; " +
          (value as any).unchanged +
          " unchanged.",
      );
    } catch (error) {
      setMessage(formatCommandError("capture_x_browser", error));
    }
    setXState("ready");
  };
  const search = async () => {
    if (!vaultPath || !query) return;
    try {
      setResults(
        await invoke<Result[]>("search_documents", {
          vaultPath,
          query,
          limit: 20,
        }),
      );
    } catch {
      setResults([]);
    }
  };
  return (
    <>
      <section className="inbox-grid">
        <div className="source-rail">
          <p className="eyebrow">SOURCE CONTROL</p>
          <h3>Move research from raw to useful.</h3>
          <p className="muted">
            Every action stays visible: capture, fetch references, distill, and
            export from one local queue.
          </p>
          <div className="queue-stats">
            <span>
              <strong>{status?.documentCount ?? 0}</strong> indexed
            </span>
            <span>
              <strong>25</strong> enrichment batch
            </span>
          </div>
        </div>
        <div className="action-stack">
          <Action
            title="LinkedIn"
            label={
              linkedinState === "needs-auth"
                ? "Connect browser"
                : linkedinState === "capturing"
                  ? "Capturing…"
                  : "Connected"
            }
            state={
              linkedinState === "needs-auth"
                ? "Needs authentication"
                : linkedinState === "capturing"
                  ? "Capture in progress"
                  : "Ready to capture"
            }
            onClick={() => void captureLinkedIn()}
            disabled={linkedinState === "capturing"}
          />
          <Action
            title="Hacker News"
            label={
              hnState === "needs-auth"
                ? "Connect browser"
                : hnState === "capturing"
                  ? "Capturing..."
                  : "Connected"
            }
            state={
              hnState === "needs-auth"
                ? "Needs authentication"
                : hnState === "capturing"
                  ? "Capture in progress"
                  : "Ready to capture"
            }
            onClick={() => void captureHackerNews()}
            disabled={hnState === "capturing"}
          />
          <Action
            title="Reddit"
            label={
              redditState === "needs-auth"
                ? "Connect browser"
                : redditState === "capturing"
                  ? "Capturing..."
                  : "Connected"
            }
            state={
              redditState === "needs-auth"
                ? "Needs username"
                : redditState === "capturing"
                  ? "Capture in progress"
                  : "Ready to capture"
            }
            onClick={() => void captureReddit()}
            disabled={redditState === "capturing"}
          />
          <Action
            title="X"
            label={
              xState === "needs-auth"
                ? "Connect browser"
                : xState === "capturing"
                  ? "Capturing..."
                  : "Connected"
            }
            state={
              xState === "needs-auth"
                ? "Needs authentication"
                : xState === "capturing"
                  ? "Capture in progress"
                  : "Ready to capture"
            }
            onClick={() => void captureX()}
            disabled={xState === "capturing"}
          />
          <Action
            title="GitHub"
            label="Import starred repos"
            state="Token cleared after import"
            onClick={() =>
              void run(
                "import_github",
                { vaultPath, token },
                (value: ImportResult) => {
                  setToken("");
                  return `Imported ${value.created + value.updated} repositories; ${value.unchanged} unchanged.`;
                },
              )
            }
          />
          <Action
            title="References"
            label="Fetch linked sources"
            state="Up to 10 public pages"
            onClick={() =>
              void run(
                "fetch_pending_references",
                { vaultPath, limit: 10 },
                (value: ImportResult) =>
                  `Fetched ${value.updated} linked sources; ${value.failed} failed.`,
              )
            }
          />
          <Action
            title="Enrichment"
            label="Distill pending notes"
            state="Up to 25 sources"
            onClick={() =>
              void run(
                "process_pending_enrichment",
                { vaultPath, limit: 25 },
                (value: ImportResult) =>
                  `Created ${value.created + value.updated} distilled notes.`,
              )
            }
          />
        </div>
      </section>
      <section className="capture-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">LINKEDIN CONNECTION</p>
            <h3>
              {linkedinState === "needs-auth"
                ? "Sign in once, capture locally"
                : "Browser profile connected"}
            </h3>
          </div>
          <span className={`state-pill ${linkedinState}`}>
            {linkedinState === "needs-auth"
              ? "AUTH REQUIRED"
              : linkedinState === "capturing"
                ? "CAPTURING"
                : "READY"}
          </span>
        </div>
        <p className="muted">
          Use a dedicated persistent Chrome profile. ResearchLedger opens
          LinkedIn in that profile so SSO, cookies, and MFA remain in your
          browser; it never stores your LinkedIn password.
        </p>
        <div className="profile-row">
          <input
            aria-label="LinkedIn browser profile"
            placeholder="Default profile or /Users/you/Library/Application Support/ResearchLedger/linkedin-profile"
            value={linkedinProfile}
            onChange={(event) => setLinkedinProfile(event.target.value)}
          />
          <button
            className="button secondary"
            type="button"
            onClick={() => void openLinkedInSignin()}
            disabled={linkedinState === "capturing"}
          >
            Open LinkedIn sign-in
          </button>
        </div>
        <div className="capture-actions">
          <button
            className="button primary"
            type="button"
            onClick={() => void captureLinkedIn()}
            disabled={linkedinState === "capturing"}
          >
            Capture reactions in browser
          </button>
          <input
            aria-label="LinkedIn capture path"
            placeholder="Optional capture JSON path"
            value={linkedinPath}
            onChange={(event) => setLinkedinPath(event.target.value)}
          />
          <button
            className="button secondary"
            type="button"
            onClick={() =>
              void run(
                "import_linkedin_capture",
                { vaultPath, capturePath: linkedinPath },
                (value: ImportResult) =>
                  `Imported ${value.created + value.updated} LinkedIn posts.`,
              )
            }
          >
            Import capture
          </button>
        </div>
        <p className="import-message">
          Advanced API credentials are not requested because LinkedIn’s approved
          member APIs do not expose a general reactions feed. Use an approved
          integration only through a future provider adapter.
        </p>
      </section>
      <section className="capture-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">REDDIT SAVED POSTS</p>
            <h3>
              {redditState === "needs-auth"
                ? "Sign in once, capture locally"
                : "Reddit saved-posts connected"}
            </h3>
          </div>
          <span className={`state-pill ${redditState}`}>
            {redditState === "needs-auth"
              ? "USERNAME REQUIRED"
              : redditState === "capturing"
                ? "CAPTURING"
                : "READY"}
          </span>
        </div>
        <p className="muted">
          Persistent Chrome profile opens your saved-posts page. Profile path
          keeps the Reddit session in your browser so cookies never touch
          ResearchLedger.
        </p>
        <div className="profile-row">
          <input
            aria-label="Reddit username"
            placeholder="Your Reddit username (no /u/)"
            value={redditUsername}
            onChange={(event) => setRedditUsername(event.target.value)}
          />
          <input
            aria-label="Reddit browser profile"
            title={redditProfile || "Default profile"}
            placeholder="Browser profile path (optional)"
            value={redditProfile}
            onChange={(event) => setRedditProfile(event.target.value)}
          />
        </div>
        <div className="capture-actions">
          <button
            className="button primary"
            type="button"
            onClick={() => void captureReddit()}
            disabled={redditState === "capturing"}
          >
            Capture Reddit saved posts
          </button>
          <input
            aria-label="Reddit capture path"
            placeholder="/Users/you/captures/reddit.json"
            value={redditPath}
            onChange={(event) => setRedditPath(event.target.value)}
          />
          <button
            className="button secondary"
            type="button"
            onClick={() =>
              void run(
                "import_reddit_capture",
                { vaultPath, capturePath: redditPath },
                (value: ImportResult) =>
                  `Imported ${value.created + value.updated} Reddit posts.`,
              )
            }
          >
            Import capture
          </button>
        </div>
      </section>
      <section className="capture-panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">X BOOKMARKS</p>
            <h3>
              {xState === "needs-auth"
                ? "Sign in once, capture locally"
                : "X bookmarks connected"}
            </h3>
          </div>
          <span className={`state-pill ${xState}`}>
            {xState === "needs-auth"
              ? "AUTH REQUIRED"
              : xState === "capturing"
                ? "CAPTURING"
                : "READY"}
          </span>
        </div>
        <p className="muted">
          Persistent Chrome profile opens x.com/i/bookmarks. Profile path keeps
          the X session in your browser so MFA never touches ResearchLedger.
        </p>
        <div className="profile-row">
          <input
            aria-label="X browser profile"
            title={xProfile || "Default profile"}
            placeholder="Browser profile path (optional)"
            value={xProfile}
            onChange={(event) => setXProfile(event.target.value)}
          />
        </div>
        <div className="capture-actions">
          <button
            className="button primary"
            type="button"
            onClick={() => void captureX()}
            disabled={xState === "capturing"}
          >
            Capture X bookmarks
          </button>
          <input
            aria-label="X capture path"
            placeholder="/Users/you/captures/x.json"
            value={xPath}
            onChange={(event) => setXPath(event.target.value)}
          />
          <button
            className="button secondary"
            type="button"
            onClick={() =>
              void run(
                "import_x_capture",
                { vaultPath, capturePath: xPath },
                (value: ImportResult) =>
                  `Imported ${value.created + value.updated} X bookmarks.`,
              )
            }
          >
            Import capture
          </button>
        </div>
      </section>
      <HackerNewsPanel vaultPath={vaultPath} setMessage={setMessage} />
      <section className="vault-strip">
        <div>
          <p className="eyebrow">VAULT</p>
          <strong>{vaultPath || "No local vault selected"}</strong>
          <span>
            {status
              ? `${status.documentCount} documents indexed`
              : "Choose a Markdown vault to begin"}
          </span>
        </div>
        <button
          className="button secondary"
          type="button"
          onClick={() => void chooseVault()}
        >
          Choose vault
        </button>
        <input
          aria-label="Vault path"
          placeholder="/Users/you/ResearchVault"
          title={vaultPath || "No vault selected"}
          value={vaultPath}
          onChange={(event) => setVaultPath(event.target.value)}
        />
      </section>
      <section className="search-panel" aria-label="Search">
        <input
          aria-label="Search research"
          placeholder="Search your ledger…"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") void search();
          }}
        />
        <button
          className="button secondary"
          type="button"
          onClick={() => void search()}
        >
          Search
        </button>
        {results.length > 0 && (
          <div className="results">
            {results.map((result) => (
              <article className="result" key={result.documentId}>
                <strong>{result.title}</strong>
                <p>
                  <HighlightedSnippet value={result.snippet} />
                </p>
              </article>
            ))}
          </div>
        )}
      </section>
      <section className="export-row">
        <input
          aria-label="GitHub App client ID"
          placeholder="GitHub App client ID (required for OAuth)"
          value={githubClientId}
          onChange={(event) => {
            setGithubClientId(event.target.value);
            if (!event.target.value.trim()) setGithubState("needs-config");
          }}
        />
        <button
          className="button secondary"
          type="button"
          disabled={githubState === "waiting"}
          onClick={async () => {
            const clientId = githubClientId.trim();
            if (!clientId) {
              setMessage(
                "Enter your GitHub App client ID to start device sign-in.",
              );
              setGithubState("needs-config");
              return;
            }
            try {
              setGithubState("waiting");
              setDeviceAuth(
                await invoke<DeviceAuthorization>("github_device_start", {
                  clientId,
                }),
              );
              setMessage(
                "GitHub verification code ready. Finish sign-in in your browser; ResearchLedger will poll automatically.",
              );
            } catch (error) {
              setGithubState("needs-config");
              setMessage(formatCommandError("github_device_start", error));
            }
          }}
        >
          {githubState === "waiting"
            ? "Waiting for GitHub…"
            : githubState === "ready"
              ? "GitHub connected"
              : "Connect GitHub"}
        </button>
        <input
          aria-label="GitHub token"
          type="password"
          placeholder="GitHub token (never stored)"
          value={token}
          onChange={(event) => setToken(event.target.value)}
        />
        <button
          className="button secondary"
          type="button"
          onClick={async () => {
            const destination = await open({
              directory: true,
              multiple: false,
              title: "Choose Markdown export folder",
              defaultPath: vaultPath || undefined,
            });
            if (typeof destination !== "string") {
              setMessage(
                "Markdown export canceled; no destination was selected.",
              );
              return;
            }
            await run(
              "export_obsidian",
              { vaultPath, destination },
              (count: number) =>
                `Exported ${count} Markdown documents to ${destination}.`,
            );
          }}
        >
          Export Markdown
        </button>
      </section>
      {deviceAuth && (
        <p className="import-message" role="status">
          Open{" "}
          <a href={deviceAuth.verificationUri} target="_blank" rel="noreferrer">
            {deviceAuth.verificationUri}
          </a>{" "}
          and enter <strong>{deviceAuth.userCode}</strong>. The app will poll
          until GitHub approves or the code expires.
        </p>
      )}
      {message && (
        <p className="import-message" role="status">
          {message}
        </p>
      )}
    </>
  );
}

function Action({
  title,
  label,
  state,
  onClick,
  disabled,
}: {
  title: string;
  label: string;
  state: string;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <article className="action-card">
      <div>
        <strong>{title}</strong>
        <span>{state}</span>
      </div>
      <button
        className="button secondary"
        type="button"
        onClick={onClick}
        disabled={disabled}
      >
        {label}
      </button>
    </article>
  );
}
function WorkspaceView({
  title,
  eyebrow,
  description,
  command,
  vaultPath,
  render,
}: {
  title: string;
  eyebrow: string;
  description: string;
  command: string;
  vaultPath: string;
  render?: (data: any[]) => ReactNode;
}) {
  const [data, setData] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const load = async () => {
    if (!vaultPath) return;
    setLoading(true);
    try {
      setData(await invoke<any[]>(command, { vaultPath }));
    } catch {
      setData([]);
    } finally {
      setLoading(false);
    }
  };
  useEffect(() => {
    if (vaultPath) void load();
  }, [vaultPath]);
  return (
    <section className="workspace-view">
      <div className="workspace-intro">
        <p className="eyebrow">{eyebrow}</p>
        <h3>{title}</h3>
        <p className="muted">{description}</p>
        <button
          className="button secondary"
          type="button"
          onClick={() => void load()}
          disabled={!vaultPath || loading}
        >
          {loading ? "Refreshing…" : "Refresh view"}
        </button>
      </div>
      {render ? (
        render(data)
      ) : (
        <div className="data-surface">
          {data.length ? (
            data.map((item, index) => (
              <div className="data-row" key={item.id ?? index}>
                <strong>
                  {item.title ?? item.name ?? `Research item ${index + 1}`}
                </strong>
                <span>
                  {item.description ??
                    item.kind ??
                    item.sourceKind ??
                    "Indexed locally"}
                </span>
              </div>
            ))
          ) : (
            <EmptyState vaultPath={vaultPath} />
          )}
        </div>
      )}
    </section>
  );
}
function Library({ vaultPath }: { vaultPath: string }) {
  return <DetailedLibrary vaultPath={vaultPath} />;
}
function DetailedLibrary({ vaultPath }: { vaultPath: string }) {
  const [documents, setDocuments] = useState<
    Array<{
      id: string;
      title: string;
      sourceKind: string;
      sourceUri: string | null;
      tags: string[];
    }>
  >([]);
  const [selectedId, setSelectedId] = useState("");
  const [claims, setClaims] = useState<
    Array<{ claim: string; sourceUri: string | null; citationId: string }>
  >([]);
  const [loading, setLoading] = useState(false);
  const load = async () => {
    if (!vaultPath) return;
    setLoading(true);
    try {
      setDocuments(await invoke("list_document_summaries", { vaultPath }));
    } catch {
      setDocuments([]);
    } finally {
      setLoading(false);
    }
  };
  useEffect(() => {
    void load();
  }, [vaultPath]);
  useEffect(() => {
    if (!vaultPath || !selectedId) {
      setClaims([]);
      return;
    }
    void invoke<
      Array<{ claim: string; sourceUri: string | null; citationId: string }>
    >("list_document_claims", { vaultPath, documentId: selectedId })
      .then(setClaims)
      .catch(() => setClaims([]));
  }, [vaultPath, selectedId]);
  return (
    <section className="workspace-view">
      <div className="workspace-intro">
        <p className="eyebrow">LIBRARY</p>
        <h3>Your indexed corpus</h3>
        <p className="muted">
          Select a document to inspect its persisted claims and source
          citations.
        </p>
        <button
          className="button secondary"
          type="button"
          onClick={() => void load()}
          disabled={!vaultPath || loading}
        >
          {loading ? "Refreshing…" : "Refresh view"}
        </button>
      </div>
      <div className="data-surface">
        {documents.length ? (
          documents.map((document) => (
            <button
              className={`data-row${selectedId === document.id ? " selected" : ""}`}
              key={document.id}
              type="button"
              onClick={() => setSelectedId(document.id)}
            >
              <strong>{document.title}</strong>
              <span>
                {document.sourceKind} · {document.tags.join(", ") || "untagged"}
              </span>
            </button>
          ))
        ) : (
          <EmptyState vaultPath={vaultPath} />
        )}
      </div>
      {selectedId && (
        <aside className="data-surface">
          <p className="eyebrow">CLAIMS / PROVENANCE</p>
          {claims.length ? (
            claims.map((claim, index) => (
              <div className="data-row" key={`${claim.citationId}-${index}`}>
                <strong>
                  [{claim.citationId}] {claim.claim}
                </strong>
                <span>{claim.sourceUri || "Local source"}</span>
              </div>
            ))
          ) : (
            <p className="muted">
              No claim records yet. Distill this source from Inbox.
            </p>
          )}
        </aside>
      )}
    </section>
  );
}
function Collections({ vaultPath }: { vaultPath: string }) {
  return (
    <WorkspaceView
      title="Working sets for active questions"
      eyebrow="COLLECTIONS"
      description="Collections are deliberate slices of the corpus. The view is ready for collection commands as they land."
      command="list_collections"
      vaultPath={vaultPath}
    />
  );
}
function Graph({ vaultPath }: { vaultPath: string }) {
  return (
    <WorkspaceView
      title="Connections worth following"
      eyebrow="GRAPH"
      description="A relationship surface for sources, distilled notes, and recurring themes."
      command="list_document_links"
      vaultPath={vaultPath}
      render={(data) => (
        <div className="graph-surface">
          {data.length ? (
            data.map((link, index) => (
              <div
                className="data-row"
                key={`${link.sourceDocumentId}-${link.targetUrl}-${index}`}
              >
                <strong>{link.sourceTitle}</strong>
                <span>
                  {link.relation} → {link.targetUrl}
                </span>
              </div>
            ))
          ) : (
            <EmptyState vaultPath={vaultPath} />
          )}
        </div>
      )}
    />
  );
}
function EmptyState({ vaultPath }: { vaultPath: string }) {
  return (
    <div className="empty-state">
      <span className="empty-orbit">+</span>
      <strong>
        {vaultPath ? "Nothing surfaced yet" : "Choose a vault to begin"}
      </strong>
      <p>
        {vaultPath
          ? "Run an import or distill pending sources in Inbox, then refresh this view."
          : "Your local vault is the source of truth for every workspace."}
      </p>
    </div>
  );
}
function HighlightedSnippet({ value }: { value: string }) {
  return value.split(/(<mark>|<\/mark>)/g).map((part, index) =>
    part === "<mark>" || part === "</mark>" || !part ? null : (
      (value
        .split(/(<mark>|<\/mark>)/g)
        .slice(0, index)
        .filter((item) => item === "<mark>").length %
        2 ===
        1) ? (
        <mark key={index}>{part}</mark>
      ) : (
        <span key={index}>{part}</span>
      )
    ),
  );
}
