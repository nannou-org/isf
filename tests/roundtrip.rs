// Deserialize each ISF, serialize it back to JSON, then deserialize it again and make sure both
// deserialized instances match.
#[test]
fn roundtrip_test_files() {
    let test_files_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files");
    assert!(test_files_path.exists());
    assert!(test_files_path.is_dir());
    for entry in std::fs::read_dir(test_files_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str());
        if ext == Some("fs") || ext == Some("vs") {
            let glsl_str = std::fs::read_to_string(&path).unwrap();
            let isf = match isf::parse(&glsl_str) {
                // Ignore non-ISF vertex shaders.
                Err(isf::ParseError::MissingTopComment) if ext == Some("vs") => continue,
                Err(err) => panic!("err while parsing {}: {}", path.display(), err),
                Ok(isf) => isf,
            };
            let isf_string = serde_json::to_string_pretty(&isf).unwrap();
            let isf2 = serde_json::from_str(&isf_string).unwrap();
            assert_eq!(isf, isf2);
        }
    }
}
