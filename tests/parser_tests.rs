use envexa::toolchains::{PackageInfo, npm::parse_outdated};

#[test]
fn test_npm_parse_outdated() {
    let json_output = r#"{
      "axios": {
        "current": "1.6.8",
        "wanted": "1.7.2",
        "latest": "1.7.2",
        "dependent": "global",
        "location": "/usr/local/lib/node_modules/axios"
      },
      "eslint": {
        "current": "8.57.0",
        "wanted": "9.4.0",
        "latest": "9.4.0",
        "dependent": "global",
        "location": "/usr/local/lib/node_modules/eslint"
      }
    }"#;

    let parsed = parse_outdated(json_output);
    assert_eq!(parsed.len(), 2);
    
    // Test that the parsing correctly identifies fields
    let axios = parsed.iter().find(|p| p.name == "axios").unwrap();
    assert_eq!(axios.current, "1.6.8");
    assert_eq!(axios.latest, "1.7.2");
}
