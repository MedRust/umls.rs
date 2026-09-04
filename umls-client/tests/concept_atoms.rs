//! Integration tests for concept-atom retrieval and pagination.

use httpmock::{Method, MockServer};
use serde_json::json;
use umls_client::{
    atom::{self, All, Atom, Error},
    concept::cui::Cui,
    config,
};
use url::Url;
use uuid::Uuid;

#[tokio::test]
async fn fetch_first_page_content_of_concept_atoms() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let api_key = Uuid::new_v4();
    let cui = Cui::new("C0010200")?;

    let mock_server = MockServer::start_async().await;

    mock_server
        .mock_async(|when, then| {
            let path = format!("/content/current/CUI/{}/atoms", cui);

            when.method(Method::GET)
                .path(path)
                .query_param("apiKey", api_key.to_string());
            then.status(200)
                .json_body(json!({
                    "pageSize": 1,
                    "pageNumber": 1,
                    "pageCount": 55,
                    "result": [
                        {
                            "classType": "Atom",
                            "ui": "A24123438",
                            "sourceDescriptor": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/D065635",
                            "sourceConcept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/M0529892",
                            "concept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0155502",
                            "suppressible": "false",
                            "obsolete": "false",
                            "rootSource": "MSH",
                            "termType": "MH",
                            "code": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/D065635",
                            "language": "ENG",
                            "name": "Benign Paroxysmal Positional Vertigo",
                            "ancestors": "NONE",
                            "descendants": "NONE",
                            "attributes": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/AUI/A24123438/attributes",
                            "relations": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/AUI/A24123438/relations",
                            "children": "NONE",
                            "parents": "NONE"
                        }
                    ]
                }));
        })
        .await;

    let base_url = mock_server.base_url().parse()?;

    let umls_config = config::Builder::new(api_key.to_string())
        .base_url(base_url)
        .version("current".into())
        .build()?;

    let content = atom::Config::builder(cui)
        .atoms(&umls_config)
        .fetch()
        .await?
        .into_content();

    assert_eq!(content.list().len(), 1);

    let first = content.list().first().expect("Should contain first item");

    assert_eq!(first.ui.as_str(), "A24123438");
    assert!(!first.suppressible);
    assert!(!first.obsolete);
    assert_eq!(first.root_source.as_str(), "MSH");
    assert_eq!(first.term_type.as_str(), "MH");
    assert_eq!(first.language.as_str(), "ENG");
    assert_eq!(first.name.as_str(), "Benign Paroxysmal Positional Vertigo");

    Ok(())
}

#[tokio::test]
async fn reports_atom_url_errors() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Builder::new(Uuid::new_v4().to_string())
        .base_url("mailto:umls@example.com".parse()?)
        .build()?;
    let cui = Cui::new("C0010200")?;

    let error = match atom::Config::builder(cui).atoms(&config).fetch().await {
        Ok(_) => panic!("a non-base URL cannot be joined with the atom path"),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Url(_)));
    Ok(())
}

#[tokio::test]
async fn reports_unsuccessful_atom_statuses() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    mock_server
        .mock_async(|when, then| {
            when.method(Method::GET);
            then.status(503);
        })
        .await;

    let config = config::Builder::new(Uuid::new_v4().to_string())
        .base_url(mock_server.base_url().parse()?)
        .build()?;
    let cui = Cui::new("C0010200")?;

    let error = match atom::Config::builder(cui).atoms(&config).fetch().await {
        Ok(_) => panic!("a 503 response must be reported as an HTTP status error"),
        Err(error) => error,
    };

    assert!(matches!(error, Error::HttpStatus(_)));
    Ok(())
}

#[tokio::test]
async fn reports_invalid_atom_response_bodies() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    mock_server
        .mock_async(|when, then| {
            when.method(Method::GET);
            then.status(200).body("not JSON");
        })
        .await;

    let config = config::Builder::new(Uuid::new_v4().to_string())
        .base_url(mock_server.base_url().parse()?)
        .build()?;
    let cui = Cui::new("C0010200")?;

    let error = match atom::Config::builder(cui).atoms(&config).fetch().await {
        Ok(_) => panic!("an invalid response body must be reported as a decode error"),
        Err(error) => error,
    };

    assert!(matches!(error, Error::Decode(_)));
    Ok(())
}

