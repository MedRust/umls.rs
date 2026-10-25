//! An integration tests for Parent API's.
#![cfg(feature = "reqwest")]

use httpmock::{Method::GET, MockServer};
use umls_client::atom::Batch;
use umls_client::parent::{self, Error, SourceConcept};
use umls_client::{config, core::aui::Aui, parent::Parent, search::core::DisplayName};
use url::Url;

#[tokio::test]
async fn fetch_atom_parents_using_url() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    let mock = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/AUI/A10134087/parents")
                .query_param_exists("apiKey");

            let value = include_str!("fixtures/atom_parents.json");
            let body: serde_json::Value = serde_json::from_str(value).unwrap();
            then.status(200).json_body(body);
        })
        .await;

    let client = umls_http::reqwest::Client::default();

    let base_url: Url = mock_server.base_url().parse()?;
    let url = base_url.join("content/current/AUI/A10134087/parents")?;

    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .version(config::Version::Current)
        .build()?;

    let pages = Parent::<parent::types::Atom, _, _>::new(url)
        .with_client(client)
        .fetch(&umls_config)
        .await?;

    assert_eq!(pages.size().get(), 25);
    assert_eq!(pages.number().get(), 1);
    assert_eq!(pages.content().list().len(), 1);

    let response = pages.content().list().first().unwrap();
    assert_eq!(response.ui.as_str(), "A19720766");
    assert!(!response.suppressible);
    assert!(!response.obsolete);
    assert_eq!(response.root_source.as_str(), "HL7V3.0");
    assert_eq!(response.term_type.as_str(), "CSY");
    assert_eq!(response.language.as_str(), "ENG");
    assert_eq!(response.name.as_str(), "RouteOfAdministration");

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn validation_url_object() -> Result<(), Box<dyn std::error::Error>> {
    let client = umls_http::reqwest::Client::default();

    let base_url: Url = "https://uts-ws.nlm.nih.gov/rest/".parse()?;
    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .build()?;

    let urls: Vec<Url> = vec![
        "http://localhost:8080/rest/content/current/AUI/A10134087/parents".parse()?,
        "https://localhost:8080/rest/content/current/AUI/A10134087/parents".parse()?,
        "https://uts-ws.nlm.nih.gov:8080/rest/content/current/AUI/A10134087/parents".parse()?,
    ];

    for url in urls {
        let result = Parent::<parent::types::Atom, _, _>::new(url)
            .with_client(client.clone())
            .fetch(&umls_config)
            .await;

        assert!(matches!(
            result,
            Err(Error::<umls_http::reqwest::Error>::Validation(_))
        ));
    }
    Ok(())
}

#[tokio::test]
async fn fetch_atom_parents_using_aui() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    let mock = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/AUI/A10134087/parents")
                .query_param_exists("apiKey");

            let value = include_str!("fixtures/atom_parents.json");
            let body: serde_json::Value = serde_json::from_str(value).unwrap();
            then.status(200).json_body(body);
        })
        .await;

    let base_url: Url = mock_server.base_url().parse()?;

    let client = umls_http::reqwest::Client::default();
    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .version(config::Version::Current)
        .build()?;

    let aui: DisplayName = "A10134087".parse()?;
    let aui = Aui::new(aui)?;

    let pages = Parent::new(aui)
        .with_client(client)
        .fetch(&umls_config)
        .await?;

    assert_eq!(pages.size().get(), 25);
    assert_eq!(pages.number().get(), 1);
    assert_eq!(pages.content().list().len(), 1);

    let response = pages.content().list().first().unwrap();
    assert_eq!(response.ui.as_str(), "A19720766");
    assert!(!response.suppressible);
    assert!(!response.obsolete);
    assert_eq!(response.root_source.as_str(), "HL7V3.0");
    assert_eq!(response.term_type.as_str(), "CSY");
    assert_eq!(response.language.as_str(), "ENG");
    assert_eq!(response.name.as_str(), "RouteOfAdministration");

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn fetch_source_asserted_parents_using_url() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    let mock = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/source/SNOMEDCT_US/9468002/parents")
                .query_param_exists("apiKey");
            let json_string = include_str!("fixtures/source_conecpt_parent_page_1.json");
            let json: serde_json::Value = serde_json::from_str(json_string).unwrap();
            then.status(200).json_body(json);
        })
        .await;

    let base_url: Url = mock_server.base_url().parse()?;

    let url = base_url.join("content/current/source/SNOMEDCT_US/9468002/parents")?;

    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .build()?;

    let page = Parent::<parent::types::Source, _, _>::new(url)
        .with_client(Default::default())
        .fetch(&umls_config)
        .await?;

    assert_eq!(page.size().get(), 1);
    assert_eq!(page.number().get(), 1);
    assert_eq!(page.count().get(), 2);
    assert!(!page.content().list().is_empty());
    assert_eq!(page.content().list().len(), 1);

    let object = page.content().list().first().unwrap();
    assert_eq!(object.ui().as_str(), "82065001");

    mock.assert_async().await;

    Ok(())
}

