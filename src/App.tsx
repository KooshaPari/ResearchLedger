import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";

type VaultStatus = {
  selected: boolean;
  path: string | null;
  documentCount: number;
};
type PrimaryView = "inbox" | "library" | "collections" | "graph";
const views: Array<{ id: PrimaryView; label: string }> = [
  { id: "inbox", label: "Inbox" },
  { id: "library", label: "Library" },
  { id: "collections", label: "Collections" },
  { id: "graph", label: "Graph" },
];

export function App() {
  const [activeView, setActiveView] = useState<PrimaryView>("inbox");
  const [vaultPath, setVaultPath] = useState(() =>
    typeof localStorage === "undefined" ? "" : localStorage.getItem("researchledger.vaultPath") ?? "",
  );
  useEffect(() => {
    if (vaultPath && typeof localStorage !== "undefined") {
      localStorage.setItem("researchledger.vaultPath", vaultPath);
    }
  }, [vaultPath]);
  const chooseVault = async () => {
    const selected = await open({ directory: true, multiple: false, title: "Choose ResearchLedger vault" });
    if (typeof selected === "string") setVaultPath(selected);
  };
  return (
    <main className="shell">
      <aside className="sidebar">
        <p className="eyebrow">LOCAL RESEARCH SYSTEM</p>
        <h1>ResearchLedger</h1>
        <nav aria-label="Primary navigation" role="tablist" onKeyDown={(event) => {
          const index = views.findIndex((view) => view.id === activeView);
          const next = event.key === "ArrowRight" ? (index + 1) % views.length : event.key === "ArrowLeft" ? (index + views.length - 1) % views.length : event.key === "Home" ? 0 : event.key === "End" ? views.length - 1 : -1;
          if (next >= 0) { event.preventDefault(); setActiveView(views[next].id); document.getElementById(`tab-${views[next].id}`)?.focus(); }
        }}>
          {views.map((view) => <button key={view.id} id={`tab-${view.id}`} className={`nav-item${activeView === view.id ? " active" : ""}`} role="tab" aria-selected={activeView === view.id} aria-controls={`panel-${view.id}`} tabIndex={activeView === view.id ? 0 : -1} type="button" onClick={() => setActiveView(view.id)}>{view.label}</button>)}
        </nav>
        <div className="sidebar-footer">
          <span className="status-dot" />
          Local-first · offline ready
        </div>
      </aside>
      <section className="content">
        <header className="topbar">
          <div>
            <p className="eyebrow">INBOX</p>
            <h2>Build your research corpus</h2>
          </div>
          <button className="button secondary" type="button" onClick={() => void chooseVault()}>Select vault</button>
        </header>
        <section id={`panel-${activeView}`} role="tabpanel" aria-labelledby={`tab-${activeView}`} className="view-panel">
        {activeView === "inbox" ? <>
        <SearchPanel vaultPath={vaultPath} />
        <section className="hero-card" aria-label="Vault setup">
          <div className="hero-mark">RL</div>
          <div>
            <VaultStatusPanel vaultPath={vaultPath} setVaultPath={setVaultPath} chooseVault={chooseVault} />
          </div>
        </section>
        </> : <section className="empty-view"><p className="eyebrow">{activeView.toUpperCase()}</p><h2>{views.find((view) => view.id === activeView)?.label}</h2><p className="muted">This workspace is connected to the same local vault and will surface enriched research as it is indexed.</p></section>}
        </section>
      </section>
    </main>
  );
}

