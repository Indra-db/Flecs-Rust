use crate::error::{DocGenError, Result};
use crate::model::{CodeSnippet, SourceFileSnippets};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::path::Path;

/// Indent code with specified number of spaces
///
/// This is a common utility for language generators that need to indent code
/// for function bodies, test blocks, etc.
pub fn indent_code(code: &str, indent_size: usize) -> String {
    code.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", " ".repeat(indent_size), line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extract file stem safely from a path string with proper error handling
///
/// This is a common utility for language generators that need to convert file paths
/// to safe identifiers (like function names, module names, etc).
///
/// Returns the file stem (filename without extension) in lowercase, with proper
/// error handling for invalid paths and non-UTF-8 filenames.
pub fn extract_file_stem(path_str: &str) -> Result<String> {
    let path = Path::new(path_str);
    let file_stem = path
        .file_stem()
        .ok_or_else(|| DocGenError::InvalidFileName {
            path: path.to_path_buf(),
        })?;

    file_stem
        .to_str()
        .ok_or_else(|| DocGenError::NonUtf8FileName {
            path: path.to_path_buf(),
        })
        .map(|s| s.to_lowercase())
}

#[derive(Debug)]
struct RawSnippet {
    code: String,
    heading_path: Vec<String>,
}

/// Extracts test code snippets for a specific language from markdown content
pub fn extract_test_snippets(
    md: &str,
    target_language: &str,
    source_file: &str,
) -> Result<SourceFileSnippets> {
    extract_test_snippets_with_hide_prefix(md, target_language, source_file, "HIDE:")
}

/// Extracts test code snippets for a specific language from markdown content with custom hide prefix
pub fn extract_test_snippets_with_hide_prefix(
    md: &str,
    target_language: &str,
    source_file: &str,
    hide_prefix: &str,
) -> Result<SourceFileSnippets> {
    let processed_md = process_hide_lines_in_markdown(md, hide_prefix);
    let raw_snippets = parse_markdown(&processed_md, target_language)?;
    let snippets = raw_snippets
        .into_iter()
        .enumerate()
        .map(|(index, raw)| CodeSnippet {
            name: generate_test_name(&raw.heading_path, index + 1, source_file),
            code: raw.code,
        })
        .collect();

    Ok(SourceFileSnippets {
        source_file: source_file.to_string(),
        snippets,
    })
}

fn parse_markdown(md: &str, target_language: &str) -> Result<Vec<RawSnippet>> {
    let mut snippets = Vec::new();
    let mut heading_stack = Vec::new();
    let parser = Parser::new_ext(md, Options::ENABLE_HEADING_ATTRIBUTES);

    let mut current_heading = String::new();
    let mut in_heading = false;
    let mut code_buffer = String::new();
    let mut in_target_test_block = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                // Truncate stack to appropriate level
                let depth = heading_level_to_depth(level);
                heading_stack.truncate(depth);
                current_heading.clear();
                in_heading = true;
            }
            Event::Text(text) if in_heading => {
                if !current_heading.is_empty() {
                    current_heading.push(' ');
                }
                current_heading.push_str(&text);
            }
            Event::End(TagEnd::Heading(_)) => {
                if in_heading {
                    heading_stack.push(current_heading.clone());
                    in_heading = false;
                }
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                in_target_test_block = is_target_test_block(&info, target_language);
                code_buffer.clear();
            }
            Event::Text(text) if in_target_test_block => {
                code_buffer.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak if in_target_test_block => {
                code_buffer.push('\n');
            }
            Event::End(TagEnd::CodeBlock) if in_target_test_block => {
                snippets.push(RawSnippet {
                    code: code_buffer.clone(),
                    heading_path: heading_stack.clone(),
                });
                in_target_test_block = false;
            }
            _ => {}
        }
    }

    Ok(snippets)
}

fn heading_level_to_depth(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 0,
        HeadingLevel::H2 => 1,
        HeadingLevel::H3 => 2,
        HeadingLevel::H4 => 3,
        HeadingLevel::H5 => 4,
        HeadingLevel::H6 => 5,
    }
}

fn is_target_test_block(info: &str, target_language: &str) -> bool {
    let parts: Vec<&str> = info.split_whitespace().collect();
    parts.len() >= 2 && parts[0] == target_language && parts[1] == "test"
}

fn process_hide_lines_in_markdown(md: &str, hide_prefix: &str) -> String {
    md.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            // Check if line starts with hide prefix (with or without space after)
            if let Some(after_prefix) = trimmed.strip_prefix(hide_prefix) {
                if after_prefix.is_empty() || after_prefix.starts_with(' ') {
                    // Remove the prefix and any following space
                    let leading_whitespace = &line[..line.len() - trimmed.len()];
                    let new_line = format!("{}{}", leading_whitespace, after_prefix.trim_start());
                    new_line
                } else {
                    // Prefix is part of a larger word, don't remove
                    line.to_string()
                }
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_test_name(heading_path: &[String], counter: usize, source_file: &str) -> String {
    let sanitized_headings: Vec<String> = heading_path
        .iter()
        .filter_map(|h| {
            let sanitized = sanitize_for_function_name(h);
            if sanitized.is_empty() {
                None
            } else {
                Some(sanitized)
            }
        })
        .collect();

    let mut name_parts = if sanitized_headings.is_empty() {
        vec![file_stem(source_file)]
    } else {
        sanitized_headings
    };

    name_parts.push(format!("{:02}", counter));
    name_parts.join("_").to_lowercase()
}

fn sanitize_for_function_name(input: &str) -> String {
    input
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
        .trim_matches('_')
        .to_string()
}

fn file_stem(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(sanitize_for_function_name)
        .unwrap_or_else(|| "test".to_string())
}
