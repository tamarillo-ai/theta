// integration tests against the live MCP Registry
//
// these tests require network access and hit the real registry
// skipped by default, run with:
//  cargo nextest run -p theta-registry --test fetch_real --features online-tests

use theta_registry::{RegistryClient, is_registry_name, parse_registry_ref, synthesize_tool};

fn client() -> RegistryClient {
    RegistryClient::new(None).expect("failed to create registry client")
}

// pypi (stdio)

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn fetch_pypi_server() {
    let resp = client()
        .get_server("io.github.shigechika/mcp-stdio", None)
        .unwrap();

    assert_eq!(resp.server.name, "io.github.shigechika/mcp-stdio");
    assert!(!resp.server.packages.is_empty());

    let pkg = &resp.server.packages[0];
    assert_eq!(pkg.registry_type, "pypi");
    assert_eq!(pkg.identifier, "mcp-stdio");
    assert_eq!(pkg.runtime_hint.as_deref(), Some("uvx"));

    let tool = synthesize_tool(&resp.server).unwrap();
    assert_eq!(tool.name, "mcp-stdio");
    let cmd = tool.command.unwrap();
    assert_eq!(cmd[0], "uvx");
    assert!(
        cmd[1].starts_with("mcp-stdio=="),
        "expected pinned version, got {}",
        cmd[1]
    );
}

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn fetch_pypi_pinned_version() {
    let resp = client()
        .get_server("io.github.shigechika/mcp-stdio", Some("0.4.7"))
        .unwrap();
    assert_eq!(resp.server.version.as_deref(), Some("0.4.7"));

    let tool = synthesize_tool(&resp.server).unwrap();
    assert_eq!(tool.version, "0.4.7");
}

// npm (stdio)

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn fetch_npm_server() {
    let resp = client().get_server("ai.autoblocks/ctxl-mcp", None).unwrap();

    assert_eq!(resp.server.name, "ai.autoblocks/ctxl-mcp");
    let pkg = resp
        .server
        .packages
        .iter()
        .find(|p| p.registry_type == "npm");
    assert!(pkg.is_some(), "expected an npm package");

    let tool = synthesize_tool(&resp.server).unwrap();
    let cmd = tool.command.unwrap();
    assert_eq!(cmd[0], "npx");
    assert_eq!(cmd[1], "-y");
    assert!(
        cmd[2].contains("ctxl-mcp"),
        "expected identifier to contain 'ctxl-mcp', got {}",
        cmd[2]
    );
}

// oci (docker)

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn fetch_oci_server() {
    let resp = client().get_server("ai.aliengiraffe/spotdb", None).unwrap();

    assert_eq!(resp.server.name, "ai.aliengiraffe/spotdb");
    let pkg = resp
        .server
        .packages
        .iter()
        .find(|p| p.registry_type == "oci");
    assert!(pkg.is_some(), "expected an oci package");

    let tool = synthesize_tool(&resp.server).unwrap();
    let cmd = tool.command.unwrap();
    assert_eq!(cmd[0], "docker");
    assert_eq!(cmd[1], "run");
    assert!(
        cmd.last().unwrap().contains("spotdb"),
        "expected identifier to contain 'spotdb'"
    );
}

// nuget (dotnet)

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn fetch_nuget_server() {
    let resp = client().get_server("com.microsoft/nuget", None).unwrap();

    assert_eq!(resp.server.name, "com.microsoft/nuget");
    let pkg = resp
        .server
        .packages
        .iter()
        .find(|p| p.registry_type == "nuget");
    assert!(pkg.is_some(), "expected a nuget package");

    let tool = synthesize_tool(&resp.server).unwrap();
    let cmd = tool.command.unwrap();
    assert_eq!(cmd[0], "dotnet");
    assert_eq!(cmd[1], "tool");
    assert_eq!(cmd[2], "run");
}

// remote (http)

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn fetch_remote_server() {
    let resp = client()
        .get_server("ai.filegraph/document-processing", None)
        .unwrap();

    assert_eq!(resp.server.name, "ai.filegraph/document-processing");
    assert!(!resp.server.remotes.is_empty());

    let tool = synthesize_tool(&resp.server).unwrap();
    assert_eq!(tool.name, "document-processing");
    assert!(tool.command.is_none());
    assert!(tool.url.as_ref().unwrap().starts_with("https://"));
}

// search

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn search_returns_results() {
    let results = client().search("filesystem", 5).unwrap();
    assert!(
        !results.servers.is_empty(),
        "expected results for 'filesystem'"
    );
    assert!(!results.servers[0].server.name.is_empty());
}

// error handling

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn fetch_nonexistent_server_fails() {
    let result = client().get_server("io.github.nonexistent/does-not-exist-99999", None);
    assert!(result.is_err());
}

// caching

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn cache_hit_on_second_fetch() {
    let c = client();
    let resp1 = c
        .get_server("io.github.shigechika/mcp-stdio", None)
        .unwrap();
    let resp2 = c
        .get_server("io.github.shigechika/mcp-stdio", None)
        .unwrap();
    assert_eq!(resp1.server.name, resp2.server.name);
    assert_eq!(resp1.server.version, resp2.server.version);
}

// end to end

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn end_to_end_parse_and_fetch() {
    let input = "io.github.shigechika/mcp-stdio@0.4.7";
    assert!(is_registry_name(input));

    let (name, version) = parse_registry_ref(input);
    assert_eq!(name, "io.github.shigechika/mcp-stdio");
    assert_eq!(version, Some("0.4.7"));

    let resp = client().get_server(name, version).unwrap();
    let tool = synthesize_tool(&resp.server).unwrap();
    assert_eq!(tool.name, "mcp-stdio");
    assert_eq!(tool.version, "0.4.7");
}
