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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceJob {
    pub source_document_id: String,
    pub target_url: String,
}

pub fn initialize(root: &Path) -> SqlResult<LedgerPaths> {
    fs::create_dir_all(root).map_err(|_| rusqlite::Error::InvalidPath(root.to_path_buf()))?;
    let database = root.join(".researchledger.db");
    let connection = Connection::open(&database)?;
    connection.execute_batch(include_str!("../migrations/001_initial.sql"))?;
    for (name, definition) in [
        ("embedding_version", "TEXT NOT NULL DEFAULT 'v1'"),
        ("input_hash", "TEXT NOT NULL DEFAULT ''"),
    ] {
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('chunk_embeddings') WHERE name = ?1)",
            params![name],
            |row| row.get(0),
        )?;
        if !exists {
            connection.execute(
                &format!("ALTER TABLE chunk_embeddings ADD COLUMN {name} {definition}"),
                [],
            )?;
        }
    }
    for (name, definition) in [
        ("evidence_quote", "TEXT NOT NULL DEFAULT ''"),
        ("span_start", "INTEGER NOT NULL DEFAULT 0"),
        ("span_end", "INTEGER NOT NULL DEFAULT 0"),
    ] {
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('claims') WHERE name = ?1)",
            params![name],
            |row| row.get(0),
        )?;
        if !exists {
            connection.execute(
                &format!("ALTER TABLE claims ADD COLUMN {name} {definition}"),
                [],
            )?;
        }
    }
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

fn provenance_quote(content: &str) -> String {
    let mut lines = content.lines();
    if lines.next() == Some("---") {
        for line in lines.by_ref() {
            if line == "---" {
                break;
            }
        }
    }
    lines
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .trim_start_matches('#')
        .trim()
        .chars()
        .take(280)
        .collect()
}

fn write_chunks(
    transaction: &rusqlite::Transaction<'_>,
    document_id: &str,
    chunks: &[(Option<String>, String)],
) -> SqlResult<()> {
    for (ordinal, (heading, text)) in chunks.iter().enumerate() {
        transaction.execute(
            "INSERT INTO chunks(document_id, ordinal, heading_path, text) VALUES(?1, ?2, ?3, ?4)",
            params![document_id, ordinal as i64, heading, text],
        )?;
        let rowid = transaction.last_insert_rowid();
        transaction.execute(
            "INSERT INTO chunk_fts(rowid, text, heading_path) VALUES(?1, ?2, ?3)",
            params![rowid, text, heading],
        )?;
    }
    Ok(())
}

fn chunk_index_matches(
    connection: &Connection,
    document_id: &str,
    expected: &[(Option<String>, String)],
) -> SqlResult<bool> {
    let mut statement = connection
        .prepare("SELECT heading_path, text FROM chunks WHERE document_id = ?1 ORDER BY ordinal")?;
    let rows = statement.query_map(params![document_id], |row| {
        Ok((row.get::<_, Option<String>>(0)?, row.get::<_, String>(1)?))
    })?;
    let actual = rows.collect::<SqlResult<Vec<_>>>()?;
    Ok(actual == expected)
}

