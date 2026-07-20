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

function VaultStatusPanel() {
  const [status, setStatus] = useState<VaultStatus | null>(null);

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
      <button className="button primary" type="button">Select local vault</button>
    </>
  );
}
