use std::{path::PathBuf, time::Duration};

use clap::Parser;
use crossterm::event::{self, Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures_util::StreamExt;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use umls_downloader::{
    Release, ReleaseFetcher,
    client::ReqwestClient,
    download_release_to_path,
    kind::All,
    release_type::{self, ReleaseType},
};

#[derive(Debug, Parser)]
#[command(
    name = "umls-download",
    about = "Browse and download UMLS, RxNorm, and SNOMED CT releases"
)]
struct Args {
    /// UTS API key. UMLS_API_KEY is used when this option is omitted.
    #[arg(long, env = "UMLS_API_KEY", hide_env_values = true)]
    api_key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    ReleaseTypes,
    Versions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modal {
    None,
    Destination,
    ApiKey,
}

struct App {
    release_types: Vec<ReleaseType>,
    releases: Vec<Release>,
    release_type_state: ListState,
    release_state: ListState,
    focus: Focus,
    modal: Modal,
    input: String,
    api_key: Option<String>,
    pending_release: Option<Release>,
    pending_destination: Option<PathBuf>,
    status: String,
    quit: bool,
}

impl App {
    fn new(api_key: Option<String>) -> Self {
        Self {
            release_types: Vec::new(),
            releases: Vec::new(),
            release_type_state: ListState::default(),
            release_state: ListState::default(),
            focus: Focus::ReleaseTypes,
            modal: Modal::None,
            input: String::new(),
            api_key: api_key.filter(|key| !key.trim().is_empty()),
            pending_release: None,
            pending_destination: None,
            status: "Loading release types…".into(),
            quit: false,
        }
    }

    fn set_release_types(&mut self, release_types: Vec<ReleaseType>) {
        self.release_types = release_types;
        self.release_type_state
            .select((!self.release_types.is_empty()).then_some(0));
    }

    fn set_releases(&mut self, releases: Vec<Release>) {
        self.releases = releases;
        self.release_state
            .select((!self.releases.is_empty()).then_some(0));
    }

    fn move_selection(&mut self, delta: isize) -> bool {
        let (state, len) = match self.focus {
            Focus::ReleaseTypes => (&mut self.release_type_state, self.release_types.len()),
            Focus::Versions => (&mut self.release_state, self.releases.len()),
        };
        if len == 0 {
            return false;
        }
        let current = state.selected().unwrap_or(0);
        let next = (current as isize + delta).clamp(0, len.saturating_sub(1) as isize) as usize;
        state.select(Some(next));
        next != current
    }

    fn selected_release_type(&self) -> Option<&ReleaseType> {
        self.release_type_state
            .selected()
            .and_then(|index| self.release_types.get(index))
    }

    fn begin_download(&mut self) {
        let Some(release) = self
            .release_state
            .selected()
            .and_then(|index| self.releases.get(index))
            .cloned()
        else {
            self.status = "Select a release version first".into();
            return;
        };
        self.pending_release = Some(release);
        self.modal = Modal::Destination;
        self.input.clear();
    }

    fn accept_destination(&mut self) -> Option<Download> {
        let release = self.pending_release.as_ref()?;
        let destination = destination_path(&self.input, &release.file_name);
        self.pending_destination = Some(destination);
        if self.api_key.is_none() {
            self.modal = Modal::ApiKey;
            self.input.clear();
            None
        } else {
            self.finish_download_request()
        }
    }

    fn accept_api_key(&mut self) -> Option<Download> {
        let key = self.input.trim();
        if key.is_empty() {
            self.status = "API key cannot be empty".into();
            return None;
        }
        self.api_key = Some(key.to_owned());
        self.input.clear();
        self.finish_download_request()
    }