pub fn upsert_document(
    connection: &mut Connection,
    root: &Path,
    document: &SourceDocument,
) -> SqlResult<UpsertResult> {
    crate::okf::validate_concept(&document.content).map_err(|_| {
        rusqlite::Error::InvalidParameterName("document is not an OKF concept".into())
    })?;
    let hash = format!("{:x}", Sha256::digest(document.content.as_bytes()));
    let previous: Option<String> = connection
        .query_row(
            "SELECT content_hash FROM documents WHERE id = ?1",
            params![document.id],
            |row| row.get(0),
        )
        .optional()?;
    if previous.as_deref() == Some(hash.as_str()) {
        let expected_chunks = crate::chunking::split_document(&document.content);
        if !chunk_index_matches(connection, &document.id, &expected_chunks)? {
            let transaction = connection.transaction()?;
            transaction.execute(
                "DELETE FROM chunk_fts WHERE rowid IN (SELECT id FROM chunks WHERE document_id = ?1)",
                params![document.id],
            )?;
            transaction.execute(
                "DELETE FROM chunk_embeddings WHERE chunk_id IN (SELECT id FROM chunks WHERE document_id = ?1)",
                params![document.id],
            )?;
            transaction.execute(
                "DELETE FROM chunks WHERE document_id = ?1",
                params![document.id],
            )?;
            write_chunks(&transaction, &document.id, &expected_chunks)?;
            transaction.commit()?;
        }
        if let Some(source_uri) = document.source_uri.as_deref() {
            let quote = provenance_quote(&document.content);
            connection.execute(
                "DELETE FROM provenance WHERE document_id = ?1",
                params![document.id],
            )?;
            connection.execute(
                "INSERT INTO provenance(document_id, source_uri, locator, quote, captured_at) VALUES(?1, ?2, ?3, ?4, ?5)",
                params![document.id, source_uri, document.relative_path, quote, document.captured_at],
            )?;
        }
        connection.execute(
            "DELETE FROM claims WHERE document_id = ?1",
            params![document.id],
        )?;
        for (ordinal, evidence) in crate::distill::extract_claim_evidence(&document.content)
            .into_iter()
            .enumerate()
        {
            connection.execute(
                "INSERT INTO claims(document_id, ordinal, claim, source_uri, citation_id, evidence_quote, span_start, span_end, created_at) VALUES(?1, ?2, ?3, ?4, '1', ?5, ?6, ?7, ?8)",
                params![document.id, ordinal as i64, evidence.claim, document.source_uri, evidence.quote, evidence.start, evidence.end, document.captured_at],
            )?;
        }
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
        "DELETE FROM chunk_embeddings WHERE chunk_id IN (SELECT id FROM chunks WHERE document_id = ?1)",
        params![document.id],
    )?;
    tx.execute(
        "DELETE FROM chunks WHERE document_id = ?1",
        params![document.id],
    )?;
    tx.execute("INSERT OR IGNORE INTO document_versions(document_id, content_hash, raw_content, captured_at) VALUES(?1, ?2, ?3, ?4)",
        params![document.id, hash, document.content, document.captured_at])?;
    for (ordinal, (heading, text)) in crate::chunking::split_document(&document.content)
        .into_iter()
        .enumerate()
    {
        tx.execute(
            "INSERT INTO chunks(document_id, ordinal, heading_path, text) VALUES(?1, ?2, ?3, ?4)",
            params![document.id, ordinal as i64, heading, text],
        )?;
        let rowid = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO chunk_fts(rowid, text, heading_path) VALUES(?1, ?2, ?3)",
            params![rowid, text, heading],
        )?;
    }
    tx.execute(
        "DELETE FROM document_links WHERE source_document_id = ?1",
        params![document.id],
    )?;
    // Record discovered links; only explicit active-consent logic may queue reference fetches.
    for url in crate::enrichment::extract_urls(&document.content) {
        tx.execute(
            "INSERT OR IGNORE INTO document_links (source_document_id, target_url, discovered_at) VALUES (?1, ?2, ?3)",
            params![document.id, crate::enrichment::canonical_url(&url), document.captured_at],
        )?;
    }
    tx.execute(
        "DELETE FROM provenance WHERE document_id = ?1",
        params![document.id],
    )?;
    tx.execute(
        "DELETE FROM claims WHERE document_id = ?1",
        params![document.id],
    )?;
    if let Some(source_uri) = document.source_uri.as_deref() {
        let quote = provenance_quote(&document.content);
        tx.execute(
            "INSERT INTO provenance(document_id, source_uri, locator, quote, captured_at) VALUES(?1, ?2, ?3, ?4, ?5)",
            params![document.id, source_uri, document.relative_path, quote, document.captured_at],
        )?;
    }
    for (ordinal, evidence) in crate::distill::extract_claim_evidence(&document.content)
        .into_iter()
        .enumerate()
    {
        tx.execute(
            "INSERT INTO claims(document_id, ordinal, claim, source_uri, citation_id, evidence_quote, span_start, span_end, created_at) VALUES(?1, ?2, ?3, ?4, '1', ?5, ?6, ?7, ?8)",
            params![document.id, ordinal as i64, evidence.claim, document.source_uri, evidence.quote, evidence.start, evidence.end, document.captured_at],
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
    let relative = Path::new(&document.relative_path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(rusqlite::Error::InvalidPath(relative.to_path_buf()));
    }
    let path = root.join(relative);
    if !path.starts_with(root) {
        return Err(rusqlite::Error::InvalidPath(path));
    }
    document.content = fs::read_to_string(path).unwrap_or_default();
    Ok(Some(document))
}

pub fn pending_enrichment_ids(connection: &Connection, limit: u32) -> SqlResult<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT document_id FROM enrichment_jobs WHERE status='pending' ORDER BY created_at LIMIT ?1",
    )?;
    let rows = statement.query_map(params![limit], |row| row.get(0))?;
    rows.collect()
}

