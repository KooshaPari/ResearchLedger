use serde::Serialize;
mod distill;
mod embeddings;
mod enrichment;
mod github;
mod linkedin;
mod provider_html;
mod rag;
mod reddit;
mod safe_paths;
mod storage;
mod x;

mod commands {
    include!("commands.rs");

    use super::{
        distill, embeddings::OllamaEmbedder, github, github::GithubClient, linkedin, rag, reddit,
        storage, x, VaultStatus,
    };
    use serde::Serialize;
    use tauri::Manager;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ImportSummary {
        pub created: u64,
        pub updated: u64,
        pub unchanged: u64,
        pub failed: u64,
    }

    /// Render a third-party-provider post (LinkedIn, Reddit, X) into a `SourceDocument`.
    /// `id_prefix`/`type_label`/`relative_dir` describe the schema, while `url`, `title`,
    /// and `body` come from the connector-specific parser. The captured-at timestamp is
    /// **not** embedded in the content so re-imports of identical input hash
    /// identically and surface as `Unchanged` rather than churning to `Updated`.
    pub fn render_provider_document(
        timestamp: &str,
        id_prefix: &str,
        source_kind: &str,
        type_label: &str,
        description: &str,
        tags: &[&str],
        relative_dir: &str,
        url: &str,
        title: &str,
        body: &str,
    ) -> storage::SourceDocument {
        let id_segment = url_id_segment(url);
        let id = format!("{id_prefix}:{id_segment}");
        let tags_text = format!("[{}]", tags.join(", "));
        // NB: do NOT include `timestamp` inside the content body — including it would
        // change the SHA-256 hash on every re-import and break downsync detection.
        let content = format!(
            "---\n\
             type: {type_label}\n\
             id: {id}\n\
             title: {title}\n\
             description: {description}\n\
             resource: {url}\n\
             tags: {tags_text}\n\
             source_kind: {source_kind}\n\
             source_uri: {url}\n\
             ---\n\
             \n\
             {body}\n\
             \n\
             # Citations\n\
             \n\
             [1] [{type_label}]({url})\n"
        );
        storage::SourceDocument {
            id: id.clone(),
            relative_path: format!("{relative_dir}/{id_segment}.md"),
            title: title.into(),
            source_kind: source_kind.into(),
            source_uri: Some(url.into()),
            content,
            captured_at: timestamp.into(),
        }
    }

    /// Derive a filesystem/document-id-safe slug from a provider URL. Strips an
    /// optional `https?://` scheme so the slug begins at the domain.
    pub fn url_id_segment(url: &str) -> String {
        let trimmed = url
            .trim()
            .split('?')
            .next()
            .unwrap_or(url)
            .trim_end_matches('/');
        let trimmed = trimmed
            .strip_prefix("https://")
            .or_else(|| trimmed.strip_prefix("http://"))
            .unwrap_or(trimmed);
        let mut slug = String::with_capacity(trimmed.len());
        for character in trimmed.chars() {
            if character.is_ascii_alphanumeric() {
                slug.push(character.to_ascii_lowercase());
            } else if matches!(character, '_' | '-' | '.') {
                slug.push(character);
            } else if character == '/' {
                if !slug.ends_with('-') {
                    slug.push('-');
                }
            }
        }
        let slug = slug.trim_matches('-').to_string();
        if slug.is_empty() {
            "post".into()
        } else {
            slug
        }
    }

