# UMLS Downloader CLI

Interactive terminal browser and downloader for UMLS, RxNorm, and SNOMED CT
releases.

## Run

Supply the UTS API key as an option:

```sh
cargo run -p umls-downloader-cli -- --api-key YOUR_KEY
```

or through the environment:

```sh
UMLS_API_KEY=YOUR_KEY cargo run -p umls-downloader-cli
```

The API key is only needed when a download starts. If neither source supplies
one, the application displays a masked, non-empty API-key prompt.

## Controls

- `Up` / `Down`: change the selected row
- `Tab`, `Shift+Tab`, `Left`, or `Right`: switch tables
- `Enter`: download the selected release version
- `Esc`: close the destination prompt, or quit from the main screen
- `q`: quit from the main screen

While a download is active, press `Esc` to cancel it and return to the release
browser. Press `q` to cancel the download and exit the application. A cancelled
download may leave a partial destination file behind.

At the destination prompt, leave the value empty to save the release's original
filename in the current directory. Enter an existing directory to save under
that directory with the original filename, or enter a full destination
filename.
