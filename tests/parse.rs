// Check that we can parse and deserialize every test file.
#[test]
fn parse_test_files() {
    let test_files_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files");
    assert!(test_files_path.exists());
    assert!(test_files_path.is_dir());
    for entry in std::fs::read_dir(test_files_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str());
        if ext == Some("fs") || ext == Some("vs") {
            let glsl_str = std::fs::read_to_string(&path).unwrap();
            let _isf = match isf::parse(&glsl_str) {
                // Ignore non-ISF vertex shaders.
                Err(isf::ParseError::MissingTopComment) if ext == Some("vs") => continue,
                Err(err) => panic!("err while parsing {}: {}", path.display(), err),
                Ok(isf) => isf,
            };
        }
    }
}
