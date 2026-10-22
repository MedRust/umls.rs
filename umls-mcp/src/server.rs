//! MCP adapters for the existing UMLS client.
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{num::NonZeroU64, sync::Arc};
use umls_client::{atom, concept::Info, config::Config, core::cui::Cui, search};
use umls_http::reqwest::Client;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SearchArgs {
    /// Nonempty term to search, for example diabetes.
    query: String,
    /// One-based page number; defaults to 1.
    page: Option<u64>,
    /// Results per page, from 1 to 100; defaults to 25.
    page_size: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ConceptArgs {
    /// CUI consisting of C followed by seven digits, for example C0010200.
    cui: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct AtomArgs {
    /// Concept Unique Identifier, for example C0010200.
    cui: String,
    /// One-based page number; defaults to 1.
    page: Option<u64>,
    /// Results per page, from 1 to 100; defaults to 25.
    page_size: Option<u64>,
}

#[derive(Clone)]
pub(crate) struct UmlsServer {
    config: Arc<Config>,
    client: Client,
}

fn pagination(
    page: Option<u64>,
    size: Option<u64>,
) -> Result<(NonZeroU64, NonZeroU64), &'static str> {
    let page = NonZeroU64::new(page.unwrap_or(1)).ok_or("page must be at least 1")?;
    let size = size.unwrap_or(25);
    if !(1..=100).contains(&size) {
        return Err("page_size must be between 1 and 100");
    }
    Ok((page, NonZeroU64::new(size).unwrap()))
}

fn failure(message: &str) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message)])
}

fn success(value: Value) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(value.to_string())])
}

#[tool_router]
impl UmlsServer {
    pub(crate) fn new(config: Config, client: Client) -> Self {
        Self {
            config: Arc::new(config),
            client,
        }
    }

    #[tool(
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        ),
        description = "Search UMLS concepts by term. Returns one page of concept identifiers and names; use get_concept for details."
    )]
    async fn search_concepts(&self, Parameters(args): Parameters<SearchArgs>) -> CallToolResult {
        let Ok(term) = args.query.parse() else {
            return failure("query must not be empty");
        };
        let (page, size) = match pagination(args.page, args.page_size) {
            Ok(value) => value,
            Err(message) => return failure(message),
        };
        let response = search::Request::builder(term)
            .page_number(page)
            .page_size(size)
            .concept(&self.config)
            .with_client(self.client.clone())
            .get()
            .await;
        let Ok(response) = response else {
            return failure(
                "UMLS search failed; check credentials, service availability, and release configuration",
            );
        };
        let Ok(page) = response.into_page() else {
            return failure("UMLS returned invalid concept search results");
        };
        let results: Vec<_> = page
            .content()
            .list()
            .iter()
            .map(|item| {
                json!({
                    "cui": item.cui(), "name": item.name(), "root_source": item.root_source(),
                })
            })
            .collect();
        success(
            json!({"results": results, "page": page.number().get(), "page_size": page.size().get(),
            "total_pages": page.total_pages().map(NonZeroU64::get), "has_next": page.has_next(),
            "version": self.config.version().as_str()}),
        )
    }

    #[tool(
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        ),
        description = "Retrieve UMLS concept details by CUI, including name, status, semantic types, and atom count."
    )]
    async fn get_concept(&self, Parameters(args): Parameters<ConceptArgs>) -> CallToolResult {
        let Ok(cui) = Cui::new(&args.cui) else {
            return failure("cui must be C followed by seven digits");
        };
        let response = Info::new(cui)
            .with_client(self.client.clone())
            .fetch(&self.config)
            .await;
        let Ok(value) = response else {
            return failure(
                "UMLS concept lookup failed; check CUI, credentials, service availability, and release configuration",
            );
        };
        success(
            json!({"cui": value.ui, "name": value.name, "status": value.status,
            "suppressible": value.suppressible, "atom_count": value.atom_count,
            "relation_count": value.relation_count, "date_added": value.date_added,
            "major_revision_date": value.major_revision_date,
            "semantic_types": value.semantic_types.iter().map(|item| &item.name).collect::<Vec<_>>(),
            "version": self.config.version().as_str()}),
        )
    }

    #[tool(
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        ),
        description = "Retrieve one page of source vocabulary atoms for a UMLS CUI, excluding obsolete and suppressible atoms."
    )]
    async fn get_concept_atoms(&self, Parameters(args): Parameters<AtomArgs>) -> CallToolResult {
        let Ok(cui) = Cui::new(&args.cui) else {
            return failure("cui must be C followed by seven digits");
        };
        let (number, size) = match pagination(args.page, args.page_size) {
            Ok(value) => value,
            Err(message) => return failure(message),
        };
        let response = atom::Config::builder(cui)
            .page_number(number)
            .page_size(size)
            .atoms::<atom::All>()
            .with_client(self.client.clone())
            .fetch(&self.config)
            .await;
        let Ok(response) = response else {
            return failure(
                "UMLS atom lookup failed; check CUI, credentials, service availability, and release configuration",
            );
        };
        let page = response.into_page();
        let results: Vec<_> = page.content().list().iter().map(|item| json!({
            "aui": item.ui.as_ref(), "name": item.name.as_ref(), "root_source": item.root_source.as_ref(),
            "term_type": item.term_type.as_ref(), "language": item.language.as_ref(),
            "obsolete": item.obsolete, "suppressible": item.suppressible,
        })).collect();
        success(
            json!({"results": results, "page": page.number().get(), "page_size": page.size().get(),
            "total_pages": page.total_pages().map(NonZeroU64::get), "has_next": page.has_next(),
            "version": self.config.version().as_str()}),
        )
    }
}