    fn finish_download_request(&mut self) -> Option<Download> {
        self.modal = Modal::None;
        Some(Download {
            release: self.pending_release.take()?,
            destination: self.pending_destination.take()?,
            api_key: self.api_key.clone()?,
        })
    }
}

struct Download {
    release: Release,
    destination: PathBuf,
    api_key: String,
}

enum DownloadResult {
    Completed(Result<(), umls_downloader::Error>),
    Cancelled { quit: bool },
}

fn destination_path(input: &str, file_name: &str) -> PathBuf {
    let input = input.trim();
    if input.is_empty() {
        return PathBuf::from(file_name);
    }
    let path = PathBuf::from(input);
    if path.is_dir() || input.ends_with(std::path::MAIN_SEPARATOR) {
        path.join(file_name)
    } else {
        path
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = ReqwestClient::new();
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, client.clone(), args.api_key).await;
    ratatui::restore();
    result
}

async fn run(
    terminal: &mut DefaultTerminal,
    client: ReqwestClient,
    api_key: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(api_key);
    terminal.draw(|frame| draw(frame, &mut app))?;

    let release_types = release_type::Client::default().with_client(client.clone());

    match release_types.fetch().await {
        Ok(types) => {
            app.set_release_types(types);
            app.status = "↑/↓ select · Tab switch table · Enter download · q quit".into();
            load_selected_releases(&mut app, client.clone()).await;
        }
        Err(error) => app.status = format!("Could not load release types: {error}"),
    }

    while !app.quit {
        terminal.draw(|frame| draw(frame, &mut app))?;
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if let Some(download) = handle_key(&mut app, key) {
            app.status = format!(
                "Downloading {}… · Esc cancel · q cancel and quit",
                download.release.file_name
            );
            terminal.draw(|frame| draw(frame, &mut app))?;
            match run_download(client.clone(), &download).await? {
                DownloadResult::Completed(Ok(())) => {
                    app.status = format!("Downloaded to {}", download.destination.display());
                }
                DownloadResult::Completed(Err(error)) => {
                    app.status = format!("Download failed: {error}");
                }
                DownloadResult::Cancelled { quit } => {
                    app.status = format!(
                        "Download cancelled; a partial file may remain at {}",
                        download.destination.display()
                    );
                    app.quit = quit;
                }
            }
        } else if app.modal == Modal::None
            && app.focus == Focus::ReleaseTypes
            && matches!(key.code, KeyCode::Up | KeyCode::Down)
        {
            load_selected_releases(&mut app, client.clone()).await;
        }
    }
    Ok(())
}

async fn run_download(
    client: ReqwestClient,
    download: &Download,
) -> Result<DownloadResult, http::uri::InvalidUri> {
    let source = download.release.download_url.parse()?;
    let operation =
        download_release_to_path(client, source, &download.api_key, &download.destination);
    tokio::pin!(operation);
    let mut events = EventStream::new();

    loop {
        tokio::select! {
            result = &mut operation => return Ok(DownloadResult::Completed(result)),
            event = events.next() => {
                let Some(Ok(Event::Key(key))) = event else {
                    continue;
                };
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Esc => return Ok(DownloadResult::Cancelled { quit: false }),
                    KeyCode::Char('q') => return Ok(DownloadResult::Cancelled { quit: true }),
                    _ => {}
                }
            }
        }
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Option<Download> {
    if app.modal != Modal::None {
        return match key.code {
            KeyCode::Esc if app.modal == Modal::Destination => {
                app.modal = Modal::None;
                app.pending_release = None;
                app.pending_destination = None;
                app.input.clear();
                None
            }
            KeyCode::Backspace => {
                app.input.pop();
                None
            }
            KeyCode::Char(character) => {
                app.input.push(character);
                None
            }
            KeyCode::Enter if app.modal == Modal::Destination => app.accept_destination(),
            KeyCode::Enter => app.accept_api_key(),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Left | KeyCode::Right => {
            app.focus = match app.focus {
                Focus::ReleaseTypes => Focus::Versions,
                Focus::Versions => Focus::ReleaseTypes,
            };
        }
        KeyCode::Up => {
            app.move_selection(-1);
        }
        KeyCode::Down => {
            app.move_selection(1);
        }
        KeyCode::Enter if app.focus == Focus::Versions => app.begin_download(),
        _ => {}
    }
    None
}

async fn load_selected_releases(app: &mut App, client: ReqwestClient) {
    let Some(release_type) = app.selected_release_type().cloned() else {
        app.set_releases(Vec::new());
        return;
    };
    app.status = format!("Loading {} releases…", release_type);
    match ReleaseFetcher::new(client, release_type, All).send().await {
        Ok(releases) => {
            let count = releases.len();
            app.set_releases(releases);
            app.status = format!("Loaded {count} versions");
        }
        Err(error) => {
            app.set_releases(Vec::new());
            app.status = format!("Could not load versions: {error}");
        }
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(frame.area());
    let tables = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(areas[0]);

    let release_types = app
        .release_types
        .iter()
        .map(|item| ListItem::new(format!("{} · {}", item.product, item.name)));
    let type_style = focus_style(app.focus == Focus::ReleaseTypes);
    frame.render_stateful_widget(
        List::new(release_types)
            .block(
                Block::default()
                    .title(" Release types ")
                    .borders(Borders::ALL)
                    .border_style(type_style),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("› "),
        tables[0],
        &mut app.release_type_state,
    );

    let releases = app.releases.iter().map(|release| {
        ListItem::new(vec![
            Line::from(format!(
                "{}  {}",
                release.release_version, release.release_date
            )),
            Line::styled(
                format!("  {}", release.file_name),
                Style::default().fg(Color::DarkGray),
            ),
        ])
    });
    let version_style = focus_style(app.focus == Focus::Versions);
    frame.render_stateful_widget(
        List::new(releases)
            .block(
                Block::default()
                    .title(" Release versions ")
                    .borders(Borders::ALL)
                    .border_style(version_style),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("› "),
        tables[1],
        &mut app.release_state,
    );
    frame.render_widget(
        Paragraph::new(app.status.as_str())
            .block(Block::default().title(" Status ").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        areas[1],
    );

    match app.modal {
        Modal::Destination => draw_input_popup(
            frame,
            " Download destination ",
            "Path or filename (empty uses the release filename):",
            &app.input,
            false,
        ),
        Modal::ApiKey => draw_input_popup(
            frame,
            " UMLS API key required ",
            "Enter an API key to continue:",
            &app.input,
            true,
        ),
        Modal::None => {}
    }
}

fn focus_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    }
}

fn draw_input_popup(frame: &mut Frame, title: &str, prompt: &str, input: &str, secret: bool) {
    let area = centered_rect(70, 7, frame.area());
    let visible = if secret {
        "•".repeat(input.chars().count())
    } else {
        input.to_owned()
    };
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(vec![Line::from(prompt), Line::from(visible)]).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        ),
        area,
    );
    let cursor_x = area.x
        + 1
        + input
            .chars()
            .count()
            .min(area.width.saturating_sub(3) as usize) as u16;
    frame.set_cursor_position((cursor_x, area.y + 2));
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height.min(area.height)),
        Constraint::Fill(1),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn release() -> Release {
        Release {
            file_name: "rxnorm.zip".into(),
            release_version: "2026AA".into(),
            release_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            download_url: "https://example.test/rxnorm.zip".into(),
            release_type: "rxnorm".into(),
            product: "RxNorm".into(),
            current: true,
        }
    }

    #[test]
    fn empty_destination_uses_release_filename() {
        assert_eq!(destination_path("", "rxnorm.zip"), Path::new("rxnorm.zip"));
    }

    #[test]
    fn missing_key_blocks_download_until_nonempty_key_is_entered() {
        let mut app = App::new(None);
        app.releases.push(release());
        app.release_state.select(Some(0));
        app.begin_download();
        assert!(app.accept_destination().is_none());
        assert_eq!(app.modal, Modal::ApiKey);
        assert!(app.accept_api_key().is_none());
        app.input = "secret".into();
        let download = app.accept_api_key().unwrap();
        assert_eq!(download.api_key, "secret");
        assert_eq!(download.destination, Path::new("rxnorm.zip"));
    }
}
