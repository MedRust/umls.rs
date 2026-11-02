//! Base URL validation and endpoint resolution regression tests.

use umls_client::config::Config;

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
