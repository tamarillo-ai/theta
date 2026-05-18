use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Typed errors for registry operations
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RegistryError {
    /// Server or version not found (404)
    #[error("server '{name}' version '{version}' not found in registry")]
    NotFound {
        /// Server name
        name: String,
        /// Requested version
        version: String,
    },
    /// Non-success HTTP status (not 404)
    #[error("registry request failed: {status} {body}")]
    Http {
        /// HTTP status code
        status: u16,
        /// Response body (or read-error placeholder)
        body: String,
    },
    /// Connection or transport failure
    #[error("registry transport error: {0}")]
    Transport(#[from] reqwest::Error),
    /// Cache I/O failure
    #[error("registry cache error: {0}")]
    Cache(#[from] std::io::Error),
    /// JSON decode failure
    #[error("registry decode error: {0}")]
    Decode(#[from] serde_json::Error),
    /// Server has no packages or remotes
    #[error("server '{name}' has no packages or remotes - cannot synthesize a tool entry")]
    NoPackages {
        /// Server name
        name: String,
    },
    /// Unsupported package type
    #[error(
        "unsupported package type '{registry_type}' (runtime hint: {runtime_hint:?}) - supported: npm, pypi, oci, nuget"
    )]
    UnsupportedPackageType {
        /// The `registry_type` value
        registry_type: String,
        /// Optional runtime hint
        runtime_hint: Option<String>,
    },
}

const DEFAULT_REGISTRY: &str = "https://registry.modelcontextprotocol.io/v0.1";
const CACHE_TTL_SECS: u64 = 3600; // 1 hour for latest lookups
const USER_AGENT: &str = "theta-registry/0.1";
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_MS: u64 = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
    pub server: ServerJson,
    #[serde(default, rename = "_meta")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerJson {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub packages: Vec<Package>,
    #[serde(default)]
    pub remotes: Vec<Remote>,
    #[serde(default)]
    pub repository: Option<Repository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub registry_type: String,
    pub identifier: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub runtime_hint: Option<String>,
    pub transport: Transport,
    #[serde(default)]
    pub environment_variables: Vec<EnvVar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transport {
    #[serde(rename = "type")]
    pub transport_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVar {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_required: bool,
    #[serde(default)]
    pub is_secret: bool,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remote {
    #[serde(rename = "type")]
    pub remote_type: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<EnvVar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub url: String,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerListResponse {
    pub servers: Vec<ServerResponse>,
    #[serde(default)]
    pub metadata: Option<ListMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMetadata {
    pub next_cursor: Option<String>,
    pub count: Option<u64>,
}

pub struct SynthesizedTool {
    pub name: String,
    pub command: Option<Vec<String>>,
    pub url: Option<String>,
    pub env_vars: Vec<EnvVar>,
    pub headers: Vec<EnvVar>,
    pub description: Option<String>,
    pub version: String,
    pub registry_name: String,
}

pub struct RegistryClient {
    base_url: String,
    http: reqwest::blocking::Client,
    cache_dir: PathBuf,
}

impl RegistryClient {
    pub fn new(registry_url: Option<&str>) -> Result<Self, RegistryError> {
        let base_url = registry_url
            .unwrap_or(DEFAULT_REGISTRY)
            .trim_end_matches('/')
            .to_string();

        let http = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()?;

        let cache_dir = registry_cache_dir();

        Ok(Self {
            base_url,
            http,
            cache_dir,
        })
    }

    pub fn get_server(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<ServerResponse, RegistryError> {
        self.get_server_impl(name, version, false)
    }

    pub fn get_server_no_cache(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<ServerResponse, RegistryError> {
        self.get_server_impl(name, version, true)
    }

    fn get_server_impl(
        &self,
        name: &str,
        version: Option<&str>,
        skip_cache: bool,
    ) -> Result<ServerResponse, RegistryError> {
        let version_key = version.unwrap_or("latest");

        let cache_path = self.cache_path(name, Some(version_key));
        if !skip_cache && cache_path.exists() {
            let use_cache = if version.is_some() {
                true // pinned versions never expire
            } else {
                let modified = fs_err::metadata(&cache_path)?.modified()?;
                modified.elapsed().unwrap_or(Duration::MAX) < Duration::from_secs(CACHE_TTL_SECS)
            };
            if use_cache {
                tracing::debug!(name, version_key, "registry cache hit");
                let data = fs_err::read_to_string(&cache_path)?;
                return Ok(serde_json::from_str(&data)?);
            }
            tracing::debug!(name, version_key, "registry cache expired");
        }

        let encoded_name = urlencoded(name);
        let encoded_version = urlencoded(version_key);
        let url = format!(
            "{}/servers/{}/versions/{}",
            self.base_url, encoded_name, encoded_version
        );
        tracing::debug!(name, version_key, %url, "fetching from registry");
        let resp = get_with_retry(&self.http, &url)?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            tracing::debug!(name, version_key, "not found in registry");
            return Err(RegistryError::NotFound {
                name: name.to_string(),
                version: version_key.to_string(),
            });
        }
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp
                .text()
                .unwrap_or_else(|e| format!("<body unreadable: {e}>"));
            return Err(RegistryError::Http { status, body });
        }
        let entry: ServerResponse = resp.json()?;

        let body = serde_json::to_string(&entry)?;
        if let Some(parent) = cache_path.parent() {
            fs_err::create_dir_all(parent)?;
            let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
            std::io::Write::write_all(&mut tmp, body.as_bytes())?;
            tmp.persist(&cache_path)
                .map_err(|e| RegistryError::Cache(e.error))?;
        }
        tracing::debug!(name, version_key, path = %cache_path.display(), "cached registry response");

        Ok(entry)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<ServerListResponse, RegistryError> {
        self.search_raw(query, limit)
    }

    fn search_raw(&self, query: &str, limit: usize) -> Result<ServerListResponse, RegistryError> {
        let url = format!(
            "{}/servers?search={}&limit={}",
            self.base_url,
            urlencoded(query),
            limit
        );
        tracing::debug!(query, limit, %url, "searching registry");
        let resp = get_with_retry(&self.http, &url)?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp
                .text()
                .unwrap_or_else(|e| format!("<body unreadable: {e}>"));
            return Err(RegistryError::Http { status, body });
        }
        Ok(resp.json()?)
    }

    fn cache_path(&self, name: &str, version: Option<&str>) -> PathBuf {
        let version_key = version.unwrap_or("_latest");
        self.cache_dir
            .join(name)
            .join(format!("{version_key}.json"))
    }
}

pub fn synthesize_tool(server: &ServerJson) -> Result<SynthesizedTool, RegistryError> {
    if let Some(pkg) = server.packages.first() {
        let command = synthesize_command(pkg)?;
        let tool_name = derive_tool_name(&server.name);
        return Ok(SynthesizedTool {
            name: tool_name,
            command: Some(command),
            url: None,
            env_vars: pkg.environment_variables.clone(),
            headers: vec![],
            description: server.description.clone(),
            version: pkg.version.clone().unwrap_or_default(),
            registry_name: server.name.clone(),
        });
    }

    if let Some(remote) = server.remotes.first() {
        let tool_name = derive_tool_name(&server.name);
        return Ok(SynthesizedTool {
            name: tool_name,
            command: None,
            url: Some(remote.url.clone()),
            env_vars: vec![],
            headers: remote.headers.clone(),
            description: server.description.clone(),
            version: server.version.clone().unwrap_or_default(),
            registry_name: server.name.clone(),
        });
    }

    Err(RegistryError::NoPackages {
        name: server.name.clone(),
    })
}

pub fn synthesize_command(pkg: &Package) -> Result<Vec<String>, RegistryError> {
    match (pkg.registry_type.as_str(), pkg.runtime_hint.as_deref()) {
        ("npm", Some("npx") | None) => {
            let spec = match &pkg.version {
                Some(v) => format!("{}@{}", pkg.identifier, v),
                None => pkg.identifier.clone(),
            };
            Ok(vec!["npx".into(), "-y".into(), spec])
        }
        ("pypi", Some("uvx") | None) => {
            let spec = match &pkg.version {
                Some(v) => format!("{}=={}", pkg.identifier, v),
                None => pkg.identifier.clone(),
            };
            Ok(vec!["uvx".into(), spec])
        }
        ("pypi", Some("pip")) => Ok(vec!["python".into(), "-m".into(), pkg.identifier.clone()]),
        ("oci", _) => Ok(vec![
            "docker".into(),
            "run".into(),
            "--rm".into(),
            "-i".into(),
            pkg.identifier.clone(),
        ]),
        ("nuget", _) => Ok(vec![
            "dotnet".into(),
            "tool".into(),
            "run".into(),
            pkg.identifier.clone(),
        ]),
        (rt, hint) => Err(RegistryError::UnsupportedPackageType {
            registry_type: rt.to_string(),
            runtime_hint: hint.map(std::string::ToString::to_string),
        }),
    }
}

pub fn is_registry_name(name: &str) -> bool {
    name.contains('/') && !name.starts_with('.') && !name.starts_with('/') && !name.contains("://")
}

pub fn parse_registry_ref(input: &str) -> (&str, Option<&str>) {
    match input.rsplit_once('@') {
        Some((name, version)) if !name.is_empty() && !version.is_empty() => (name, Some(version)),
        _ => (input, None),
    }
}

fn derive_tool_name(registry_name: &str) -> String {
    // "io.github.user/my-tool" --> "my-tool"
    registry_name
        .rsplit_once('/')
        .map_or(registry_name, |(_, name)| name)
        .to_string()
}

fn registry_cache_dir() -> PathBuf {
    theta_dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("registry")
}

fn urlencoded(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
}

/// Send a GET request with retry on transient errors (5xx, connection failures).
/// Exponential backoff: 500ms, 1s, 2s.
fn get_with_retry(
    http: &reqwest::blocking::Client,
    url: &str,
) -> Result<reqwest::blocking::Response, RegistryError> {
    let mut last_err = None;
    for attempt in 0..MAX_RETRIES {
        match http.get(url).send() {
            Ok(resp) if resp.status().is_server_error() && attempt + 1 < MAX_RETRIES => {
                let status = resp.status();
                tracing::debug!(url, %status, attempt, "transient server error, retrying");
                std::thread::sleep(Duration::from_millis(RETRY_BASE_MS * 2u64.pow(attempt)));
            }
            Ok(resp) => return Ok(resp),
            Err(e) if attempt + 1 < MAX_RETRIES => {
                tracing::debug!(url, %e, attempt, "request failed, retrying");
                std::thread::sleep(Duration::from_millis(RETRY_BASE_MS * 2u64.pow(attempt)));
                last_err = Some(e);
            }
            Err(e) => return Err(e.into()),
        }
    }
    Err(last_err.map_or_else(
        || RegistryError::Http {
            status: 0,
            body: format!("request failed after {MAX_RETRIES} retries"),
        },
        RegistryError::Transport,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_registry_name_detects_registry_refs() {
        assert!(is_registry_name("io.github.user/my-tool"));
        assert!(is_registry_name("ai.filegraph/document-processing"));
        assert!(is_registry_name("com.example/tool"));
    }

    #[test]
    fn is_registry_name_rejects_non_registry() {
        assert!(!is_registry_name("my-tool"));
        assert!(!is_registry_name("./local/path"));
        assert!(!is_registry_name("/absolute/path"));
        assert!(!is_registry_name("https://example.com/tool"));
    }

    #[test]
    fn parse_registry_ref_with_version() {
        let (name, version) = parse_registry_ref("io.github.user/tool@1.0.1");
        assert_eq!(name, "io.github.user/tool");
        assert_eq!(version, Some("1.0.1"));
    }

    #[test]
    fn parse_registry_ref_without_version() {
        let (name, version) = parse_registry_ref("io.github.user/tool");
        assert_eq!(name, "io.github.user/tool");
        assert_eq!(version, None);
    }

    #[test]
    fn derive_tool_name_extracts_suffix() {
        assert_eq!(derive_tool_name("io.github.user/my-tool"), "my-tool");
        assert_eq!(
            derive_tool_name("ai.filegraph/document-processing"),
            "document-processing"
        );
    }

    #[test]
    fn synthesize_command_npm() {
        let pkg = Package {
            registry_type: "npm".into(),
            identifier: "@modelcontextprotocol/server-brave-search".into(),
            version: Some("1.0.2".into()),
            runtime_hint: Some("npx".into()),
            transport: Transport {
                transport_type: "stdio".into(),
            },
            environment_variables: vec![],
        };
        let cmd = synthesize_command(&pkg).unwrap();
        assert_eq!(
            cmd,
            vec![
                "npx",
                "-y",
                "@modelcontextprotocol/server-brave-search@1.0.2"
            ]
        );
    }

    #[test]
    fn synthesize_command_pypi() {
        let pkg = Package {
            registry_type: "pypi".into(),
            identifier: "mcp-stdio".into(),
            version: Some("0.4.7".into()),
            runtime_hint: Some("uvx".into()),
            transport: Transport {
                transport_type: "stdio".into(),
            },
            environment_variables: vec![],
        };
        let cmd = synthesize_command(&pkg).unwrap();
        assert_eq!(cmd, vec!["uvx", "mcp-stdio==0.4.7"]);
    }

    #[test]
    fn synthesize_command_oci() {
        let pkg = Package {
            registry_type: "oci".into(),
            identifier: "docker.io/user/tool:1.0".into(),
            version: Some("1.0".into()),
            runtime_hint: Some("docker".into()),
            transport: Transport {
                transport_type: "stdio".into(),
            },
            environment_variables: vec![],
        };
        let cmd = synthesize_command(&pkg).unwrap();
        assert_eq!(
            cmd,
            vec!["docker", "run", "--rm", "-i", "docker.io/user/tool:1.0"]
        );
    }

    #[test]
    fn synthesize_command_unsupported() {
        let pkg = Package {
            registry_type: "mcpb".into(),
            identifier: "https://github.com/user/tool/releases/download/v1.0/tool.mcpb".into(),
            version: None,
            runtime_hint: None,
            transport: Transport {
                transport_type: "stdio".into(),
            },
            environment_variables: vec![],
        };
        assert!(synthesize_command(&pkg).is_err());
    }

    #[test]
    fn synthesize_tool_from_packages() {
        let server = ServerJson {
            name: "io.github.user/my-tool".into(),
            description: Some("A test tool".into()),
            title: None,
            version: Some("1.0.0".into()),
            packages: vec![Package {
                registry_type: "npm".into(),
                identifier: "@user/my-tool".into(),
                version: Some("1.0.0".into()),
                runtime_hint: None,
                transport: Transport {
                    transport_type: "stdio".into(),
                },
                environment_variables: vec![EnvVar {
                    name: "API_KEY".into(),
                    description: Some("Your API key".into()),
                    is_required: true,
                    is_secret: true,
                    value: None,
                    default: None,
                }],
            }],
            remotes: vec![],
            repository: None,
        };
        let tool = synthesize_tool(&server).unwrap();
        assert_eq!(tool.name, "my-tool");
        assert_eq!(
            tool.command,
            Some(vec![
                "npx".into(),
                "-y".into(),
                "@user/my-tool@1.0.0".into()
            ])
        );
        assert!(tool.url.is_none());
        assert_eq!(tool.env_vars.len(), 1);
        assert_eq!(tool.env_vars[0].name, "API_KEY");
    }

    #[test]
    fn synthesize_tool_from_remotes() {
        let server = ServerJson {
            name: "ai.filegraph/document-processing".into(),
            description: Some("Doc processing".into()),
            title: None,
            version: Some("1.0.1".into()),
            packages: vec![],
            remotes: vec![Remote {
                remote_type: "sse".into(),
                url: "https://api.filegraph.ai/mcp".into(),
                headers: vec![],
            }],
            repository: None,
        };
        let tool = synthesize_tool(&server).unwrap();
        assert_eq!(tool.name, "document-processing");
        assert!(tool.command.is_none());
        assert_eq!(tool.url, Some("https://api.filegraph.ai/mcp".into()));
    }

    #[test]
    fn deserialize_real_server_json_stdio() {
        let json = r#"{
            "server": {
                "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
                "name": "io.github.shigechika/mcp-stdio",
                "description": "Stdio-to-HTTP gateway",
                "title": "MCP Stdio",
                "version": "0.4.7",
                "repository": { "url": "https://github.com/shigechika/mcp-stdio", "source": "github" },
                "packages": [{
                    "registryType": "pypi",
                    "registryBaseUrl": "https://pypi.org",
                    "identifier": "mcp-stdio",
                    "version": "0.4.7",
                    "runtimeHint": "uvx",
                    "transport": { "type": "stdio" }
                }]
            },
            "_meta": {
                "io.modelcontextprotocol.registry/official": {
                    "status": "active",
                    "isLatest": true
                }
            }
        }"#;
        let resp: ServerResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.server.name, "io.github.shigechika/mcp-stdio");
        assert_eq!(resp.server.packages.len(), 1);
        assert_eq!(resp.server.packages[0].registry_type, "pypi");
        assert_eq!(resp.server.packages[0].runtime_hint, Some("uvx".into()));

        let tool = synthesize_tool(&resp.server).unwrap();
        assert_eq!(
            tool.command,
            Some(vec!["uvx".into(), "mcp-stdio==0.4.7".into()])
        );
    }

    #[test]
    fn deserialize_real_server_json_remote() {
        let json = r#"{
            "server": {
                "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
                "name": "ai.filegraph/document-processing",
                "description": "Extract text from documents",
                "version": "1.0.1",
                "repository": { "url": "https://github.com/filegraph/docconvert", "source": "github" },
                "remotes": [{ "type": "sse", "url": "https://api.filegraph.ai/mcp" }]
            },
            "_meta": {}
        }"#;
        let resp: ServerResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.server.name, "ai.filegraph/document-processing");
        assert_eq!(resp.server.remotes.len(), 1);

        let tool = synthesize_tool(&resp.server).unwrap();
        assert!(tool.command.is_none());
        assert_eq!(tool.url, Some("https://api.filegraph.ai/mcp".into()));
    }
}
