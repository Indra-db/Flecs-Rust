use poly_doctest::generator::generate_docs;
use poly_doctest::source::DocsSource;
use rust_doctest::RustGenerator;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_quickstart_generation() {
    let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");

    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("output");

    fs::create_dir_all(&output_dir).unwrap();

    // Generate from Quickstart.md
    let generator = RustGenerator;
    let source = DocsSource::local(&test_data_dir);
    generate_docs(&generator, source, Some(output_dir.clone())).unwrap();

    // Compare files
    let generated = fs::read_to_string(output_dir.join("quickstart.rs")).unwrap();
    let expected = fs::read_to_string(test_data_dir.join("quickstart_expected.rs")).unwrap();

    assert_eq!(
        generated, expected,
        "Generated file does not match expected"
    );
}