#[tool_handler(
    name = "umls-mcp",
    version = "0.1.0",
    instructions = "Search UMLS terminology and retrieve concepts or source atoms. Use CUIs from search results for lookups. Paginated tools fetch one page per call."
)]
impl rmcp::ServerHandler for UmlsServer {}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    fn server(base_url: &str) -> UmlsServer {
        let config = Config::builder("secret-test-key")
            .base_url(base_url.parse().unwrap())
            .build()
            .unwrap();
        UmlsServer::new(config, Client::default())
    }

    fn body(result: CallToolResult) -> Value {
        assert!(!result.is_error.unwrap_or(false));
        serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap()
    }

    #[tokio::test]
    async fn search_preserves_prefix_and_returns_page_metadata() {
        let mock_server = MockServer::start_async().await;
        let mock = mock_server
            .mock_async(|when, then| {
                when.method(GET)
                    .path("/rest/search/current")
                    .query_param("apiKey", "secret-test-key")
                    .query_param("string", "cough")
                    .query_param("pageNumber", "2")
                    .query_param("pageSize", "1");
                then.status(200)
                    .json_body(json!({"pageNumber": 2, "pageSize": 1,
                "result": {"classType": "searchResults", "recCount": 3, "results": [{"ui": "C0010200", "name": "Cough",
                "rootSource": "SNOMEDCT_US", "semanticTypes": ["Sign or Symptom"], "uri": "https://example.com/concept"}]}}));
            })
            .await;
        let result = server(&format!("{}/rest/", mock_server.base_url()))
            .search_concepts(Parameters(SearchArgs {
                query: "cough".into(),
                page: Some(2),
                page_size: Some(1),
            }))
            .await;
        let value = body(result);
        assert_eq!(value["results"][0]["cui"], "C0010200");
        assert_eq!(value["page"], 2);
        assert_eq!(value["total_pages"], 3);
        assert_eq!(value["has_next"], true);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn invalid_inputs_do_not_reach_upstream() {
        let server = server("http://127.0.0.1:1/rest/");
        for (page, page_size) in [(Some(0), None), (None, Some(0)), (None, Some(101))] {
            let result = server
                .search_concepts(Parameters(SearchArgs {
                    query: "cough".into(),
                    page,
                    page_size,
                }))
                .await;
            assert_eq!(result.is_error, Some(true));
        }
        let result = server
            .get_concept(Parameters(ConceptArgs {
                cui: "invalid".into(),
            }))
            .await;
        assert_eq!(result.is_error, Some(true));
    }

    #[tokio::test]
    async fn mcp_handshake_discovers_tools_and_routes_validation_errors() {
        use rmcp::{ServiceExt, model::CallToolRequestParams};
        let (client_io, server_io) = tokio::io::duplex(8192);
        let task = tokio::spawn(async move {
            server("http://127.0.0.1:1/rest/")
                .serve(server_io)
                .await
                .unwrap()
        });
        let client = ().serve(client_io).await.unwrap();
        let server = task.await.unwrap();
        let tools = client.list_all_tools().await.unwrap();
        let names: Vec<_> = tools.iter().map(|tool| tool.name.as_ref()).collect();
        for name in ["search_concepts", "get_concept", "get_concept_atoms"] {
            assert!(names.contains(&name));
        }
        let response = client
            .call_tool(
                CallToolRequestParams::new("get_concept")
                    .with_arguments(json!({"cui": "invalid"}).as_object().unwrap().clone()),
            )
            .await
            .unwrap();
        assert_eq!(response.is_error, Some(true));
        client.cancel().await.unwrap();
        server.cancel().await.unwrap();
    }

    #[tokio::test]
    async fn atoms_return_source_terms_and_page_count() {
        let mock_server = MockServer::start_async().await;
        let mock = mock_server.mock_async(|when, then| {
            when.method(GET).path("/rest/content/current/CUI/C0010200/atoms")
                .query_param("pageSize", "1");
            then.status(200).json_body(json!({"pageNumber": 1, "pageSize": 1, "pageCount": 2,
                "result": [{"ui": "A1234567", "name": "Cough", "rootSource": "MSH",
                    "termType": "MH", "language": "ENG", "obsolete": "false", "suppressible": "false"}]}));
        }).await;
        let result = server(&format!("{}/rest/", mock_server.base_url()))
            .get_concept_atoms(Parameters(AtomArgs {
                cui: "C0010200".into(),
                page: None,
                page_size: Some(1),
            }))
            .await;
        let value = body(result);
        assert_eq!(value["results"][0]["root_source"], "MSH");
        assert_eq!(value["total_pages"], 2);
        assert_eq!(value["has_next"], true);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn concept_details_use_configured_release() {
        let mock_server = MockServer::start_async().await;
        let mock = mock_server.mock_async(|when, then| {
            when.method(GET).path("/rest/content/2024AB/CUI/C0010200");
            then.status(200).json_body(json!({"result": {
                "classType": "Concept", "ui": "C0010200", "name": "Cough",
                "suppressible": false, "status": "R", "dateAdded": "01-01-1990",
                "majorRevisionDate": "01-01-2024", "semanticTypes": [{"name": "Sign or Symptom", "uri": "NONE"}],
                "atomCount": 2, "attributeCount": 0, "cvMemberCount": 0, "relationCount": 1,
                "atoms": format!("{}/content/current/CUI/C0155502/atoms", mock_server.base_url()), "definitions": "NONE", "relations": "NONE", "defaultPreferredAtom": format!("{}/content/current/CUI/C0155502/atoms/preferred", mock_server.base_url())
            }}));
        }).await;
        let config = Config::builder("secret-test-key")
            .try_version("2024AB")
            .unwrap()
            .base_url(format!("{}/rest/", mock_server.base_url()).parse().unwrap())
            .build()
            .unwrap();
        let server = UmlsServer::new(config, Client::default());
        let value = body(
            server
                .get_concept(Parameters(ConceptArgs {
                    cui: "C0010200".into(),
                }))
                .await,
        );
        assert_eq!(value["name"], "Cough");
        assert_eq!(value["semantic_types"][0], "Sign or Symptom");
        assert_eq!(value["version"], "2024AB");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn upstream_errors_do_not_expose_credentials() {
        let mock_server = MockServer::start_async().await;
        let mock = mock_server
            .mock_async(|when, then| {
                when.method(GET).path("/rest/content/current/CUI/C0010200");
                then.status(401).body("secret-test-key");
            })
            .await;
        let result = server(&format!("{}/rest/", mock_server.base_url()))
            .get_concept(Parameters(ConceptArgs {
                cui: "C0010200".into(),
            }))
            .await;
        assert_eq!(result.is_error, Some(true));
        assert!(
            !serde_json::to_string(&result)
                .unwrap()
                .contains("secret-test-key")
        );
        mock.assert_async().await;
    }
}