#[tokio::test]
async fn fetch_source_asserted_concepts_using_batch_forward_pagination()
-> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    let mock_1 = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/source/SNOMEDCT_US/9468002/parents")
                .query_param_exists("apiKey")
                .query_param("pageSize", "1")
                .query_param("pageNumber", "1");

            let json_string = include_str!("fixtures/source_conecpt_parent_page_1.json");
            let body: serde_json::Value = serde_json::from_str(json_string).unwrap();
            then.status(200)
                .header("CONTENT-TYPE", "application/json")
                .json_body(body);
        })
        .await;

    let mock_2 = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/source/SNOMEDCT_US/9468002/parents")
                .query_param_exists("apiKey")
                .query_param("pageSize", "1")
                .query_param("pageNumber", "2");

            let json_string = include_str!("fixtures/source_conecpt_parent_page_2.json");
            let body: serde_json::Value = serde_json::from_str(json_string).unwrap();
            then.status(200)
                .header("CONTENT-TYPE", "application/json")
                .json_body(body);
        })
        .await;

    let base_url = mock_server.base_url().parse()?;

    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .build()?;

    let client = umls_http::reqwest::Client::default();

    let source_concept = SourceConcept::new("SNOMEDCT_US".try_into()?, "9468002".try_into()?);
    let batch = Batch::new(1.try_into()?, 1.try_into()?, source_concept);

    let response = Parent::new(batch)
        .with_client(client)
        .fetch(&umls_config)
        .await?;

    let page = response.into_page();

    assert_eq!(page.size().get(), 1);
    assert_eq!(page.number().get(), 1);
    assert_eq!(page.count().map(|x| x.get()), Some(2));

    let content = page.content();
    assert!(!content.list().is_empty());
    assert_eq!(content.list().len(), 1);

    let object = content
        .list()
        .first()
        .expect("Page 2 content should contain at least one object");

    assert_eq!(object.ui().as_str(), "82065001");
    assert_eq!(object.name().as_str(), "Fracture of carpal bone");
    assert!(!object.suppressible());
    assert!(!object.obsolete());
    assert_eq!(object.root_source().as_str(), "SNOMEDCT_US");
    assert_eq!(object.atom_count(), 7);
    assert_eq!(object.cv_member_count(), 0);

    assert!(page.has_next());
    assert!(!page.has_previous());

    let page = page
        .next()
        .expect("Next page should be available when page 1 of 2 is fetched")
        .fetch(&umls_config)
        .await?
        .into_page();

    assert_eq!(page.size().get(), 1);
    assert_eq!(page.number().get(), 2);
    assert_eq!(page.count().map(|x| x.get()), Some(2));

    let content = page.content();
    assert!(!content.list().is_empty());
    assert_eq!(content.list().len(), 1);

    let object = content
        .list()
        .first()
        .expect("Page 2 content should contain at least one object");

    assert_eq!(object.ui().as_str(), "36991002");
    assert_eq!(object.name().as_str(), "Closed fracture of upper limb");
    assert_eq!(object.root_source().as_str(), "SNOMEDCT_US");
    assert!(!object.suppressible());
    assert!(!object.obsolete());
    assert_eq!(object.atom_count(), 5);
    assert_eq!(object.cv_member_count(), 0);

    assert!(!page.has_next());
    assert!(page.has_previous());

    mock_1.assert_async().await;
    mock_2.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn source_asserted_parents_validation_failed() -> Result<(), Box<dyn std::error::Error>> {
    let client = umls_http::reqwest::Client::default();

    let base_url: Url = "https://uts-ws.nlm.nih.gov/rest/".parse()?;

    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .build()?;

    let urls: Vec<Url> = vec![
        "http://localhost:8080/rest/content/current/source/SNOMEDCT_US/9468002/parents".parse()?,
        "https://localhost:8080/rest/content/current/source/SNOMEDCT_US/9468002/parents".parse()?,
        "https://uts-ws.nlm.nih.gov:8080/rest/content/current/source/SNOMEDCT_US/9468002/parents"
            .parse()?,
    ];

    for url in urls {
        let result = Parent::<parent::types::Atom, _, _>::new(url)
            .with_client(client.clone())
            .fetch(&umls_config)
            .await;

        assert!(matches!(
            result,
            Err(Error::<umls_http::reqwest::Error>::Validation(_))
        ));
    }

    Ok(())
}

