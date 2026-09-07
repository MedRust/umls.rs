//! Base URL validation and endpoint resolution regression tests.

use umls_client::config::{Config, Error};

#[test]
fn default_base_url_preserves_rest_prefix() {
    let config = Config::builder("test-key").build().unwrap();
    let endpoint = config
        .base_url()
        .join("content/current/CUI/C0010200/atoms")
        .unwrap();
    assert_eq!(endpoint.path(), "/rest/content/current/CUI/C0010200/atoms");
}

#[test]
fn rejects_base_url_paths_without_trailing_slash() {
    for url in [
        "https://example.com/rest",
        "https://example.com/proxy/rest",
        "https://example.com/rest?redirect=/",
        "https://example.com/rest#fragment/",
    ] {
        let result = Config::builder("test-key")
            .base_url(url.parse().unwrap())
            .build();
        assert!(
            matches!(result, Err(Error::MissingBaseUrlTrailingSlash)),
            "{url}"
        );
    }
}

#[test]
fn accepts_directory_paths_and_preserves_custom_prefixes() {
    for url in [
        "https://example.com",
        "https://example.com/",
        "https://example.com/proxy/rest/",
        "https://example.com/proxy/rest/?key=value",
    ] {
        let base_url: url::Url = url.parse().unwrap();
        let config = Config::builder("test-key")
            .base_url(base_url.clone())
            .build()
            .unwrap();
        assert_eq!(config.base_url(), &base_url);
        let endpoint = config.base_url().join("content/current").unwrap();
        assert_eq!(
            endpoint.path(),
            format!("{}content/current", base_url.path())
        );
    }
}