#[tokio::test]
async fn fetch_first_page_of_concept_atoms() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let api_key = Uuid::new_v4();
    let cui = Cui::new("C0010200")?;

    let mock_server = MockServer::start_async().await;

    mock_server
        .mock_async(|when, then| {
            let path = format!("/content/current/CUI/{}/atoms", cui);

            when.method(Method::GET)
                .path(path)
                .query_param("apiKey", api_key.to_string())
                .query_param("pageNumber", "1");
            then.status(200)
                .json_body(json!({
                    "pageSize": 1,
                    "pageNumber": 1,
                    "pageCount": 55,
                    "result": [
                        {
                            "classType": "Atom",
                            "ui": "A24123438",
                            "sourceDescriptor": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/D065635",
                            "sourceConcept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/M0529892",
                            "concept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0155502",
                            "suppressible": "false",
                            "obsolete": "false",
                            "rootSource": "MSH",
                            "termType": "MH",
                            "code": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/D065635",
                            "language": "ENG",
                            "name": "Benign Paroxysmal Positional Vertigo",
                            "ancestors": "NONE",
                            "descendants": "NONE",
                            "attributes": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/AUI/A24123438/attributes",
                            "relations": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/AUI/A24123438/relations",
                            "children": "NONE",
                            "parents": "NONE"
                        }
                    ]
                }));
        })
        .await;

    mock_server
        .mock_async(|when, then| {
            let path = format!("/content/current/CUI/{}/atoms", cui);

            when.method(Method::GET)
                .path(path)
                .query_param("apiKey", api_key.to_string())
                .query_param("pageNumber", "2");
            then.status(200).json_body(json!({
                "pageSize": 1,
                "pageNumber": 2,
                "pageCount": 54,
                "result": [
                    {
                        "classType": "Atom",
                        "ui": "A24327380",
                        "sourceDescriptor": "NONE",
                        "sourceConcept": "NONE",
                        "concept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0155502",
                        "suppressible": "false",
                        "obsolete": "false",
                        "rootSource": "MTH",
                        "termType": "PN",
                        "code": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MTH/NOCODE",
                        "language": "ENG",
                        "name": "Benign Paroxysmal Positional Vertigo",
                        "ancestors": "NONE",
                        "descendants": "NONE",
                        "attributes": "NONE",
                        "relations": "NONE",
                        "children": "NONE",
                        "parents": "NONE"
                    }
                ]
            }));
        })
        .await;

    let base_url = mock_server.base_url().parse()?;

    let umls_config = config::Builder::new(api_key.to_string())
        .base_url(base_url)
        .version("current".into())
        .build()?;

    let first_page = atom::Config::builder(cui)
        .atoms(&umls_config)
        .fetch()
        .await?
        .into_page();

    let content = first_page.content();

    assert_eq!(content.list().len(), 1);

    let first = content.list().first().expect("Should contain first item");

    assert_eq!(first.ui.as_str(), "A24123438");
    assert!(!first.suppressible);
    assert!(!first.obsolete);
    assert_eq!(first.root_source.as_str(), "MSH");
    assert_eq!(first.term_type.as_str(), "MH");
    assert_eq!(first.language.as_str(), "ENG");
    assert_eq!(first.name.as_str(), "Benign Paroxysmal Positional Vertigo");

    let second_page = first_page.next()?.fetch().await?.into_page();

    let content = second_page
        .content()
        .list()
        .first()
        .expect("Should contain first item");

    assert_eq!(content.ui.as_str(), "A24327380");
    assert!(!content.suppressible);
    assert!(!content.obsolete);
    assert_eq!(content.root_source.as_str(), "MTH");
    assert_eq!(content.language.as_str(), "ENG");
    assert_eq!(content.term_type.as_str(), "PN");
    assert_eq!(
        content.name.as_str(),
        "Benign Paroxysmal Positional Vertigo"
    );

    Ok(())
}

