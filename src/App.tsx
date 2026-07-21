import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type VaultStatus = {
  selected: boolean;
  path: string | null;
  documentCount: number;
};

export function App() {
  return (
    <main className="shell">
      <aside className="sidebar">
        <p className="eyebrow">LOCAL RESEARCH SYSTEM</p>
        <h1>ResearchLedger</h1>
        <nav aria-label="Primary navigation">
          <button className="nav-item active">Inbox</button>
          <button className="nav-item">Library</button>
          <button className="nav-item">Collections</button>
          <button className="nav-item">Graph</button>
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
          <button className="button secondary" type="button">Select vault</button>
        </header>
        <SearchPanel />
        <section className="hero-card" aria-label="Vault setup">
          <div className="hero-mark">RL</div>
          <div>
            <VaultStatusPanel />
          </div>
        </section>
      </section>
    </main>
  );
}

function SearchPanel() {
  const [vaultPath, setVaultPath] = useState("");
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
      {results.length > 0 && <div className="results">{results.map((result) => <article className="result" key={result.documentId}><strong>{result.title}</strong><p dangerouslySetInnerHTML={{ __html: result.snippet }} /></article>)}</div>}
    </section>
  );
}

function VaultStatusPanel() {
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const [vaultPath, setVaultPath] = useState("");
  const [token, setToken] = useState("");
  const [exportPath, setExportPath] = useState("");
  const [message, setMessage] = useState("");

  useEffect(() => {
    invoke<VaultStatus>("get_vault_status").then(setStatus).catch(() => setStatus(null));
  }, []);

  return (
    <>
      <p className="eyebrow">{status?.selected ? "VAULT READY" : "NO VAULT SELECTED"}</p>
      <h3>Your sources stay yours.</h3>
      <p className="muted">
        {status ? `Local index: ${status.documentCount} documents` : "Choose a local Markdown vault to begin importing and indexing research."}
      </p>
      <div className="import-form">
        <button className="button secondary" type="button">Select local vault</button>
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
        <input aria-label="Obsidian export path" placeholder="/Users/you/ResearchVault-export" value={exportPath} onChange={(event) => setExportPath(event.target.value)} />
        <button className="button secondary" type="button" onClick={async () => {
          try { const count = await invoke<number>("export_obsidian", { vaultPath, destination: exportPath }); setMessage(`Exported ${count} Markdown documents.`); } catch (error) { setMessage(String(error)); }
        }}>Export Markdown vault</button>
        {message && <p className="import-message" role="status">{message}</p>}
      </div>
    </>
  );
}
