use crate::error::Result;
use crate::model::SourceFileSnippets;
use crate::processor::extract_test_snippets_with_hide_prefix;
use crate::source::DocsSource;
use std::fs;
use std::path::{Path, PathBuf};

pub trait LangGenerator {
    /// Return the code fence languages this generator should process (e.g., ["rust", "rs"])
    fn code_fence_languages(&self) -> &[&str];
    fn default_output(&self) -> PathBuf;
    fn generate(&self, source_files: &[SourceFileSnippets], out_dir: &Path) -> Result<()>;
}

pub fn generate_docs<G: LangGenerator>(
    generator: &G,
    source: DocsSource,
    out_dir: Option<PathBuf>,
) -> Result<()> {
    generate_docs_with_recursive(generator, source, out_dir, true)
}

pub fn generate_docs_with_recursive<G: LangGenerator>(
    generator: &G,
    source: DocsSource,
    out_dir: Option<PathBuf>,
    recursive: bool,
) -> Result<()> {
    generate_docs_with_options(generator, source, out_dir, recursive, "HIDE:")
}

pub fn generate_docs_with_options<G: LangGenerator>(
    generator: &G,
    source: DocsSource,
    out_dir: Option<PathBuf>,
    recursive: bool,
    hide_prefix: &str,
) -> Result<()> {
    let out = out_dir.unwrap_or_else(|| generator.default_output());
    if !out.exists() {
        fs::create_dir_all(&out)?;
    }

    let files = source.discover_markdown_files_with_recursive(recursive)?;
    let mut all: Vec<SourceFileSnippets> = Vec::new();
    let mut total_snippets = 0;

    let target_languages = generator.code_fence_languages();

    for (path, content) in files {
        let mut file_snippets = Vec::new();

        // Extract snippets for each language this generator handles
        for &target_language in target_languages {
            let source_snippets = extract_test_snippets_with_hide_prefix(
                &content,
                target_language,
                &path,
                hide_prefix,
            )?;
            file_snippets.extend(source_snippets.snippets);
        }

        if !file_snippets.is_empty() {
            let file_name = std::path::Path::new(&path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            println!("  {} → {} test cases", file_name, file_snippets.len());
            total_snippets += file_snippets.len();

            all.push(SourceFileSnippets {
                source_file: path,
                snippets: file_snippets,
            });
        }
    }

    println!(
        "Generated {} test cases from {} files\n",
        total_snippets,
        all.len()
    );
    generator.generate(&all, &out)?;
    Ok(())
}
