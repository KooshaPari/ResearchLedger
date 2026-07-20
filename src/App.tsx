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
            <p className="eyebrow">NO VAULT SELECTED</p>
            <h3>Your sources stay yours.</h3>
            <p className="muted">
              Choose a local Markdown vault to begin importing and indexing research.
            </p>
            <button className="button primary" type="button">Select local vault</button>
          </div>
        </section>
      </section>
    </main>
  );
}