/// Return a deterministic, bounded batch of explicitly queued reference jobs.
pub fn pending_reference_jobs(connection: &Connection, limit: u32) -> SqlResult<Vec<ReferenceJob>> {
    pending_reference_jobs_at(connection, limit, &chrono::Utc::now().to_rfc3339())
}

pub fn pending_reference_jobs_at(
    connection: &Connection,
    limit: u32,
    now: &str,
) -> SqlResult<Vec<ReferenceJob>> {
    let mut statement = connection.prepare(
        "SELECT source_document_id, target_url FROM reference_fetches
         WHERE status = 'pending' ORDER BY id LIMIT ?1",
    )?;
    let rows = statement.query_map(params![limit], |row| {
        Ok(ReferenceJob {
            source_document_id: row.get(0)?,
            target_url: row.get(1)?,
        })
    })?;
    let mut jobs = Vec::new();
    for row in rows {
        let job = row?;
        if crate::consent::ConsentRegistry::new(connection)
            .decide(&job.target_url, now)?
            .allowed
        {
            jobs.push(job);
        }
    }
    Ok(jobs)
}

pub fn queue_reference_fetch(
    connection: &Connection,
    source_document_id: &str,
    target_url: &str,
    now: &str,
) -> SqlResult<bool> {
    let decision = crate::consent::ConsentRegistry::new(connection).decide(target_url, now)?;
    if !decision.allowed {
        return Ok(false);
    }
    let target_url = crate::consent::canonical_scope(target_url);
    connection.execute(
        "INSERT INTO reference_fetches (source_document_id, target_url, status)
         VALUES (?1, ?2, 'pending')
         ON CONFLICT(source_document_id, target_url) DO UPDATE SET status='pending', error=NULL",
        params![source_document_id, target_url],
    )?;
    Ok(true)
}

pub fn mark_reference_fetch_started(connection: &Connection, job: &ReferenceJob) -> SqlResult<()> {
    connection.execute(
        "UPDATE reference_fetches SET status = 'running', error = NULL
         WHERE source_document_id = ?1 AND target_url = ?2",
        params![job.source_document_id, job.target_url],
    )?;
    Ok(())
}

pub fn record_reference_fetch(
    connection: &Connection,
    job: &ReferenceJob,
    status: &str,
    artifact_path: Option<&str>,
    content_type: Option<&str>,
    http_status: Option<u16>,
    byte_count: Option<usize>,
    content_hash: Option<&str>,
    fetched_at: Option<&str>,
    error: Option<&str>,
) -> SqlResult<()> {
    connection.execute(
        "UPDATE reference_fetches SET status = ?3, artifact_path = ?4, content_type = ?5,
         http_status = ?6, byte_count = ?7, content_hash = ?8, fetched_at = ?9, error = ?10
         WHERE source_document_id = ?1 AND target_url = ?2",
        params![
            job.source_document_id,
            job.target_url,
            status,
            artifact_path,
            content_type,
            http_status.map(i64::from),
            byte_count.map(|value| value as i64),
            content_hash,
            fetched_at,
            error,
        ],
    )?;
    Ok(())
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
            if entry.file_type()?.is_symlink() {
                continue;
            }
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
                let reserved = path
                    .file_name()
                    .is_some_and(|name| name == "index.md" || name == "log.md");
                if !reserved {
                    let content = fs::read_to_string(&path)?;
                    crate::okf::validate_concept(&content).map_err(|error| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, error)
                    })?;
                }
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
            let entry = entry?;
            if entry.file_type()?.is_symlink() {
                continue;
            }
            let path = entry.path();
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
