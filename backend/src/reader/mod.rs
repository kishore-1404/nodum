use anyhow::{Result, Context};
use std::path::Path;

/// A parsed chunk of text from a document
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    pub index: i32,
    pub page_number: Option<i32>,
    pub chapter_title: Option<String>,
    pub content: String,
    pub token_estimate: usize,
}

/// Parse a PDF file into semantic chunks
pub fn parse_pdf(path: &Path) -> Result<(String, Vec<DocumentChunk>)> {
    let bytes = std::fs::read(path).context("Failed to read PDF file")?;

    // Extract text using pdf-extract
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .context("Failed to extract text from PDF. The file may be scanned (image-only). OCR is required.")?;

    if text.trim().is_empty() {
        anyhow::bail!("PDF appears to be image-only (scanned). OCR processing is required for this document.");
    }

    // Try to extract title from first meaningful line
    let title = text.lines()
        .find(|l| !l.trim().is_empty() && l.trim().len() > 3)
        .unwrap_or("Untitled Document")
        .trim()
        .to_string();

    let chunks = chunk_text(&text, 600, 100);
    Ok((title, chunks))
}

/// Parse an EPUB file into semantic chunks
pub fn parse_epub(path: &Path) -> Result<(String, Vec<DocumentChunk>)> {
    let doc = epub::doc::EpubDoc::new(path)
        .map_err(|e| anyhow::anyhow!("Failed to parse EPUB: {}", e))?;

    let title = doc.mdata("title").unwrap_or_else(|| "Untitled".into());

    let mut all_text = String::new();
    let mut doc = epub::doc::EpubDoc::new(path)
        .map_err(|e| anyhow::anyhow!("Failed to parse EPUB: {}", e))?;

    let mut chapter_num = 0;
    let mut raw_chunks = Vec::new();

    while doc.go_next() {
        if let Some((content, _mime)) = doc.get_current_str() {
            // Strip HTML tags for plain text
            let plain = strip_html(&content);
            if plain.trim().len() > 50 {
                chapter_num += 1;
                let chapter_chunks = chunk_text_with_chapter(&plain, 600, 100, chapter_num);
                raw_chunks.extend(chapter_chunks);
            }
        }
    }

    // Re-index
    let chunks: Vec<DocumentChunk> = raw_chunks.into_iter().enumerate().map(|(i, mut c)| {
        c.index = i as i32;
        c
    }).collect();

    Ok((title, chunks))
}

/// Parse a URL / web article into chunks
pub fn parse_url_content(html: &str, title: &str) -> Vec<DocumentChunk> {
    let plain = strip_html(html);
    let mut chunks = chunk_text(&plain, 600, 100);
    // Override title detection
    for c in &mut chunks {
        if c.chapter_title.is_none() {
            c.chapter_title = Some(title.to_string());
        }
    }
    chunks
}

/// Chunk text into ~target_tokens-sized pieces with overlap
fn chunk_text(text: &str, target_tokens: usize, overlap_tokens: usize) -> Vec<DocumentChunk> {
    chunk_text_with_chapter(text, target_tokens, overlap_tokens, 0)
}

fn chunk_text_with_chapter(text: &str, target_tokens: usize, overlap_tokens: usize, chapter: i32) -> Vec<DocumentChunk> {
    let paragraphs: Vec<&str> = text.split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    let mut chunks = Vec::new();
    let mut current_content = String::new();
    let mut current_tokens = 0;
    let mut chunk_index = 0;
    let mut page_estimate = 1;

    for para in &paragraphs {
        let para_tokens = estimate_tokens(para);

        // If adding this paragraph exceeds target, finalize current chunk
        if current_tokens + para_tokens > target_tokens && !current_content.is_empty() {
            chunks.push(DocumentChunk {
                index: chunk_index,
                page_number: Some(page_estimate),
                chapter_title: if chapter > 0 { Some(format!("Chapter {}", chapter)) } else { None },
                content: current_content.clone(),
                token_estimate: current_tokens,
            });
            chunk_index += 1;

            // Keep overlap: take last ~overlap_tokens worth of text
            let overlap_text = get_tail_tokens(&current_content, overlap_tokens);
            current_content = overlap_text;
            current_tokens = estimate_tokens(&current_content);
        }

        if !current_content.is_empty() {
            current_content.push_str("\n\n");
        }
        current_content.push_str(para);
        current_tokens += para_tokens;

        // Rough page estimation (250 words ~ 1 page)
        page_estimate = (chunks.len() * target_tokens / 250).max(1) as i32;
    }

    // Final chunk
    if !current_content.trim().is_empty() {
        chunks.push(DocumentChunk {
            index: chunk_index,
            page_number: Some(page_estimate),
            chapter_title: if chapter > 0 { Some(format!("Chapter {}", chapter)) } else { None },
            content: current_content,
            token_estimate: current_tokens,
        });
    }

    chunks
}

/// Rough token estimation: ~0.75 tokens per word for English
fn estimate_tokens(text: &str) -> usize {
    let words = text.split_whitespace().count();
    (words as f64 * 1.3) as usize
}

/// Get the tail of text that's approximately `target_tokens` tokens
fn get_tail_tokens(text: &str, target_tokens: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let target_words = (target_tokens as f64 / 1.3) as usize;
    if words.len() <= target_words {
        return text.to_string();
    }
    words[words.len() - target_words..].join(" ")
}

/// Strip HTML tags from a string
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut last_was_space = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            }
            _ if !in_tag => {
                if ch.is_whitespace() {
                    if !last_was_space {
                        result.push(' ');
                        last_was_space = true;
                    }
                } else {
                    result.push(ch);
                    last_was_space = false;
                }
            }
            _ => {}
        }
    }

    // Decode common HTML entities
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text() {
        let text = (0..50).map(|i| format!("Paragraph {} with some content that fills the space.", i)).collect::<Vec<_>>().join("\n\n");
        let chunks = chunk_text(&text, 100, 20);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
        }
    }

    #[test]
    fn test_strip_html() {
        let html = "<p>Hello <strong>world</strong></p>";
        assert_eq!(strip_html(html).trim(), "Hello world");
    }

    #[test]
    fn test_token_estimate() {
        assert!(estimate_tokens("Hello world this is a test") > 5);
    }
}