#[tokio::test]
async fn fetch_first_page_content_of_concept_atoms_using_url()
-> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let api_key = Uuid::new_v4();
    let cui = Cui::new("C0010200")?;

    let mock_server = MockServer::start_async().await;

    mock_server
        .mock_async(|when, then| {
            let path = format!("/content/current/CUI/{}/atoms", cui);

            when.method(Method::GET)
                .path(path)
                .query_param("apiKey", api_key.to_string());
            then.status(200)
                .json_body(json!({
                    "pageSize": 1,
                    "pageNumber": 1,
                    "pageCount": 55,
                    "result": [
                        {
                            "classType": "Atom",
                            "ui": "A24123438",
                            "sourceDescriptor": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/D065635",
                            "sourceConcept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/M0529892",
                            "concept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0155502",
                            "suppressible": "false",
                            "obsolete": "false",
                            "rootSource": "MSH",
                            "termType": "MH",
                            "code": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/D065635",
                            "language": "ENG",
                            "name": "Benign Paroxysmal Positional Vertigo",
                            "ancestors": "NONE",
                            "descendants": "NONE",
                            "attributes": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/AUI/A24123438/attributes",
                            "relations": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/AUI/A24123438/relations",
                            "children": "NONE",
                            "parents": "NONE"
                        }
                    ]
                }));
        })
        .await;

    let base_url: Url = mock_server.base_url().parse()?;

    let umls_config = config::Builder::new(api_key.to_string())
        .base_url(base_url.clone())
        .version("current".into())
        .build()?;

    let path = format!("/content/current/CUI/{}/atoms", cui);
    let url = base_url.join(&path)?;

    let content = atom::Config::builder(url)
        .atoms(&umls_config)
        .fetch()
        .await?
        .into_content();

    assert_eq!(content.list().len(), 1);

    let first = content.list().first().expect("Should contain first item");

    assert_eq!(first.ui.as_str(), "A24123438");
    assert!(!first.suppressible);
    assert!(!first.obsolete);
    assert_eq!(first.root_source.as_str(), "MSH");
    assert_eq!(first.term_type.as_str(), "MH");
    assert_eq!(first.language.as_str(), "ENG");
    assert_eq!(first.name.as_str(), "Benign Paroxysmal Positional Vertigo");

    Ok(())
}

#[tokio::test]
async fn fetch_first_page_of_concept_atoms_using_url() -> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let api_key = Uuid::new_v4();
    let cui = Cui::new("C0010200")?;

    let mock_server = MockServer::start_async().await;

    mock_server
        .mock_async(|when, then| {
            let path = format!("/content/current/CUI/{}/atoms", cui);

            when.method(Method::GET)
                .path(path)
                .query_param("apiKey", api_key.to_string())
                .query_param("pageNumber", "1");
            then.status(200)
                .json_body(json!({
                    "pageSize": 1,
                    "pageNumber": 1,
                    "pageCount": 55,
                    "result": [
                        {
                            "classType": "Atom",
                            "ui": "A24123438",
                            "sourceDescriptor": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/D065635",
                            "sourceConcept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/M0529892",
                            "concept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0155502",
                            "suppressible": "false",
                            "obsolete": "false",
                            "rootSource": "MSH",
                            "termType": "MH",
                            "code": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/D065635",
                            "language": "ENG",
                            "name": "Benign Paroxysmal Positional Vertigo",
                            "ancestors": "NONE",
                            "descendants": "NONE",
                            "attributes": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/AUI/A24123438/attributes",
                            "relations": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/AUI/A24123438/relations",
                            "children": "NONE",
                            "parents": "NONE"
                        }
                    ]
                }));
        })
        .await;

    mock_server
        .mock_async(|when, then| {
            let path = format!("/content/current/CUI/{}/atoms", cui);

            when.method(Method::GET)
                .path(path)
                .query_param("apiKey", api_key.to_string())
                .query_param("pageNumber", "2");
            then.status(200).json_body(json!({
                "pageSize": 1,
                "pageNumber": 2,
                "pageCount": 54,
                "result": [
                    {
                        "classType": "Atom",
                        "ui": "A24327380",
                        "sourceDescriptor": "NONE",
                        "sourceConcept": "NONE",
                        "concept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0155502",
                        "suppressible": "false",
                        "obsolete": "false",
                        "rootSource": "MTH",
                        "termType": "PN",
                        "code": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MTH/NOCODE",
                        "language": "ENG",
                        "name": "Benign Paroxysmal Positional Vertigo",
                        "ancestors": "NONE",
                        "descendants": "NONE",
                        "attributes": "NONE",
                        "relations": "NONE",
                        "children": "NONE",
                        "parents": "NONE"
                    }
                ]
            }));
        })
        .await;

    let base_url: Url = mock_server.base_url().parse()?;

    let umls_config = config::Builder::new(api_key.to_string())
        .base_url(base_url.clone())
        .version("current".into())
        .build()?;

    let path = format!("/content/current/CUI/{}/atoms", cui);
    let url = base_url.join(&path)?;

    let first_page = atom::Config::builder(url)
        .atoms(&umls_config)
        .fetch()
        .await?
        .into_page();

    let content = first_page.content();

    assert_eq!(content.list().len(), 1);

    let first = content.list().first().expect("Should contain first item");

    assert_eq!(first.ui.as_str(), "A24123438");
    assert!(!first.suppressible);
    assert!(!first.obsolete);
    assert_eq!(first.root_source.as_str(), "MSH");
    assert_eq!(first.term_type.as_str(), "MH");
    assert_eq!(first.language.as_str(), "ENG");
    assert_eq!(first.name.as_str(), "Benign Paroxysmal Positional Vertigo");

    let second_page = first_page.next()?.fetch().await?.into_page();

    let content = second_page
        .content()
        .list()
        .first()
        .expect("Should contain first item");

    assert_eq!(content.ui.as_str(), "A24327380");
    assert!(!content.suppressible);
    assert!(!content.obsolete);
    assert_eq!(content.root_source.as_str(), "MTH");
    assert_eq!(content.language.as_str(), "ENG");
    assert_eq!(content.term_type.as_str(), "PN");
    assert_eq!(
        content.name.as_str(),
        "Benign Paroxysmal Positional Vertigo"
    );

    Ok(())
}

