const MAX_CHARS: usize = 1_200;

pub fn split_document(content: &str) -> Vec<(Option<String>, String)> {
    let mut chunks = Vec::new();
    let mut heading: Option<String> = None;
    let mut current = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix('#') {
            heading = Some(value.trim().to_string());
        }
        let would_overflow = !current.is_empty() && current.len() + line.len() + 1 > MAX_CHARS;
        if would_overflow || (trimmed.is_empty() && current.len() > 200) {
            chunks.push((heading.clone(), current.trim().to_string()));
            current.clear();
        }
        if !trimmed.is_empty() {
            current.push_str(line);
            current.push('\n');
        }
    }
    if !current.trim().is_empty() {
        chunks.push((heading, current.trim().to_string()));
    }
    if chunks.is_empty() {
        vec![(None, String::new())]
    } else {
        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_heading_context_and_bounds_chunk_size() {
        let content = format!("# Intro\n{}\n\n# Next\nsecond", "a".repeat(1_300));
        let chunks = split_document(&content);
        assert!(chunks.len() >= 2);
        assert_eq!(chunks[0].0.as_deref(), Some("Intro"));
        assert!(chunks.iter().all(|(_, text)| text.len() <= 1_301));
    }
}
