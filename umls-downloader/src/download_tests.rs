use super::*;
use httpmock::prelude::*;
use std::{error::Error as _, time::Duration};

#[test]
fn category_names_select_current_or_all() {
    for current in [false, true] {
        let uri = release_uri("umls-full-release", current).unwrap();
        let url = url::Url::parse(&uri.to_string()).unwrap();
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["releaseType"], "umls-full-release");
        assert_eq!(
            pairs.get("current").map(String::as_str),
            current.then_some("true")
        );
    }
    assert!(release_uri("invalid category", false).is_err());
}

#[test]
fn outer_errors_preserve_original_sources() {
    let uri = "invalid uri".parse::<http::Uri>().unwrap_err();
    let error: Error<reqwest::Error> = Error::Uri(uri);
    assert!(error.source().unwrap().is::<http::uri::InvalidUri>());
    let error: Error<reqwest::Error> = url::Url::parse(":invalid").unwrap_err().into();
    assert!(error.source().unwrap().is::<url::ParseError>());
    let error: Error<reqwest::Error> =
        std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied").into();
    assert_eq!(
        error
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::PermissionDenied
    );
    let original = http::Request::builder()
        .header("bad\nname", "value")
        .body(())
        .unwrap_err();
    let error = Error::Client(reqwest::Error::from(original));
    assert!(error.source().unwrap().is::<reqwest::Error>());
    assert!(
        error
            .source()
            .unwrap()
            .source()
            .unwrap()
            .is::<http::Error>()
    );
    let original = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
    let error = reqwest::Error::from(original);
    assert!(error.source().unwrap().is::<serde_json::Error>());
}

#[tokio::test]
async fn file_output_creates_and_replaces_with_all_chunks() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("archive.zip");
    for existing in [false, true] {
        if existing {
            std::fs::write(&path, b"old content longer than new archive").unwrap();
        }
        let stream = futures_util::stream::iter([
            Ok(Bytes::from_static(b"abc")),
            Ok(Bytes::from_static(b"def")),
        ]);
        write_download(stream, &path).await.unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"abcdef");
    }
}

#[tokio::test]
async fn invalid_destination_returns_io_error() {
    let dir = tempfile::tempdir().unwrap();
    let error = write_download(futures_util::stream::empty(), dir.path())
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Io(_)));
    assert!(error.source().unwrap().is::<std::io::Error>());
}

#[tokio::test]
async fn stream_error_is_preserved_by_file_output() {
    let original = http::Request::builder()
        .header("bad\nname", "value")
        .body(())
        .unwrap_err();
    let dir = tempfile::tempdir().unwrap();
    let error = write_download(
        futures_util::stream::iter([Err(reqwest::Error::Http(original))]),
        dir.path().join("partial.zip"),
    )
    .await
    .unwrap_err();
    assert!(matches!(error, Error::Client(reqwest::Error::Http(_))));
    assert!(
        error
            .source()
            .unwrap()
            .source()
            .unwrap()
            .is::<http::Error>()
    );
}

#[tokio::test]
async fn download_response_stream_writes_local_archive() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/download")
                .query_param("apiKey", "test+key")
                .query_param("url", "https://example.test/archive.zip");
            then.body("archive bytes");
        })
        .await;
    let source = "https://example.test/archive.zip".parse().unwrap();
    let downloader = ReleaseDownloader::new("test+key", source).with_client(Client::default());
    // Keep the production query construction; route only the origin to the local server.
    let uri = download_uri(&downloader.value, "test+key").unwrap();
    let local_uri = format!("{}{}", server.base_url(), uri.path_and_query().unwrap())
        .parse()
        .unwrap();
    let stream = downloader.download_from_uri(local_uri).await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("archive.zip");
    write_download(stream, &path).await.unwrap();
    assert_eq!(std::fs::read(path).unwrap(), b"archive bytes");
    mock.assert_async().await;
}

#[tokio::test]
async fn cancelling_delayed_file_download_drops_stream_and_leaves_incomplete_file() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (started, ready) = tokio::sync::oneshot::channel();
    let (finish, wait) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            request.push(socket.read_u8().await.unwrap());
        }
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100000\r\nConnection: close\r\n\r\nabc")
            .await
            .unwrap();
        wait.await.unwrap();
    });
    let downloader = ReleaseDownloader::new(
        "key",
        "https://example.test/archive".parse::<http::Uri>().unwrap(),
    )
    .with_client(Client::default());
    let stream = downloader
        .download_from_uri(format!("http://{address}/download").parse().unwrap())
        .await
        .unwrap();
    let mut signal = Some(started);
    let stream = stream.inspect(move |_| {
        if let Some(signal) = signal.take() {
            let _ = signal.send(());
        }
    });
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("partial.zip");
    let task = tokio::spawn(write_download(stream, path.clone()));
    tokio::time::timeout(Duration::from_secs(3), ready)
        .await
        .unwrap()
        .unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert!(std::fs::metadata(path).unwrap().len() < 100000);
    finish.send(()).unwrap();
    server.await.unwrap();
}
