PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS schema_version (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS documents (
  id TEXT PRIMARY KEY,
  canonical_path TEXT NOT NULL UNIQUE,
  title TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  source_uri TEXT,
  content_hash TEXT NOT NULL,
  captured_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS document_versions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  document_id TEXT NOT NULL REFERENCES documents(id),
  content_hash TEXT NOT NULL,
  raw_content TEXT NOT NULL,
  captured_at TEXT NOT NULL,
  UNIQUE(document_id, content_hash)
);

CREATE TABLE IF NOT EXISTS chunks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  document_id TEXT NOT NULL REFERENCES documents(id),
  ordinal INTEGER NOT NULL,
  heading_path TEXT,
  text TEXT NOT NULL,
  UNIQUE(document_id, ordinal)
);

CREATE VIRTUAL TABLE IF NOT EXISTS chunk_fts USING fts5(
  text,
  heading_path,
  content='chunks',
  content_rowid='id'
);

CREATE TABLE IF NOT EXISTS provenance (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  document_id TEXT NOT NULL REFERENCES documents(id),
  source_uri TEXT,
  locator TEXT,
  quote TEXT,
  captured_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS import_jobs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_kind TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  error TEXT
);

INSERT OR IGNORE INTO schema_version(version) VALUES (1);

CREATE TABLE IF NOT EXISTS document_links (
  source_document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  target_url TEXT NOT NULL,
  relation TEXT NOT NULL DEFAULT 'mentioned',
  discovered_at TEXT NOT NULL,
  PRIMARY KEY (source_document_id, target_url)
);

CREATE INDEX IF NOT EXISTS idx_document_links_target ON document_links(target_url);

CREATE TABLE IF NOT EXISTS enrichment_jobs (
  document_id TEXT PRIMARY KEY REFERENCES documents(id) ON DELETE CASCADE,
  strategy TEXT NOT NULL,
  status TEXT NOT NULL,
  input_hash TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  error TEXT
);
