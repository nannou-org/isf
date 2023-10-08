use serde_json::json;

const OMMISSION_TEST: &str = r#"""
/* {
  "ISFVSN" : "2",
  "INPUTS" : [
    {
      "NAME" : "inputImage",
      "TYPE" : "image"
    }
  ]
 }
*/
void main() {

}
"""#;

#[test]
fn does_not_emit_extra_fields() {
    let isf = isf::parse(&OMMISSION_TEST).unwrap();
    let isf_string = serde_json::to_string_pretty(&isf).unwrap();
    let object: serde_json::Value = serde_json::from_str(&isf_string).unwrap();
    let expected_object = json!({
        "ISFVSN" : "2",
        "INPUTS" : [{ "NAME" : "inputImage", "TYPE" : "image" }]
    });
    assert_eq!(object, expected_object);
}

const PANIC_TEST: &str = r#"""
/* {
  "INPUTS" : [
    {
      "NAME" : "panic_causer",
      "TYPE" : "panic_type"
    }
  ]
 }
*/
void main() {

}
"""#;

#[test]
fn does_not_panic_on_unknown_type() {
    let isf_res = isf::parse(&PANIC_TEST);
    assert!(matches!(isf_res, Err(isf::ParseError::Json { .. })));
}