function SearchPanel({ vaultPath }: { vaultPath: string }) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Array<{ documentId: string; title: string; snippet: string; sourceUri: string | null }>>([]);
  const search = async () => {
    if (!vaultPath || !query) return;
    try { setResults(await invoke("search_documents", { vaultPath, query, limit: 20 })); } catch { setResults([]); }
  };
  return (
    <section className="search-panel" aria-label="Search">
      <input aria-label="Search research" placeholder="Search your ledger…" value={query} onChange={(event) => setQuery(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") void search(); }} />
      <button className="button secondary" type="button" onClick={() => void search()}>Search</button>
      {results.length > 0 && <div className="results">{results.map((result) => <article className="result" key={result.documentId}><strong>{result.title}</strong><p><HighlightedSnippet value={result.snippet} /></p></article>)}</div>}
    </section>
  );
}

function HighlightedSnippet({ value }: { value: string }) {
  return value.split(/(<mark>|<\/mark>)/g).map((part, index) => {
    if (part === "<mark>" || part === "</mark>" || !part) return null;
    const marked = value.split(/(<mark>|<\/mark>)/g).slice(0, index).filter((item) => item === "<mark>").length % 2 === 1;
    return marked ? <mark key={`${part}-${index}`}>{part}</mark> : <span key={`${part}-${index}`}>{part}</span>;
  });
}

function VaultStatusPanel({ vaultPath, setVaultPath, chooseVault }: { vaultPath: string; setVaultPath: (path: string) => void; chooseVault: () => Promise<void> }) {
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const [token, setToken] = useState("");
  const [linkedinPath, setLinkedinPath] = useState("");
  const [exportPath, setExportPath] = useState("");
  const [message, setMessage] = useState("");
  const chooseLinkedInExport = async () => {
    const selected = await open({ multiple: false, title: "Choose LinkedIn activity HTML export", filters: [{ name: "HTML", extensions: ["html", "htm"] }] });
    if (typeof selected === "string") setLinkedinPath(selected);
  };


  useEffect(() => {
    invoke<VaultStatus>("get_vault_status", { vaultPath: vaultPath || null }).then(setStatus).catch(() => setStatus(null));
  }, [vaultPath]);

  return (
    <>
      <p className="eyebrow">{status?.selected ? "VAULT READY" : "NO VAULT SELECTED"}</p>
      <h3>Your sources stay yours.</h3>
      <p className="muted">
        {status ? `Local index: ${status.documentCount} documents` : "Choose a local Markdown vault to begin importing and indexing research."}
      </p>
      <div className="import-form">
        <button className="button secondary" type="button" onClick={() => void chooseVault()}>Choose local vault</button>
        <input aria-label="Vault path" placeholder="/Users/you/ResearchVault" value={vaultPath} onChange={(event) => setVaultPath(event.target.value)} />
        <input aria-label="GitHub token" type="password" placeholder="GitHub token (never stored)" value={token} onChange={(event) => setToken(event.target.value)} />
        <button className="button primary" type="button" onClick={async () => {
          setMessage("Importing starred repositories…");
          try {
            const result = await invoke<{ created: number; updated: number; unchanged: number; failed: number }>("import_github", { vaultPath, token });
            setToken("");
            setMessage(`Imported ${result.created + result.updated} repositories; ${result.unchanged} unchanged.`);
          } catch (error) { setMessage(String(error)); }
        }}>Import GitHub stars</button>
        <input aria-label="LinkedIn HTML export path" placeholder="/Users/you/Downloads/LinkedIn activity.html" value={linkedinPath} onChange={(event) => setLinkedinPath(event.target.value)} />
        <button className="button secondary" type="button" onClick={() => void chooseLinkedInExport()}>Choose LinkedIn export</button>
        <button className="button secondary" type="button" onClick={async () => {
          setMessage("Importing LinkedIn activity export…");
          try {
            const result = await invoke<{ created: number; updated: number; unchanged: number; failed: number }>("import_linkedin_html", { vaultPath, htmlPath: linkedinPath });
            setMessage(`Imported ${result.created + result.updated} LinkedIn posts; ${result.unchanged} unchanged.`);
          } catch (error) { setMessage(String(error)); }
        }}>Import LinkedIn HTML</button>
        <input aria-label="Obsidian export path" placeholder="/Users/you/ResearchVault-export" value={exportPath} onChange={(event) => setExportPath(event.target.value)} />
        <button className="button secondary" type="button" onClick={async () => {
          try { const count = await invoke<number>("export_obsidian", { vaultPath, destination: exportPath }); setMessage(`Exported ${count} Markdown documents.`); } catch (error) { setMessage(String(error)); }
        }}>Export Markdown vault</button>
        {message && <p className="import-message" role="status">{message}</p>}
      </div>
    </>
  );
}
