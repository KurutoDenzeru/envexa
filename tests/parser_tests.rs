use envexa::toolchains::npm::parse_outdated;

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

use envexa::toolchains::cargo::parse_outdated as cargo_parse_outdated;

#[test]
fn test_cargo_parse_outdated() {
    let json_output = r#"{
      "dependencies": [
        {
          "name": "serde",
          "project_version": "1.0.197",
          "compat_version": "1.0.203",
          "latest_version": "1.0.203",
          "kind": "Normal"
        },
        {
          "name": "tokio",
          "project_version": "1.37.0",
          "compat_version": "1.38.0",
          "latest_version": "1.38.0",
          "kind": "Normal"
        }
      ]
    }"#;

    let parsed = cargo_parse_outdated(json_output);
    assert_eq!(parsed.len(), 2);
    
    let serde_pkg = parsed.iter().find(|p| p.name == "serde").unwrap();
    assert_eq!(serde_pkg.current, "1.0.197");
    assert_eq!(serde_pkg.latest, "1.0.203");
}