    /// Validate that `user_path` is a safe, descendant of an acceptable root.
    /// Returns the canonicalised path. Used by import_* and capture_*
    /// commands to mitigate path-injection rules (Sonar `rust:S2089`).
    fn validated_user_path(
        user_path: &str,
        label: &str,
        vault_path: &str,
    ) -> Result<std::path::PathBuf, String> {
        let vault = std::path::PathBuf::from(vault_path);
        let temp = std::env::temp_dir();
        let home = std::env::var("HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| temp.clone());
        super::safe_paths::ensure_within_acceptable_roots(
            user_path,
            label,
            &[vault, temp, home],
        )
    }

    #[tauri::command]
    pub fn get_vault_status(vault_path: Option<String>) -> Result<VaultStatus, String> {
        let Some(path) = vault_path.filter(|value| !value.trim().is_empty()) else {
            return Ok(VaultStatus {
                selected: false,
                path: None,
                document_count: 0,
            });
        };
        let root = std::path::PathBuf::from(&path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let document_count =
            storage::document_count(&connection).map_err(|error| error.to_string())?;
        Ok(VaultStatus {
            selected: true,
            path: Some(path),
            document_count,
        })
    }

    #[tauri::command]
    pub async fn import_github(vault_path: String, token: String) -> Result<ImportSummary, String> {
        if token.trim().is_empty() {
            return Err("GitHub token is required".into());
        }
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let client = GithubClient::new(token).map_err(|error| error.to_string())?;
        let repositories = client
            .list_starred()
            .await
            .map_err(|error| error.to_string())?;
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for repository in repositories {
            let readme = match client
                .read_readme(&repository.owner.login, &repository.name)
                .await
            {
                Ok(value) => value.unwrap_or_else(|| "README unavailable.\n".into()),
                Err(_) => {
                    summary.failed += 1;
                    continue;
                }
            };
            let description = repository
                .description
                .as_deref()
                .unwrap_or("No description provided.")
                .replace('"', "'");
            let content = format!("---\ntype: GitHub Repository\nid: github:{}\ntitle: {}\ndescription: \"{}\"\nresource: {}\ntags: [github, repository]\ntimestamp: {}\nsource_kind: github\nsource_uri: {}\n---\n\n# {}\n\n{}\n\n## Repository\n\n{}\n\n# Citations\n\n[1] [{}]({})\n",
                repository.full_name, repository.name, description, repository.html_url, chrono::Utc::now().to_rfc3339(), repository.html_url, repository.name,
                description, readme, repository.name, repository.html_url);
            let document = storage::SourceDocument {
                id: format!("github:{}", repository.full_name),
                relative_path: format!(
                    "sources/github/{}--{}.md",
                    repository.owner.login, repository.name
                ),
                title: repository.name,
                source_kind: "github".into(),
                source_uri: Some(repository.html_url),
                content,
                captured_at: chrono::Utc::now().to_rfc3339(),
            };
            match storage::upsert_document(&mut connection, &root, &document)
                .map_err(|error| error.to_string())?
            {
                storage::UpsertResult::Created => summary.created += 1,
                storage::UpsertResult::Updated => summary.updated += 1,
                storage::UpsertResult::Unchanged => summary.unchanged += 1,
            }
        }
        Ok(summary)
    }

    #[tauri::command]
    pub async fn github_device_start(
        client_id: String,
    ) -> Result<github::DeviceAuthorization, String> {
        GithubClient::new("")
            .map_err(|error| error.to_string())?
            .request_device_authorization(&client_id)
            .await
            .map_err(|error| error.to_string())
    }

    #[tauri::command]
    pub async fn github_device_poll(
        client_id: String,
        device_code: String,
        interval: u64,
        expires_in: u64,
    ) -> Result<String, String> {
        GithubClient::new("")
            .map_err(|error| error.to_string())?
            .poll_device_token(&client_id, &device_code, interval, expires_in)
            .await
            .map_err(|error| error.to_string())
    }

    #[tauri::command]
    pub fn import_linkedin_html(
        vault_path: String,
        html_path: String,
    ) -> Result<ImportSummary, String> {
        let path = validated_user_path(&html_path, "LinkedIn html", &vault_path)?;
        let html = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let posts = linkedin::parse_activity_html(&html);
        import_provider_posts(
            &vault_path,
            ProviderMetadata::linkedin(),
            posts.into_iter().map(|post| ProviderImportInput {
                url: post.url,
                title: extract_linkedin_title(&post.text),
                body: post.text,
            }),
        )
    }

    #[tauri::command]
    pub fn import_linkedin_capture(
        vault_path: String,
        capture_path: String,
    ) -> Result<ImportSummary, String> {
        let path = validated_user_path(&capture_path, "LinkedIn capture", &vault_path)?;
        let json = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let posts = linkedin::parse_capture_json(&json).map_err(|error| error.to_string())?;
        import_provider_posts(
            &vault_path,
            ProviderMetadata::linkedin(),
            posts.into_iter().map(|post| ProviderImportInput {
                url: post.url,
                title: extract_linkedin_title(&post.text),
                body: post.text,
            }),
        )
    }

    pub struct ProviderImportInput {
        pub url: String,
        pub title: String,
        pub body: String,
    }

    pub struct ProviderMetadata {
        pub id_prefix: &'static str,
        pub source_kind: &'static str,
        pub type_label: &'static str,
        pub description: &'static str,
        pub tags: &'static [&'static str],
        pub relative_dir: &'static str,
    }

    impl ProviderMetadata {
        pub const fn linkedin() -> Self {
            Self {
                id_prefix: "linkedin",
                source_kind: "linkedin",
                type_label: "LinkedIn Post",
                description: "Captured LinkedIn post",
                tags: &["linkedin", "captured"],
                relative_dir: "sources/linkedin",
            }
        }

        pub const fn reddit() -> Self {
            Self {
                id_prefix: "reddit",
                source_kind: "reddit",
                type_label: "Reddit Saved Post",
                description: "Captured Reddit saved post",
                tags: &["reddit", "saved", "captured"],
                relative_dir: "sources/reddit",
            }
        }

        pub const fn x() -> Self {
            Self {
                id_prefix: "x",
                source_kind: "x",
                type_label: "X Bookmark",
                description: "Captured X bookmark",
                tags: &["x", "bookmark", "captured"],
                relative_dir: "sources/x",
            }
        }
    }

    /// Persist a batch of provider posts into the vault using the shared renderer.
    pub fn import_provider_posts(
        vault_path: &str,
        metadata: ProviderMetadata,
        posts: impl IntoIterator<Item = ProviderImportInput>,
    ) -> Result<ImportSummary, String> {
        let inputs: Vec<ProviderImportInput> = posts.into_iter().collect();
        if inputs.is_empty() {
            return Ok(ImportSummary {
                created: 0,
                updated: 0,
                unchanged: 0,
                failed: 0,
            });
        }
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let timestamp = chrono::Utc::now().to_rfc3339();
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for input in inputs {
            let document = render_provider_document(
                &timestamp,
                metadata.id_prefix,
                metadata.source_kind,
                metadata.type_label,
                metadata.description,
                metadata.tags,
                metadata.relative_dir,
                &input.url,
                &input.title,
                &input.body,
            );
            match storage::upsert_document(&mut connection, &root, &document)
                .map_err(|error| error.to_string())?
            {
                storage::UpsertResult::Created => summary.created += 1,
                storage::UpsertResult::Updated => summary.updated += 1,
                storage::UpsertResult::Unchanged => summary.unchanged += 1,
            }
        }
        Ok(summary)
    }

    /// Pick a short LinkedIn title from the body text. LinkedIn posts don't expose a
    /// separate title in the activity feed; the first non-empty line is usually a
    /// reasonable stand-in. We fall back to a fixed placeholder if the body is empty.
    fn extract_linkedin_title(body: &str) -> String {
        body.lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(|line| line.chars().take(80).collect::<String>())
            .unwrap_or_else(|| "LinkedIn post".into())
    }

    /// Resolve the absolute path to a bundled capture script. Prefers the packaged
    /// Tauri resource directory and falls back to the development checkout.
    fn resolve_capture_script(
        app: &tauri::AppHandle,
        script_name: &str,
    ) -> Result<std::path::PathBuf, String> {
        let packaged = app
            .path()
            .resource_dir()
            .map_err(|error| error.to_string())?
            .join(format!("scripts/{script_name}"));
        if packaged.exists() {
            return Ok(packaged);
        }
        let local = std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(format!("scripts/{script_name}"));
        if local.exists() {
            return Ok(local);
        }
        Err(format!(
            "Capture script {script_name} was not found in resource or local scripts/."
        ))
    }

    /// Build a tokio Command for invoking a browser-capture script with the shared
    /// Playwright module + profile URL conventions.
    fn build_capture_command(
        app: &tauri::AppHandle,
        script: &std::path::Path,
        output: &std::path::Path,
    ) -> Result<tokio::process::Command, String> {
        let mut command = tokio::process::Command::new("node");
        command.arg(script).arg("--output").arg(output);
        if let Ok(resource_dir) = app.path().resource_dir() {
            let packaged_module = resource_dir.join("node_modules/playwright/index.mjs");
            if packaged_module.exists() {
                command.env("RESEARCHLEDGER_PLAYWRIGHT_MODULE", packaged_module);
            }
        }
        Ok(command)
    }

    #[tauri::command]
    pub async fn capture_linkedin_browser(
        app: tauri::AppHandle,
        vault_path: String,
        activity_url: Option<String>,
        profile_path: Option<String>,
    ) -> Result<ImportSummary, String> {
        let output = std::path::PathBuf::from(&vault_path)
            .join(".researchledger")
            .join("linkedin-capture.json");
        let script =
            resolve_capture_script(&app, "linkedin_capture.mjs")?;
        let mut command = build_capture_command(&app, &script, &output)?;
        if let Some(profile) = profile_path.filter(|value| !value.trim().is_empty()) {
            let safe = super::safe_paths::ensure_safe_command_arg(&profile, "profile path")?;
            command.arg("--profile").arg(safe);
        }
        if let Some(url) = activity_url {
            let safe = super::safe_paths::ensure_safe_provider_url(
                &url,
                &["www.linkedin.com", "linkedin.com"],
            )?;
            command.arg("--url").arg(safe);
        }
        let result = command.output().await.map_err(|error| error.to_string())?;
        if !result.status.success() {
            return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
        }
        import_linkedin_capture(vault_path, output.to_string_lossy().into_owned())
    }

    #[tauri::command]
    pub fn import_reddit_html(
        vault_path: String,
        html_path: String,
    ) -> Result<ImportSummary, String> {
        let path = validated_user_path(&html_path, "Reddit html", &vault_path)?;
        let html = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        import_provider_posts(
            &vault_path,
            ProviderMetadata::reddit(),
            reddit::parse_saved_html(&html).into_iter().map(|post| ProviderImportInput {
                title: if post.title.is_empty() {
                    post.subreddit
                        .as_deref()
                        .map(|subreddit| format!("Saved Reddit post in r/{subreddit}"))
                        .unwrap_or_else(|| "Saved Reddit post".into())
                } else {
                    post.title
                },
                url: post.url,
                body: post.text,
            }),
        )
    }

    #[tauri::command]
    pub fn import_reddit_capture(
        vault_path: String,
        capture_path: String,
    ) -> Result<ImportSummary, String> {
        let path = validated_user_path(&capture_path, "Reddit capture", &vault_path)?;
        let json = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let posts = reddit::parse_capture_json(&json).map_err(|error| error.to_string())?;
        import_provider_posts(
            &vault_path,
            ProviderMetadata::reddit(),
            posts.into_iter().map(|post| ProviderImportInput {
                title: if post.title.is_empty() {
                    post.subreddit
                        .as_deref()
                        .map(|subreddit| format!("Saved Reddit post in r/{subreddit}"))
                        .unwrap_or_else(|| "Saved Reddit post".into())
                } else {
                    post.title
                },
                url: post.url,
                body: post.text,
            }),
        )
    }

    #[tauri::command]
    pub async fn capture_reddit_browser(
        app: tauri::AppHandle,
        vault_path: String,
        saved_url: Option<String>,
        profile_path: Option<String>,
    ) -> Result<ImportSummary, String> {
        let output = std::path::PathBuf::from(&vault_path)
            .join(".researchledger")
            .join("reddit-capture.json");
        let script = resolve_capture_script(&app, "reddit_capture.mjs")?;
        let mut command = build_capture_command(&app, &script, &output)?;
        if let Some(profile) = profile_path.filter(|value| !value.trim().is_empty()) {
            let safe = super::safe_paths::ensure_safe_command_arg(&profile, "profile path")?;
            command.arg("--profile").arg(safe);
        }
        if let Some(url) = saved_url {
            let safe = super::safe_paths::ensure_safe_provider_url(
                &url,
                &["www.reddit.com", "reddit.com", "old.reddit.com"],
            )?;
            command.arg("--url").arg(safe);
        }
        let result = command.output().await.map_err(|error| error.to_string())?;
        if !result.status.success() {
            return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
        }
        import_reddit_capture(vault_path, output.to_string_lossy().into_owned())
    }

    #[tauri::command]
    pub fn import_x_html(
        vault_path: String,
        html_path: String,
    ) -> Result<ImportSummary, String> {
        let path = validated_user_path(&html_path, "X html", &vault_path)?;
        let html = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        import_provider_posts(
            &vault_path,
            ProviderMetadata::x(),
            x::parse_bookmarks_html(&html).into_iter().map(|post| ProviderImportInput {
                title: if post.author.is_empty() {
                    "Bookmarked X post".into()
                } else {
                    format!("@{author}", author = post.author)
                },
                url: post.url,
                body: post.text,
            }),
        )
    }

    #[tauri::command]
    pub fn import_x_capture(
        vault_path: String,
        capture_path: String,
    ) -> Result<ImportSummary, String> {
        let path = validated_user_path(&capture_path, "X capture", &vault_path)?;
        let json = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let posts = x::parse_capture_json(&json).map_err(|error| error.to_string())?;
        import_provider_posts(
            &vault_path,
            ProviderMetadata::x(),
            posts.into_iter().map(|post| ProviderImportInput {
                title: if post.author.is_empty() {
                    "Bookmarked X post".into()
                } else {
                    format!("@{author}", author = post.author)
                },
                url: post.url,
                body: post.text,
            }),
        )
    }

    #[tauri::command]
    pub async fn capture_x_browser(
        app: tauri::AppHandle,
        vault_path: String,
        saved_url: Option<String>,
        profile_path: Option<String>,
    ) -> Result<ImportSummary, String> {
        let output = std::path::PathBuf::from(&vault_path)
            .join(".researchledger")
            .join("x-capture.json");
        let script = resolve_capture_script(&app, "x_capture.mjs")?;
        let mut command = build_capture_command(&app, &script, &output)?;
        if let Some(profile) = profile_path.filter(|value| !value.trim().is_empty()) {
            let safe = super::safe_paths::ensure_safe_command_arg(&profile, "profile path")?;
            command.arg("--profile").arg(safe);
        }
        if let Some(url) = saved_url {
            let safe = super::safe_paths::ensure_safe_provider_url(
                &url,
                &["x.com", "twitter.com"],
            )?;
            command.arg("--url").arg(safe);
        }
        let result = command.output().await.map_err(|error| error.to_string())?;
        if !result.status.success() {
            return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
        }
        import_x_capture(vault_path, output.to_string_lossy().into_owned())
    }

    #[tauri::command]
    pub fn search_documents(
        vault_path: String,
        query: String,
        limit: Option<u32>,
    ) -> Result<Vec<storage::SearchResult>, String> {
        let paths = storage::initialize(std::path::Path::new(&vault_path))
            .map_err(|error| error.to_string())?;
        let connection = storage::open(&paths).map_err(|error| error.to_string())?;
        storage::search(&connection, &query, limit.unwrap_or(20).min(100))
            .map_err(|error| error.to_string())
    }

    #[tauri::command]
    pub fn export_obsidian(vault_path: String, destination: String) -> Result<u64, String> {
        storage::export_markdown(
            std::path::Path::new(&vault_path),
            std::path::Path::new(&destination),
        )
        .map_err(|error| error.to_string())
    }

    #[tauri::command]
    pub async fn retrieve_context(
        vault_path: String,
        query: String,
        limit: Option<u32>,
    ) -> Result<rag::RetrievalContext, String> {
        let paths = storage::initialize(std::path::Path::new(&vault_path))
            .map_err(|error| error.to_string())?;
        let connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let limit = limit.unwrap_or(8).min(50);
        let lexical =
            storage::search(&connection, &query, limit).map_err(|error| error.to_string())?;
        let vector_hits = match OllamaEmbedder::new("embeddinggemma")
            .embed_batch(std::slice::from_ref(&query))
            .await
        {
            Ok(vectors) => vectors
                .into_iter()
                .next()
                .map(|vector| {
                    storage::search_vectors(&connection, &vector, limit).unwrap_or_default()
                })
                .unwrap_or_default(),
            Err(_) => Vec::new(),
        };
        let vector_hits = vector_hits
            .into_iter()
            .map(|(result, score)| rag::VectorHit {
                document_id: result.document_id,
                score,
            })
            .collect();
        Ok(rag::build_context(
            &query,
            rag::fuse_ranked(lexical, vector_hits, limit as usize),
        ))
    }

    #[tauri::command]
    pub fn distill_document(
        vault_path: String,
        document_id: String,
    ) -> Result<ImportSummary, String> {
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let source = storage::load_document(&connection, &root, &document_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "source document not found".to_string())?;
        let distilled = storage::SourceDocument {
            id: format!("distillation:{document_id}"),
            relative_path: format!(
                "knowledge/{}--distilled.md",
                document_id.replace([':', '/'], "--")
            ),
            title: format!("{} — deterministic distillation", source.title),
            source_kind: "distillation".into(),
            source_uri: source.source_uri.clone(),
            content: distill::render_deterministic(&source),
            captured_at: chrono::Utc::now().to_rfc3339(),
        };
        let result = storage::upsert_document(&mut connection, &root, &distilled)
            .map_err(|error| error.to_string())?;
        connection.execute(
            "UPDATE enrichment_jobs SET status='completed', updated_at=?2, error=NULL WHERE document_id=?1",
            rusqlite::params![document_id, distilled.captured_at],
        ).map_err(|error| error.to_string())?;
        Ok(ImportSummary {
            created: u64::from(result == storage::UpsertResult::Created),
            updated: u64::from(result == storage::UpsertResult::Updated),
            unchanged: u64::from(result == storage::UpsertResult::Unchanged),
            failed: 0,
        })
    }

    #[tauri::command]
    pub fn process_pending_enrichment(
        vault_path: String,
        limit: Option<u32>,
    ) -> Result<ImportSummary, String> {
        let root = std::path::PathBuf::from(&vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let ids = storage::pending_enrichment_ids(&connection, limit.unwrap_or(25).min(100))
            .map_err(|error| error.to_string())?;
        drop(connection);
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for id in ids {
            match distill_document(vault_path.clone(), id) {
                Ok(result) => {
                    summary.created += result.created;
                    summary.updated += result.updated;
                    summary.unchanged += result.unchanged;
                }
                Err(_) => summary.failed += 1,
            }
        }
        Ok(summary)
    }

    #[tauri::command]
    pub async fn embed_document(
        vault_path: String,
        document_id: String,
        model: Option<String>,
    ) -> Result<usize, String> {
        let root = std::path::PathBuf::from(&vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let document = storage::load_document(&connection, &root, &document_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "source document not found".to_string())?;
        let model = model.unwrap_or_else(|| "embeddinggemma".into());
        let vectors = OllamaEmbedder::new(model.clone())
            .embed_batch(&[document.content])
            .await?;
        let Some(vector) = vectors.into_iter().next() else {
            return Err("embedding service returned no vector".into());
        };
        let chunk_id: i64 = connection
            .query_row(
                "SELECT id FROM chunks WHERE document_id=?1 ORDER BY ordinal LIMIT 1",
                rusqlite::params![document_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        connection.execute("INSERT INTO chunk_embeddings (chunk_id, model, dimensions, vector_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(chunk_id) DO UPDATE SET model=excluded.model, dimensions=excluded.dimensions, vector_json=excluded.vector_json, created_at=excluded.created_at", rusqlite::params![chunk_id, model, vector.len(), serde_json::to_string(&vector).map_err(|error| error.to_string())?, chrono::Utc::now().to_rfc3339()]).map_err(|error| error.to_string())?;
        Ok(vector.len())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    pub selected: bool,
    pub path: Option<String>,
    pub document_count: u64,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_vault_status,
            commands::import_github,
            commands::github_device_start,
            commands::github_device_poll,
            commands::import_linkedin_html,
            commands::import_linkedin_capture,
            commands::capture_linkedin_browser,
            commands::import_reddit_html,
            commands::import_reddit_capture,
            commands::capture_reddit_browser,
            commands::import_x_html,
            commands::import_x_capture,
            commands::capture_x_browser,
            commands::search_documents,
            commands::list_document_summaries,
            commands::list_collections,
            commands::list_document_links,
            commands::export_obsidian,
            commands::retrieve_context,
            commands::distill_document,
            commands::process_pending_enrichment,
            commands::embed_document
        ])
        .run(tauri::generate_context!())
        .expect("error while running ResearchLedger");
}

#[cfg(test)]
mod tests {
    use super::storage::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "researchledger-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn initializes_and_upserts_idempotently() {
        let root = temp_root();
        let paths = initialize(&root).unwrap();
        let mut db = open(&paths).unwrap();
        let document = SourceDocument {
            id: "github:octo/hello".into(),
            relative_path: "sources/github/octo--hello.md".into(),
            title: "hello".into(),
            source_kind: "github".into(),
            source_uri: Some("https://github.com/octo/hello".into()),
            content: "# hello\n".into(),
            captured_at: "2026-07-20T00:00:00Z".into(),
        };
        assert_eq!(
            upsert_document(&mut db, &root, &document).unwrap(),
            UpsertResult::Created
        );
        assert_eq!(
            upsert_document(&mut db, &root, &document).unwrap(),
            UpsertResult::Unchanged
        );
        assert_eq!(document_count(&db).unwrap(), 1);
        let results = search(&db, "hello", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, "github:octo/hello");
        assert!(root.join("sources/github/octo--hello.md").exists());
        let export = temp_root();
        assert_eq!(export_markdown(&root, &export).unwrap(), 1);
        assert!(export.join("sources/github/octo--hello.md").exists());
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(export);
    }

    #[test]
    fn markdown_writer_rejects_paths_outside_vault() {
        let root = temp_root();
        std::fs::create_dir_all(&root).unwrap();
        let outside = root.parent().unwrap().join("outside.md");
        let _ = std::fs::remove_file(&outside);
        let result = write_markdown_atomic(&root, "../outside.md", "blocked");
        assert_eq!(
            result.unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert!(!outside.exists());
        let _ = std::fs::remove_dir_all(root);
    }
}

#[cfg(test)]
mod provider_tests {
    use crate::commands::{import_provider_posts, render_provider_document, url_id_segment, ProviderImportInput, ProviderMetadata};

    #[test]
    fn url_id_segment_lowercases_and_slugifies_path() {
        assert_eq!(
            url_id_segment("https://www.Reddit.com/r/rust/comments/ABC/why/"),
            "www.reddit.com-r-rust-comments-abc-why"
        );
        assert_eq!(
            url_id_segment("https://x.com/user/status/1234567890"),
            "x.com-user-status-1234567890"
        );
        assert_eq!(url_id_segment(""), "post");
    }

    #[test]
    fn render_provider_document_emits_okf_frontmatter_and_citations() {
        let document = render_provider_document(
            "2026-07-23T00:00:00Z",
            "reddit",
            "reddit",
            "Reddit Saved Post",
            "Captured Reddit saved post",
            &["reddit", "saved"],
            "sources/reddit",
            "https://www.reddit.com/r/rust/comments/abc/hi/",
            "Saved Reddit post in r/rust",
            "Body text with substance.",
        );
        assert!(document.content.starts_with("---\ntype: Reddit Saved Post\n"));
        assert!(document.content.contains("\nid: reddit:www.reddit.com-r-rust-comments-abc-hi\n"));
        assert!(document.content.contains("\ntags: [reddit, saved]\n"));
        assert!(document.content.contains("\n# Citations\n"));
        assert!(document.content.contains("[1] [Reddit Saved Post]"));
        assert!(document.relative_path.starts_with("sources/reddit/"));
    }

    #[test]
    fn import_provider_posts_persists_and_idempotently_updates() {
        let root = std::env::temp_dir().join(format!(
            "researchledger-provider-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        vault_setup::ensure_root(&root);
        let summary = import_provider_posts(
            &root.to_string_lossy(),
            ProviderMetadata::reddit(),
            vec![ProviderImportInput {
                url: "https://www.reddit.com/r/rust/comments/abc/hi/".into(),
                title: "Test".into(),
                body: "first body".into(),
            }],
        )
        .unwrap();
        assert_eq!(summary.created, 1);
        assert!(root.join("sources/reddit").exists());
        // Re-running with identical content must remain Unchanged (re-imports of the
        // same capture must not churn through "Updated" because the renderer embeds
        // the captured-at timestamp only in the DB column, not in the hash).
        let again = import_provider_posts(
            &root.to_string_lossy(),
            ProviderMetadata::reddit(),
            vec![ProviderImportInput {
                url: "https://www.reddit.com/r/rust/comments/abc/hi/".into(),
                title: "Test".into(),
                body: "first body".into(),
            }],
        )
        .unwrap();
        assert_eq!(again.created, 0);
        assert_eq!(again.updated, 0);
        assert_eq!(again.unchanged, 1);
        assert_eq!(again.failed, 0);
        let _ = std::fs::remove_dir_all(root);
    }

    mod vault_setup {
        pub(super) fn ensure_root(root: &std::path::Path) {
            std::fs::create_dir_all(root).unwrap();
        }
    }
}