#[tokio::test]
async fn fetch_first_page_content_of_concept_atoms_using_cui()
-> Result<(), Box<dyn std::error::Error>> {
    _ = env_logger::builder().is_test(true).try_init();

    let api_key = Uuid::new_v4();
    let cui = Cui::new("C0010200")?;

    let mock_server = MockServer::start_async().await;

    mock_server
        .mock_async(|when, then| {
            let path = format!("/content/current/CUI/{}/atoms", cui);

            when.method(Method::GET)
                .path(path)
                .query_param("apiKey", api_key.to_string());
            then.status(200)
                .json_body(json!({
                    "pageSize": 1,
                    "pageNumber": 1,
                    "pageCount": 55,
                    "result": [
                        {
                            "classType": "Atom",
                            "ui": "A24123438",
                            "sourceDescriptor": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/D065635",
                            "sourceConcept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/M0529892",
                            "concept": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/CUI/C0155502",
                            "suppressible": "false",
                            "obsolete": "false",
                            "rootSource": "MSH",
                            "termType": "MH",
                            "code": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/source/MSH/D065635",
                            "language": "ENG",
                            "name": "Benign Paroxysmal Positional Vertigo",
                            "ancestors": "NONE",
                            "descendants": "NONE",
                            "attributes": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/AUI/A24123438/attributes",
                            "relations": "https://uts-ws.nlm.nih.gov/rest/content/2026AA/AUI/A24123438/relations",
                            "children": "NONE",
                            "parents": "NONE"
                        }
                    ]
                }));
        })
        .await;

    let base_url: Url = mock_server.base_url().parse()?;

    let umls_config = config::Builder::new(api_key.to_string())
        .base_url(base_url)
        .version("current".into())
        .build()?;

    let content = Atom::<'_, All, Cui>::new(&umls_config, cui).fetch().await?;

    assert_eq!(content.list().len(), 1);

    let first = content.list().first().expect("Should contain first item");

    assert_eq!(first.ui.as_str(), "A24123438");
    assert!(!first.suppressible);
    assert!(!first.obsolete);
    assert_eq!(first.root_source.as_str(), "MSH");
    assert_eq!(first.term_type.as_str(), "MH");
    assert_eq!(first.language.as_str(), "ENG");
    assert_eq!(first.name.as_str(), "Benign Paroxysmal Positional Vertigo");

    Ok(())
}