#[tokio::test]
async fn fetch_source_asserted_concepts_using_batch_backward_pagination()
-> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    let mock_1 = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/source/SNOMEDCT_US/9468002/parents")
                .query_param_exists("apiKey")
                .query_param("pageSize", "1")
                .query_param("pageNumber", "1");

            let json_string = include_str!("fixtures/source_conecpt_parent_page_1.json");
            let body: serde_json::Value = serde_json::from_str(json_string).unwrap();
            then.status(200)
                .header("CONTENT-TYPE", "application/json")
                .json_body(body);
        })
        .await;

    let mock_2 = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/source/SNOMEDCT_US/9468002/parents")
                .query_param_exists("apiKey")
                .query_param("pageSize", "1")
                .query_param("pageNumber", "2");

            let json_string = include_str!("fixtures/source_conecpt_parent_page_2.json");
            let body: serde_json::Value = serde_json::from_str(json_string).unwrap();
            then.status(200)
                .header("CONTENT-TYPE", "application/json")
                .json_body(body);
        })
        .await;

    let base_url = mock_server.base_url().parse()?;
    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .build()?;

    let client = umls_http::reqwest::Client::default();

    let source_concept = SourceConcept::new("SNOMEDCT_US".try_into()?, "9468002".try_into()?);
    let batch = Batch::new(1.try_into()?, 2.try_into()?, source_concept);

    let response = Parent::new(batch)
        .with_client(client)
        .fetch(&umls_config)
        .await?;

    let page = response.into_page();

    assert_eq!(page.size().get(), 1);
    assert_eq!(page.number().get(), 2);
    assert_eq!(page.count().map(|x| x.get()), Some(2));

    let content = page.content();
    assert!(!content.list().is_empty());
    assert_eq!(content.list().len(), 1);

    let object = content
        .list()
        .first()
        .expect("The contains at-least 1 object");

    assert_eq!(object.ui().as_str(), "36991002");
    assert_eq!(object.name().as_str(), "Closed fracture of upper limb");
    assert_eq!(object.root_source().as_str(), "SNOMEDCT_US");
    assert!(!object.suppressible());
    assert!(!object.obsolete());
    assert_eq!(object.atom_count(), 5);
    assert_eq!(object.cv_member_count(), 0);

    assert!(!page.has_next());
    assert!(page.has_previous());

    let page = page
        .previous()
        .expect("Previous page should be available when page 1 of 2 is fetched")
        .fetch(&umls_config)
        .await?
        .into_page();

    assert_eq!(page.size().get(), 1);
    assert_eq!(page.number().get(), 1);
    assert_eq!(page.count().map(|x| x.get()), Some(2));

    let content = page.content();
    assert!(!content.list().is_empty());
    assert_eq!(content.list().len(), 1);

    let object = content
        .list()
        .first()
        .expect("Should contain at least one object");

    assert_eq!(object.ui().as_str(), "82065001");
    assert_eq!(object.name().as_str(), "Fracture of carpal bone");
    assert!(!object.suppressible());
    assert!(!object.obsolete());
    assert_eq!(object.root_source().as_str(), "SNOMEDCT_US");
    assert_eq!(object.atom_count(), 7);
    assert_eq!(object.cv_member_count(), 0);

    assert!(page.has_next());
    assert!(!page.has_previous());

    mock_1.assert_async().await;
    mock_2.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn fetch_source_asserted_concepts_using_source_atom_cluster()
-> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start_async().await;
    let mock = mock_server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/content/current/source/SNOMEDCT_US/9468002/parents")
                .query_param_exists("apiKey");
            let json_string = include_str!("fixtures/source_conecpt_parent_page_1.json");
            let json: serde_json::Value = serde_json::from_str(json_string).unwrap();
            then.status(200).json_body(json);
        })
        .await;

    let base_url: Url = mock_server.base_url().parse()?;

    let umls_config = config::Config::builder("some_api_key")
        .base_url(base_url)
        .build()?;

    let source_atom_cluster = SourceConcept::new("SNOMEDCT_US".try_into()?, "9468002".try_into()?);

    let page = Parent::new(source_atom_cluster)
        .with_client(Default::default())
        .fetch(&umls_config)
        .await?;

    assert_eq!(page.size().get(), 1);
    assert_eq!(page.number().get(), 1);
    assert_eq!(page.count().get(), 2);
    assert!(!page.content().list().is_empty());
    assert_eq!(page.content().list().len(), 1);

    let object = page.content().list().first().unwrap();
    assert_eq!(object.ui().as_str(), "82065001");
    assert!(!object.suppressible());
    assert!(!object.obsolete());
    assert_eq!(object.root_source().as_str(), "SNOMEDCT_US");
    assert_eq!(object.atom_count(), 7);
    assert_eq!(object.cv_member_count(), 0);

    mock.assert_async().await;
    Ok(())
}
