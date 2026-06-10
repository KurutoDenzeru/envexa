// An example integration test for Envexa.
// This demonstrates how users or contributors can write high-level tests
// that validate Envexa's core functionality without needing complex setups.

use envexa::toolchains::ScanResult;

#[tokio::test]
async fn test_example_scan_result_creation() {
    // 1. Arrange: Create a mock scan result
    let result = ScanResult {
        tool: "mock-toolchain".to_string(),
        status: "Pass".to_string(),
        version: Some("v1.2.3".to_string()),
        node_version: None,
        python_version: None,
        ruby_version: None,
        rustc_version: None,
        cargo_version: None,
        pnpm_version: None,
        bun_version: None,
        deno_version: None,
        installed_count: None,
        outdated_formulae: vec![],
        outdated_casks: vec![],
        outdated: vec![],
        outdated_global: vec![],
        vulnerabilities: vec![],
        audit_items: vec![],
        disk_usage: None,
        issues: vec![],
        project_type: None,
    };

    // 2. Assert: Validate the struct properties
    assert_eq!(result.tool, "mock-toolchain");
    assert_eq!(result.status, "Pass");
    assert_eq!(result.version.unwrap(), "v1.2.3");
    assert!(result.outdated.is_empty());
}
