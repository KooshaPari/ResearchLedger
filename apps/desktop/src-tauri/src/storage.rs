use rusqlite::{params, Connection, Result as SqlResult};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LedgerPaths {
    pub vault: PathBuf,
    pub database: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SourceDocument {
    pub id: String,
    pub relative_path: String,
    pub title: String,
    pub source_kind: String,
    pub source_uri: Option<String>,
    pub content: String,
    pub captured_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpsertResult {
    Created,
    Updated,
    Unchanged,
}

pub fn initialize(root: &Path) -> SqlResult<LedgerPaths> {
    fs::create_dir_all(root).map_err(|_| rusqlite::Error::InvalidPath(root.to_path_buf()))?;
    let database = root.join(".researchledger.db");
    let connection = Connection::open(&database)?;
    connection.execute_batch(include_str!("../migrations/001_initial.sql"))?;
    Ok(LedgerPaths {
        vault: root.to_path_buf(),
        database,
    })
}

pub fn open(paths: &LedgerPaths) -> SqlResult<Connection> {
    let connection = Connection::open(&paths.database)?;
    connection.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(connection)
}

pub fn write_markdown_atomic(
    root: &Path,
    relative_path: &str,
    content: &str,
) -> std::io::Result<PathBuf> {
    let path = root.join(relative_path);
    if !path.starts_with(root) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "path escapes vault",
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, content)?;
    fs::rename(&temp, &path)?;
    Ok(path)
}

pub fn upsert_document(
    connection: &mut Connection,
    root: &Path,
    document: &SourceDocument,
) -> SqlResult<UpsertResult> {
    let hash = format!("{:x}", Sha256::digest(document.content.as_bytes()));
    let previous: Option<String> = connection
        .query_row(
            "SELECT content_hash FROM documents WHERE id = ?1",
            params![document.id],
            |row| row.get(0),
        )
        .optional()?;
    if previous.as_deref() == Some(hash.as_str()) {
        return Ok(UpsertResult::Unchanged);
    }

    let tx = connection.transaction()?;
    tx.execute(
        "INSERT INTO documents(id, canonical_path, title, source_kind, source_uri, content_hash, captured_at, updated_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
         ON CONFLICT(id) DO UPDATE SET canonical_path=excluded.canonical_path, title=excluded.title,
         source_kind=excluded.source_kind, source_uri=excluded.source_uri, content_hash=excluded.content_hash,
         captured_at=excluded.captured_at, updated_at=excluded.updated_at",
        params![document.id, document.relative_path, document.title, document.source_kind,
            document.source_uri, hash, document.captured_at],
    )?;
    tx.execute("INSERT OR IGNORE INTO document_versions(document_id, content_hash, raw_content, captured_at) VALUES(?1, ?2, ?3, ?4)",
        params![document.id, hash, document.content, document.captured_at])?;
    write_markdown_atomic(root, &document.relative_path, &document.content)
        .map_err(|_| rusqlite::Error::InvalidPath(root.to_path_buf()))?;
    tx.commit()?;
    Ok(if previous.is_some() {
        UpsertResult::Updated
    } else {
        UpsertResult::Created
    })
}

pub fn document_count(connection: &Connection) -> SqlResult<u64> {
    connection.query_row("SELECT COUNT(*) FROM documents", [], |row| {
        row.get::<_, u64>(0)
    })
}

trait OptionalRow<T> {
    fn optional(self) -> SqlResult<Option<T>>;
}
impl<T> OptionalRow<T> for SqlResult<T> {
    fn optional(self) -> SqlResult<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error),
        }
    }
}
