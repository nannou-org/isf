# isf [![Build Status](https://travis-ci.org/nannou-org/isf.svg?branch=master)](https://travis-ci.org/nannou-org/isf) [![Crates.io](https://img.shields.io/crates/v/isf.svg)](https://crates.io/crates/isf) [![Crates.io](https://img.shields.io/crates/l/isf.svg)](https://github.com/nannou-org/isf/blob/master/LICENSE-MIT) [![docs.rs](https://docs.rs/isf/badge.svg)](https://docs.rs/isf/)

Parsing, deserialization and serialization of ISF - the Interactive Shader Format.

Implementation is as described under "The ISF Specification Vsn 2.0" part of [the spec site](https://www.interactiveshaderformat.com/spec). 

The [**parse**](https://docs.rs/isf/latest/isf/fn.parse.html) function can parse
a given GLSL string to produce an
[**Isf**](https://docs.rs/isf/latest/isf/struct.Isf.html) instance. The **Isf**
type represents a fully structured representation of the format, including typed
[**Input**](https://docs.rs/isf/latest/isf/struct.Input.html)s.

## Tests

The repo includes all shaders from the "ISF Test/Tutorial filters" collection
provided by the specification site along with all shaders within the
["Vidvox/ISF-Files" repository](https://github.com/Vidvox/ISF-Files) for
automated testing. These tests do a full roundtrip on every shader in the
following steps:

1. Read the GLSL string from file.
2. Parse the GLSL string for the top-level dictionary.
3. Deserialize the JSON to an `Isf` struct.
4. Serialize the `Isf` struct back to a JSON string.
5. Deserialize the new JSON string to an `Isf` struct again.
6. Assert the first `Isf` instance is identical to the second.

Thanks to the Vidvox crew for allowing us to include these tests in the repo for
easy automated testing!

## Example

Parsing a GLSL string to an ISF struct looks like this:

```rust
let isf = isf::parse(&glsl_str).unwrap();
```

See the [`Isf` struct docs](https://docs.rs/isf/latest/isf/struct.Isf.html) to
get an idea of what you can do with it!

Here's a copy of the `tests/roundtrip.rs` test as described above:

```rust
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
```

## About

This crate has been developed as a step towards creating an ISF live-coding
environment based on [**nannou**](https://nannou.cc), the creative coding
framework.
