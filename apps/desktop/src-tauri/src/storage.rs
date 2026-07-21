use rusqlite::{params, Connection, Result as SqlResult};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LedgerPaths {
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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub document_id: String,
    pub title: String,
    pub source_uri: Option<String>,
    pub snippet: String,
}

pub fn initialize(root: &Path) -> SqlResult<LedgerPaths> {
    fs::create_dir_all(root).map_err(|_| rusqlite::Error::InvalidPath(root.to_path_buf()))?;
    let database = root.join(".researchledger.db");
    let connection = Connection::open(&database)?;
    connection.execute_batch(include_str!("../migrations/001_initial.sql"))?;
    Ok(LedgerPaths { database })
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
    if Path::new(relative_path).is_absolute()
        || Path::new(relative_path)
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "path escapes vault",
        ));
    }
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
    tx.execute(
        "DELETE FROM chunk_fts WHERE rowid IN (SELECT id FROM chunks WHERE document_id = ?1)",
        params![document.id],
    )?;
    tx.execute(
        "DELETE FROM chunks WHERE document_id = ?1",
        params![document.id],
    )?;
    tx.execute("INSERT OR IGNORE INTO document_versions(document_id, content_hash, raw_content, captured_at) VALUES(?1, ?2, ?3, ?4)",
        params![document.id, hash, document.content, document.captured_at])?;
    tx.execute(
        "INSERT INTO chunks(document_id, ordinal, heading_path, text) VALUES(?1, 0, NULL, ?2)",
        params![document.id, document.content],
    )?;
    let rowid = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO chunk_fts(rowid, text, heading_path) VALUES(?1, ?2, NULL)",
        params![rowid, document.content],
    )?;
    tx.execute(
        "DELETE FROM document_links WHERE source_document_id = ?1",
        params![document.id],
    )?;
    for url in crate::enrichment::extract_urls(&document.content) {
        tx.execute(
            "INSERT OR IGNORE INTO document_links (source_document_id, target_url, discovered_at) VALUES (?1, ?2, ?3)",
            params![document.id, crate::enrichment::canonical_url(&url), document.captured_at],
        )?;
    }
    if document.source_kind != "distillation" {
        tx.execute(
            "INSERT INTO enrichment_jobs (document_id, strategy, status, input_hash, created_at, updated_at) VALUES (?1, 'deterministic-v1', 'pending', ?2, ?3, ?3) ON CONFLICT(document_id) DO UPDATE SET input_hash=excluded.input_hash, updated_at=excluded.updated_at, status='pending', error=NULL",
            params![document.id, hash, document.captured_at],
        )?;
    }
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

pub fn load_document(
    connection: &Connection,
    root: &Path,
    id: &str,
) -> SqlResult<Option<SourceDocument>> {
    let result = connection.query_row(
        "SELECT id, canonical_path, title, source_kind, source_uri, captured_at FROM documents WHERE id = ?1",
        params![id],
        |row| Ok(SourceDocument { id: row.get(0)?, relative_path: row.get(1)?, title: row.get(2)?, source_kind: row.get(3)?, source_uri: row.get(4)?, content: String::new(), captured_at: row.get(5)? }),
    ).optional()?;
    let Some(mut document) = result else {
        return Ok(None);
    };
    document.content = fs::read_to_string(root.join(&document.relative_path)).unwrap_or_default();
    Ok(Some(document))
}

pub fn pending_enrichment_ids(connection: &Connection, limit: u32) -> SqlResult<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT document_id FROM enrichment_jobs WHERE status='pending' ORDER BY created_at LIMIT ?1",
    )?;
    let rows = statement.query_map(params![limit], |row| row.get(0))?;
    rows.collect()
}

pub fn search(connection: &Connection, query: &str, limit: u32) -> SqlResult<Vec<SearchResult>> {
    let mut statement = connection.prepare(
        "SELECT d.id, d.title, d.source_uri, snippet(chunk_fts, 0, '<mark>', '</mark>', '…', 24)
         FROM chunk_fts JOIN chunks c ON c.id = chunk_fts.rowid JOIN documents d ON d.id = c.document_id
         WHERE chunk_fts MATCH ?1 ORDER BY rank LIMIT ?2",
    )?;
    let rows = statement.query_map(params![query, limit], |row| {
        Ok(SearchResult {
            document_id: row.get(0)?,
            title: row.get(1)?,
            source_uri: row.get(2)?,
            snippet: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn search_vectors(
    connection: &Connection,
    query: &[f32],
    limit: u32,
) -> SqlResult<Vec<(SearchResult, f32)>> {
    let mut statement = connection.prepare(
        "SELECT d.id, d.title, d.source_uri, snippet(chunk_fts, 0, '<mark>', '</mark>', '…', 24), e.vector_json FROM chunk_embeddings e JOIN chunks c ON c.id=e.chunk_id JOIN documents d ON d.id=c.document_id JOIN chunk_fts ON chunk_fts.rowid=c.id",
    )?;
    let mut scored = statement
        .query_map([], |row| {
            let vector: Vec<f32> =
                serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default();
            Ok((
                SearchResult {
                    document_id: row.get(0)?,
                    title: row.get(1)?,
                    source_uri: row.get(2)?,
                    snippet: row.get(3)?,
                },
                vector,
            ))
        })?
        .collect::<SqlResult<Vec<_>>>()?
        .into_iter()
        .map(|(result, vector)| (result, cosine_similarity(query, &vector)))
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.1.total_cmp(&left.1));
    scored.truncate(limit as usize);
    Ok(scored)
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    let dot: f32 = left.iter().zip(right).map(|(a, b)| a * b).sum();
    let left_norm = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

pub fn export_markdown(vault: &Path, destination: &Path) -> std::io::Result<u64> {
    fn copy_tree(source: &Path, destination: &Path, count: &mut u64) -> std::io::Result<()> {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            let target = destination.join(entry.file_name());
            if path
                .file_name()
                .is_some_and(|name| name == ".researchledger.db")
            {
                continue;
            }
            if path.is_dir() {
                copy_tree(&path, &target, count)?;
            } else if path.extension().is_some_and(|ext| ext == "md") {
                fs::copy(&path, &target)?;
                *count += 1;
            }
        }
        Ok(())
    }
    let mut count = 0;
    copy_tree(vault, destination, &mut count)?;
    let mut index = String::from("# ResearchLedger Knowledge Bundle\n\n");
    fn append_index(root: &Path, current: &Path, output: &mut String) -> std::io::Result<()> {
        for entry in fs::read_dir(current)? {
            let path = entry?.path();
            if path.is_dir() {
                append_index(root, &path, output)?;
            } else if path.extension().is_some_and(|extension| extension == "md")
                && path.file_name().is_some_and(|name| name != "index.md")
            {
                let relative = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                output.push_str(&format!(
                    "- [{}]({})\n",
                    relative.trim_end_matches(".md"),
                    relative
                ));
            }
        }
        Ok(())
    }
    append_index(destination, destination, &mut index)?;
    fs::write(destination.join("index.md"), index)?;
    fs::write(
        destination.join(".researchledger-export"),
        "format: markdown-vault\nversion: 1\n",
    )?;
    Ok(count)
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
