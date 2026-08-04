use serde::Serialize;
mod chunking;
mod distill;
mod embeddings;
mod enrichment;
mod github;
mod hackernews;
mod linkedin;
mod provider_html;
mod rag;
mod reddit;
mod reference_fetch;
mod safe_paths;
mod storage;
mod x;

mod commands {
    include!("commands.rs");

    use super::{
        distill,
        embeddings::{LocalCrossEncoder, OllamaEmbedder},
        github,
        github::GithubClient,
        hackernews, linkedin, rag, reddit, reference_fetch, safe_paths, storage, x, VaultStatus,
    };
    use serde::Serialize;
    use sha2::{Digest, Sha256};
    use tauri::Manager;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ImportSummary {
        pub created: u64,
        pub updated: u64,
        pub unchanged: u64,
        pub failed: u64,
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
        let html = std::fs::read_to_string(&html_path).map_err(|error| error.to_string())?;
        let posts = linkedin::parse_activity_html(&html);
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for post in posts {
            let id = post.url.rsplit(':').next().unwrap_or(&post.url).to_string();
            let content = format!("---\ntype: LinkedIn Post\nid: linkedin:{id}\ntitle: LinkedIn post {id}\ndescription: Captured LinkedIn post\nresource: {}\ntags: [linkedin, captured]\ntimestamp: {}\nsource_kind: linkedin\nsource_uri: {}\n---\n\n{}\n\n# Citations\n\n[1] [LinkedIn post]({})\n", post.url, chrono::Utc::now().to_rfc3339(), post.url, post.text, post.url);
            let document = storage::SourceDocument {
                id: format!("linkedin:{id}"),
                relative_path: format!("sources/linkedin/{id}.md"),
                title: format!("LinkedIn post {id}"),
                source_kind: "linkedin".into(),
                source_uri: Some(post.url),
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
    pub fn import_linkedin_capture(
        vault_path: String,
        capture_path: String,
    ) -> Result<ImportSummary, String> {
        let json = std::fs::read_to_string(&capture_path).map_err(|error| error.to_string())?;
        let posts = linkedin::parse_capture_json(&json).map_err(|error| error.to_string())?;
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for post in posts {
            let id = post.url.rsplit(':').next().unwrap_or(&post.url).to_string();
            let document = storage::SourceDocument {
                id: format!("linkedin:{id}"),
                relative_path: format!("sources/linkedin/{id}.md"),
                title: format!("LinkedIn post {id}"),
                source_kind: "linkedin".into(),
                source_uri: Some(post.url.clone()),
                content: format!("---\ntype: LinkedIn Post\nid: linkedin:{id}\ntitle: LinkedIn post {id}\ndescription: Captured LinkedIn post\nresource: {}\ntags: [linkedin, captured]\ntimestamp: {}\nsource_kind: linkedin\nsource_uri: {}\n---\n\n{}\n\n# Citations\n\n[1] [LinkedIn post]({})\n", post.url, chrono::Utc::now().to_rfc3339(), post.url, post.text, post.url),
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
    pub async fn capture_linkedin_browser(
        app: tauri::AppHandle,
        vault_path: String,
        activity_url: Option<String>,
        profile_path: Option<String>,
    ) -> Result<ImportSummary, String> {
        let output = std::path::PathBuf::from(&vault_path)
            .join(".researchledger")
            .join("linkedin-capture.json");
        let resource_script = app
            .path()
            .resource_dir()
            .map_err(|error| error.to_string())?
            .join("scripts/linkedin_capture.mjs");
        let script = if resource_script.exists() {
            resource_script
        } else {
            std::env::current_dir()
                .map_err(|error| error.to_string())?
                .join("scripts/linkedin_capture.mjs")
        };
        let mut command = tokio::process::Command::new("node");
        command.arg(script).arg("--output").arg(&output);
        if let Ok(resource_dir) = app.path().resource_dir() {
            let packaged_module = resource_dir.join("node_modules/playwright/index.mjs");
            if packaged_module.exists() {
                command.env("RESEARCHLEDGER_PLAYWRIGHT_MODULE", packaged_module);
            }
        }
        if let Some(profile) = profile_path.filter(|value| !value.trim().is_empty()) {
            let safe_profile = safe_paths::ensure_safe_command_arg(&profile, "profile")
                .map_err(|error| error.to_string())?;
            command.arg("--profile").arg(safe_profile);
        }
        if let Some(url) = activity_url {
            command.arg("--url").arg(url);
        }
        let result = command.output().await.map_err(|error| error.to_string())?;
        if !result.status.success() {
            return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
        }
        import_linkedin_capture(vault_path, output.to_string_lossy().into_owned())
    }

    #[tauri::command]
    pub async fn open_linkedin_signin(
        app: tauri::AppHandle,
        profile_path: Option<String>,
    ) -> Result<String, String> {
        let profile = profile_path
            .filter(|value| !value.trim().is_empty())
            .map(|value| safe_paths::ensure_safe_command_arg(&value, "profile"))
            .transpose()
            .map_err(|error| error.to_string())?;
        if let Some(path) = profile.as_deref() {
            for lock in ["SingletonLock", "SingletonSocket", "SingletonCookie"] {
                if std::path::Path::new(path).join(lock).exists() {
                    return Err(format!(
                        "BROWSER_PROFILE_UNAVAILABLE: LinkedIn profile is already open ({path}). Close the existing Chromium window or choose a dedicated profile; no profile data was deleted."
                    ));
                }
            }
        }
        let resource_script = app
            .path()
            .resource_dir()
            .map_err(|error| error.to_string())?
            .join("scripts/linkedin_signin.mjs");
        let script = if resource_script.exists() {
            resource_script
        } else {
            std::env::current_dir()
                .map_err(|error| error.to_string())?
                .join("scripts/linkedin_signin.mjs")
        };
        let mut command = tokio::process::Command::new("node");
        command.arg(script);
        if let Some(path) = profile {
            command.arg("--profile").arg(path);
        }
        command.spawn().map_err(|error| error.to_string())?;
        Ok("LinkedIn sign-in browser opened; close it when authentication is complete.".into())
    }

    #[tauri::command]
    pub fn import_hackernews_html(
        vault_path: String,
        html_path: String,
    ) -> Result<ImportSummary, String> {
        let html = std::fs::read_to_string(&html_path).map_err(|error| error.to_string())?;
        let posts = hackernews::parse_saved_html(&html);
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for post in posts {
            let id = post.id.clone();
            let content = format!(
                "---\ntype: Hacker News Story\nid: hackernews:{id}\ntitle: {title}\ndescription: Captured HN saved story\nresource: {url}\ntags: [hackernews, saved]\ntimestamp: {timestamp}\nsource_kind: hackernews\nsource_uri: {url}\nauthor: {author}\n---\n\n{text}\n\n# Citations\n\n[1] [HN item {id}]({url})\n",
                id = id,
                title = post.title,
                url = post.url,
                timestamp = chrono::Utc::now().to_rfc3339(),
                author = post.author,
                text = post.text,
            );
            let document = storage::SourceDocument {
                id: format!("hackernews:{id}"),
                relative_path: format!("sources/hackernews/{id}.md"),
                title: post.title.clone(),
                source_kind: "hackernews".into(),
                source_uri: Some(post.url.clone()),
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
    pub fn import_hackernews_capture(
        vault_path: String,
        capture_path: String,
    ) -> Result<ImportSummary, String> {
        let json = std::fs::read_to_string(&capture_path).map_err(|error| error.to_string())?;
        let posts = hackernews::parse_capture_json(&json).map_err(|error| error.to_string())?;
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for post in posts {
            let id = post.id.clone();
            let document = storage::SourceDocument {
                id: format!("hackernews:{id}"),
                relative_path: format!("sources/hackernews/{id}.md"),
                title: post.title.clone(),
                source_kind: "hackernews".into(),
                source_uri: Some(post.url.clone()),
                content: format!(
                    "---\ntype: Hacker News Story\nid: hackernews:{id}\ntitle: {title}\ndescription: Captured HN saved story\nresource: {url}\ntags: [hackernews, saved]\ntimestamp: {timestamp}\nsource_kind: hackernews\nsource_uri: {url}\nauthor: {author}\n---\n\n{text}\n\n# Citations\n\n[1] [HN item {id}]({url})\n",
                    id = id,
                    title = post.title,
                    url = post.url,
                    timestamp = chrono::Utc::now().to_rfc3339(),
                    author = post.author,
                    text = post.text,
                ),
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
    pub async fn capture_hackernews_browser(
        app: tauri::AppHandle,
        vault_path: String,
        activity_url: Option<String>,
        profile_path: Option<String>,
    ) -> Result<ImportSummary, String> {
        if let Some(url) = activity_url
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            safe_paths::ensure_safe_provider_url(url, &["news.ycombinator.com"])
                .map_err(|error| error.to_string())?;
        }
        let output = std::path::PathBuf::from(&vault_path)
            .join(".researchledger")
            .join("hackernews-capture.json");
        let resource_script = app
            .path()
            .resource_dir()
            .map_err(|error| error.to_string())?
            .join("scripts/hackernews_capture.mjs");
        let script = if resource_script.exists() {
            resource_script
        } else {
            std::env::current_dir()
                .map_err(|error| error.to_string())?
                .join("scripts/hackernews_capture.mjs")
        };
        let mut command = tokio::process::Command::new("node");
        command.arg(script).arg("--output").arg(&output);
        if let Ok(resource_dir) = app.path().resource_dir() {
            let packaged_module = resource_dir.join("node_modules/playwright/index.mjs");
            if packaged_module.exists() {
                command.env("RESEARCHLEDGER_PLAYWRIGHT_MODULE", packaged_module);
            }
        }
        if let Some(profile) = profile_path.filter(|value| !value.trim().is_empty()) {
            let safe_profile = safe_paths::ensure_safe_command_arg(&profile, "profile")
                .map_err(|error| error.to_string())?;
            command.arg("--profile").arg(safe_profile);
        }
        if let Some(url) = activity_url.filter(|value| !value.trim().is_empty()) {
            command.arg("--url").arg(url);
        }
        let result = command.output().await.map_err(|error| error.to_string())?;
        if !result.status.success() {
            return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
        }
        import_hackernews_capture(vault_path, output.to_string_lossy().into_owned())
    }

    /// Slugify a captured URL into a filesystem-safe document id fragment.
    /// Strips the `https?://` scheme, then replaces `/`, `?`, and any
    /// remaining `:` characters with `-`. This guarantees the id never
    /// contains a path separator (which would otherwise let a malicious
    /// capture file escape the vault's per-source folder).
    fn slug_from_url(url: &str) -> String {
        let mut slug = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .replace(['/', '?', ':', '#'], "-");
        while slug.contains("--") {
            slug = slug.replace("--", "-");
        }
        slug = slug.trim_matches('-').to_string();
        if slug.is_empty() {
            return "post".into();
        }
        slug
    }

    fn render_reddit_markdown(post: &reddit::RedditSavedPost) -> String {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let subreddit = post.subreddit.clone().unwrap_or_default();
        format!(
            "---\ntype: Reddit Post\nid: reddit:{slug}\ntitle: {title}\ndescription: Captured Reddit saved post\nresource: {url}\ntags: [reddit, saved]\ntimestamp: {timestamp}\nsource_kind: reddit\nsource_uri: {url}\nsubreddit: {subreddit}\n---\n\n{text}\n\n# Citations\n\n[1] [Reddit post]({url})\n",
            slug = slug_from_url(&post.url),
            title = post.title,
            url = post.url,
            timestamp = timestamp,
            text = post.text,
            subreddit = subreddit,
        )
    }

    fn render_x_markdown(post: &x::XSavedPost) -> String {
        let timestamp = chrono::Utc::now().to_rfc3339();
        format!(
            "---\ntype: X Bookmark\nid: x:{slug}\ntitle: X bookmark by {author}\ndescription: Captured X bookmark\nresource: {url}\ntags: [x, bookmark]\ntimestamp: {timestamp}\nsource_kind: x\nsource_uri: {url}\nauthor: {author}\n---\n\n{text}\n\n# Citations\n\n[1] [X post]({url})\n",
            slug = slug_from_url(&post.url),
            url = post.url,
            timestamp = timestamp,
            text = post.text,
            author = post.author,
        )
    }

    #[tauri::command]
    pub fn import_reddit_html(
        vault_path: String,
        html_path: String,
    ) -> Result<ImportSummary, String> {
        let html = std::fs::read_to_string(&html_path).map_err(|error| error.to_string())?;
        let posts = reddit::parse_saved_html(&html);
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for post in posts {
            let slug = slug_from_url(&post.url);
            let content = render_reddit_markdown(&post);
            let document = storage::SourceDocument {
                id: format!("reddit:{slug}"),
                relative_path: format!("sources/reddit/{slug}.md"),
                title: post.title.clone(),
                source_kind: "reddit".into(),
                source_uri: Some(post.url.clone()),
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
    pub fn import_reddit_capture(
        vault_path: String,
        capture_path: String,
    ) -> Result<ImportSummary, String> {
        let json = std::fs::read_to_string(&capture_path).map_err(|error| error.to_string())?;
        let posts = reddit::parse_capture_json(&json).map_err(|error| error.to_string())?;
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for post in posts {
            let slug = slug_from_url(&post.url);
            let content = render_reddit_markdown(&post);
            let document = storage::SourceDocument {
                id: format!("reddit:{slug}"),
                relative_path: format!("sources/reddit/{slug}.md"),
                title: post.title.clone(),
                source_kind: "reddit".into(),
                source_uri: Some(post.url.clone()),
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
    pub async fn capture_reddit_browser(
        app: tauri::AppHandle,
        vault_path: String,
        activity_url: Option<String>,
        profile_path: Option<String>,
    ) -> Result<ImportSummary, String> {
        // Validate the URL is on the Reddit allow-list before shelling out so a
        // crafted activity_url can't redirect the user's authenticated profile
        // to a different host.
        if let Some(url) = activity_url
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            safe_paths::ensure_safe_provider_url(
                url,
                &["reddit.com", "www.reddit.com", "old.reddit.com"],
            )
            .map_err(|error| error.to_string())?;
        }
        let output = std::path::PathBuf::from(&vault_path)
            .join(".researchledger")
            .join("reddit-capture.json");
        let resource_script = app
            .path()
            .resource_dir()
            .map_err(|error| error.to_string())?
            .join("scripts/reddit_capture.mjs");
        let script = if resource_script.exists() {
            resource_script
        } else {
            std::env::current_dir()
                .map_err(|error| error.to_string())?
                .join("scripts/reddit_capture.mjs")
        };
        let mut command = tokio::process::Command::new("node");
        command.arg(script).arg("--output").arg(&output);
        if let Ok(resource_dir) = app.path().resource_dir() {
            let packaged_module = resource_dir.join("node_modules/playwright/index.mjs");
            if packaged_module.exists() {
                command.env("RESEARCHLEDGER_PLAYWRIGHT_MODULE", packaged_module);
            }
        }
        if let Some(profile) = profile_path
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            let safe_profile = safe_paths::ensure_safe_command_arg(profile, "profile")
                .map_err(|error| error.to_string())?;
            command.arg("--profile").arg(safe_profile);
        }
        if let Some(url) = activity_url.filter(|value| !value.trim().is_empty()) {
            command.arg("--url").arg(url);
        }
        let result = command.output().await.map_err(|error| error.to_string())?;
        if !result.status.success() {
            return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
        }
        import_reddit_capture(vault_path, output.to_string_lossy().into_owned())
    }

    #[tauri::command]
    pub fn import_x_html(vault_path: String, html_path: String) -> Result<ImportSummary, String> {
        let html = std::fs::read_to_string(&html_path).map_err(|error| error.to_string())?;
        let posts = x::parse_bookmarks_html(&html);
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for post in posts {
            let slug = slug_from_url(&post.url);
            let content = render_x_markdown(&post);
            let document = storage::SourceDocument {
                id: format!("x:{slug}"),
                relative_path: format!("sources/x/{slug}.md"),
                title: format!("X bookmark by {}", post.author),
                source_kind: "x".into(),
                source_uri: Some(post.url.clone()),
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
    pub fn import_x_capture(
        vault_path: String,
        capture_path: String,
    ) -> Result<ImportSummary, String> {
        let json = std::fs::read_to_string(&capture_path).map_err(|error| error.to_string())?;
        let posts = x::parse_capture_json(&json).map_err(|error| error.to_string())?;
        let root = std::path::PathBuf::from(vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let mut connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for post in posts {
            let slug = slug_from_url(&post.url);
            let content = render_x_markdown(&post);
            let document = storage::SourceDocument {
                id: format!("x:{slug}"),
                relative_path: format!("sources/x/{slug}.md"),
                title: format!("X bookmark by {}", post.author),
                source_kind: "x".into(),
                source_uri: Some(post.url.clone()),
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
    pub async fn capture_x_browser(
        app: tauri::AppHandle,
        vault_path: String,
        activity_url: Option<String>,
        profile_path: Option<String>,
    ) -> Result<ImportSummary, String> {
        if let Some(url) = activity_url
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            safe_paths::ensure_safe_provider_url(url, &["x.com", "twitter.com"])
                .map_err(|error| error.to_string())?;
        }
        let output = std::path::PathBuf::from(&vault_path)
            .join(".researchledger")
            .join("x-capture.json");
        let resource_script = app
            .path()
            .resource_dir()
            .map_err(|error| error.to_string())?
            .join("scripts/x_capture.mjs");
        let script = if resource_script.exists() {
            resource_script
        } else {
            std::env::current_dir()
                .map_err(|error| error.to_string())?
                .join("scripts/x_capture.mjs")
        };
        let mut command = tokio::process::Command::new("node");
        command.arg(script).arg("--output").arg(&output);
        if let Ok(resource_dir) = app.path().resource_dir() {
            let packaged_module = resource_dir.join("node_modules/playwright/index.mjs");
            if packaged_module.exists() {
                command.env("RESEARCHLEDGER_PLAYWRIGHT_MODULE", packaged_module);
            }
        }
        if let Some(profile) = profile_path
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            let safe_profile = safe_paths::ensure_safe_command_arg(profile, "profile")
                .map_err(|error| error.to_string())?;
            command.arg("--profile").arg(safe_profile);
        }
        if let Some(url) = activity_url.filter(|value| !value.trim().is_empty()) {
            command.arg("--url").arg(url);
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
                document_id: result.document_id.clone(),
                score,
                result: Some(result),
            })
            .collect();
        let fused = rag::fuse_ranked(lexical, vector_hits, limit as usize);
        let reranked = match LocalCrossEncoder::from_environment() {
            Ok(Some(reranker)) => {
                let documents = fused
                    .iter()
                    .map(|result| format!("{}\n{}", result.title, result.snippet))
                    .collect::<Vec<_>>();
                reranker
                    .rerank(&query, &documents)
                    .await
                    .map(|scores| rag::rerank_with_cross_encoder(fused.clone(), scores))
                    .unwrap_or_else(|_| rag::rerank(&query, fused))
            }
            Ok(None) | Err(_) => rag::rerank(&query, fused),
        };
        Ok(rag::build_context(&query, reranked))
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
    pub async fn fetch_pending_references(
        vault_path: String,
        limit: Option<u32>,
    ) -> Result<ImportSummary, String> {
        let root = std::path::PathBuf::from(&vault_path);
        let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
        let connection = storage::open(&paths).map_err(|error| error.to_string())?;
        let jobs = storage::pending_reference_jobs(&connection, limit.unwrap_or(10).min(25))
            .map_err(|error| error.to_string())?;
        drop(connection);
        let client = reference_fetch::client().map_err(|error| error.to_string())?;
        let mut domain_budget = reference_fetch::DomainConcurrency::new(2);
        let mut summary = ImportSummary {
            created: 0,
            updated: 0,
            unchanged: 0,
            failed: 0,
        };
        for job in jobs {
            if !domain_budget.try_acquire(&job.target_url) {
                continue;
            }
            let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
            let connection = storage::open(&paths).map_err(|error| error.to_string())?;
            storage::mark_reference_fetch_started(&connection, &job)
                .map_err(|error| error.to_string())?;
            drop(connection);

            let fetched_result = reference_fetch::fetch_with_retry(&client, &job.target_url).await;
            domain_budget.release(&job.target_url);
            match fetched_result {
                Ok(fetched) => {
                    let result = reference_fetch::write_artifact(&root, &fetched);
                    match result {
                        Ok(_) => {
                            let reference_document = storage::SourceDocument {
                                id: format!("reference:{}", fetched.content_hash),
                                relative_path: format!(
                                    "sources/references/{}.md",
                                    fetched.content_hash
                                ),
                                title: format!("Fetched reference: {}", job.target_url),
                                source_kind: "reference".into(),
                                source_uri: Some(job.target_url.clone()),
                                content: reference_fetch::render_markdown(
                                    &job.target_url,
                                    &fetched,
                                ),
                                captured_at: chrono::Utc::now().to_rfc3339(),
                            };
                            let paths =
                                storage::initialize(&root).map_err(|error| error.to_string())?;
                            let mut connection =
                                storage::open(&paths).map_err(|error| error.to_string())?;
                            let upsert_result = match storage::upsert_document(
                                &mut connection,
                                &root,
                                &reference_document,
                            ) {
                                Ok(result) => result,
                                Err(error) => {
                                    storage::record_reference_fetch(
                                        &connection,
                                        &job,
                                        "failed",
                                        None,
                                        None,
                                        None,
                                        None,
                                        None,
                                        Some(&chrono::Utc::now().to_rfc3339()),
                                        Some(&error.to_string()),
                                    )
                                    .map_err(|error| error.to_string())?;
                                    summary.failed += 1;
                                    continue;
                                }
                            };
                            match upsert_result {
                                storage::UpsertResult::Created => summary.created += 1,
                                storage::UpsertResult::Updated => summary.updated += 1,
                                storage::UpsertResult::Unchanged => summary.unchanged += 1,
                            }
                            storage::record_reference_fetch(
                                &connection,
                                &job,
                                "done",
                                Some(&fetched.artifact_path),
                                Some(&fetched.content_type),
                                Some(fetched.http_status),
                                Some(fetched.byte_count),
                                Some(&fetched.content_hash),
                                Some(&chrono::Utc::now().to_rfc3339()),
                                None,
                            )
                            .map_err(|error| error.to_string())?;
                        }
                        Err(error) => {
                            let paths =
                                storage::initialize(&root).map_err(|error| error.to_string())?;
                            let connection =
                                storage::open(&paths).map_err(|error| error.to_string())?;
                            storage::record_reference_fetch(
                                &connection,
                                &job,
                                "failed",
                                None,
                                None,
                                None,
                                None,
                                None,
                                Some(&chrono::Utc::now().to_rfc3339()),
                                Some(&error.to_string()),
                            )
                            .map_err(|error| error.to_string())?;
                            summary.failed += 1;
                        }
                    }
                }
                Err(error) => {
                    let paths = storage::initialize(&root).map_err(|error| error.to_string())?;
                    let connection = storage::open(&paths).map_err(|error| error.to_string())?;
                    storage::record_reference_fetch(
                        &connection,
                        &job,
                        "failed",
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(&chrono::Utc::now().to_rfc3339()),
                        Some(&error.to_string()),
                    )
                    .map_err(|error| error.to_string())?;
                    summary.failed += 1;
                }
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
        storage::load_document(&connection, &root, &document_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "source document not found".to_string())?;
        let model = model.unwrap_or_else(|| "embeddinggemma".into());
        let chunks = {
            let mut statement = connection
                .prepare("SELECT id, text FROM chunks WHERE document_id=?1 ORDER BY ordinal")
                .map_err(|error| error.to_string())?;
            let rows = statement
                .query_map(rusqlite::params![document_id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| error.to_string())?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string())?
        };
        if chunks.is_empty() {
            return Err("source document has no chunks".into());
        }
        let texts = chunks
            .iter()
            .map(|(_, text)| text.clone())
            .collect::<Vec<_>>();
        let vectors = OllamaEmbedder::new(model.clone())
            .embed_batch(&texts)
            .await?;
        if vectors.len() != chunks.len() {
            return Err("embedding service returned an incomplete batch".into());
        }
        for ((chunk_id, text), vector) in chunks.iter().zip(vectors.iter()) {
            let input_hash = format!("{:x}", Sha256::digest(text.as_bytes()));
            connection
                .execute(
                    "INSERT INTO chunk_embeddings (chunk_id, model, embedding_version, input_hash, dimensions, vector_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) ON CONFLICT(chunk_id) DO UPDATE SET model=excluded.model, embedding_version=excluded.embedding_version, input_hash=excluded.input_hash, dimensions=excluded.dimensions, vector_json=excluded.vector_json, created_at=excluded.created_at",
                    rusqlite::params![
                        chunk_id,
                        model,
                        "v1",
                        input_hash,
                        vector.len(),
                        serde_json::to_string(vector).map_err(|error| error.to_string())?,
                        chrono::Utc::now().to_rfc3339()
                    ],
                )
                .map_err(|error| error.to_string())?;
        }
        Ok(vectors.len())
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
            commands::open_linkedin_signin,
            commands::import_hackernews_html,
            commands::import_hackernews_capture,
            commands::capture_hackernews_browser,
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
            commands::list_document_claims,
            commands::export_obsidian,
            commands::retrieve_context,
            commands::distill_document,
            commands::process_pending_enrichment,
            commands::fetch_pending_references,
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
        let provenance: (String, String, String) = db
            .query_row(
                "SELECT source_uri, locator, quote FROM provenance WHERE document_id = ?1",
                [&document.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(provenance.0, "https://github.com/octo/hello");
        assert_eq!(provenance.1, document.relative_path);
        assert_eq!(provenance.2, "hello");
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

    #[test]
    fn load_document_rejects_database_path_traversal() {
        let root = temp_root();
        let paths = initialize(&root).unwrap();
        let db = open(&paths).unwrap();
        db.execute(
            "INSERT INTO documents(id, canonical_path, title, source_kind, content_hash, captured_at, updated_at) VALUES('unsafe', '../outside.md', 'Unsafe', 'test', 'hash', 'now', 'now')",
            [],
        )
        .unwrap();
        assert!(load_document(&db, &root, "unsafe").is_err());
        let _ = std::fs::remove_dir_all(root);
    }
}
