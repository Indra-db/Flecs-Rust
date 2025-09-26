use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use poly_doctest::{
    extract_file_stem, indent_code, CodeSnippet, LangGenerator, Result, SourceFileSnippets,
};

/// Rust-specific code generator that creates test modules from documentation snippets.
#[derive(Default)]
pub struct RustGenerator;

impl LangGenerator for RustGenerator {
    fn code_fence_languages(&self) -> &[&str] {
        &["rust", "rs"]
    }

    fn default_output(&self) -> PathBuf {
        // Default to flecs_ecs/tests/docs relative to workspace root
        PathBuf::from("flecs_ecs/tests/docs")
    }

    fn generate(&self, source_files: &[SourceFileSnippets], output_path: &Path) -> Result<()> {
        // Ensure output directory exists
        fs::create_dir_all(output_path)?;

        // Convert source files to modules
        let mut tests_by_module: HashMap<String, Vec<String>> = HashMap::new();

        for source_file in source_files {
            let module_name = extract_file_stem(&source_file.source_file)?;

            let mut test_functions = Vec::new();
            for snippet in &source_file.snippets {
                let test_func = self.generate_test_function(snippet)?;
                test_functions.push(test_func);
            }

            if !test_functions.is_empty() {
                tests_by_module.insert(module_name, test_functions);
            }
        }

        // Generate the main.rs file with module declarations
        self.create_main_module(output_path, &tests_by_module)?;

        // Generate individual test module files
        self.create_test_modules(output_path, &tests_by_module)?;

        println!("Output written to {}", output_path.display());

        Ok(())
    }
}

impl RustGenerator {
    /// Generate a single test function from a code snippet.
    fn generate_test_function(&self, snippet: &CodeSnippet) -> Result<String> {
        // Code is already processed (HIDE: lines removed by processor)
        let test_code = &snippet.code;

        // Indent the code properly (4 spaces for function body)
        let indented_code = indent_code(test_code, 4);

        let snippet_name = &snippet.name;

        Ok(format!(
            r#"#[test]
fn {snippet_name}() {{
{indented_code}
}}"#,
        ))
    }

    /// Create the main.rs file with module declarations.
    fn create_main_module(
        &self,
        output_path: &Path,
        modules: &HashMap<String, Vec<String>>,
    ) -> Result<()> {
        let mut content = String::from(
            r#"//! Generated tests from Flecs documentation

#![allow(unused_imports, unused_variables, dead_code, non_snake_case, path_statements, unreachable_code)]
#![cfg_attr(rustfmt, rustfmt_skip)]

pub mod common_test;

"#,
        );

        // Add module declarations in sorted order
        let mut module_names: Vec<_> = modules.keys().collect();
        module_names.sort();

        for module_name in module_names {
            content.push_str(&format!("mod {module_name};\n"));
        }

        let main_file = output_path.join("main.rs");
        std::fs::write(main_file, content)?;

        Ok(())
    }

    /// Create individual test module files.
    fn create_test_modules(
        &self,
        output_path: &Path,
        modules: &HashMap<String, Vec<String>>,
    ) -> Result<()> {
        for (module_name, test_functions) in modules {
            let content = format!(
                r#"//! Tests from {module_name}.md

#![allow(unused_imports, unused_variables, dead_code, non_snake_case, path_statements, unreachable_code, unused_mut)]
#![cfg_attr(rustfmt, rustfmt_skip)]

use crate::common_test::*;

{}"#,
                test_functions.join("\n\n")
            );

            let module_file = output_path.join(format!("{module_name}.rs"));
            std::fs::write(module_file, content)?;
        }

        Ok(())
    }
}
