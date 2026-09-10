//! An interactive terminal UI for searching and paging through UMLS concepts.

mod common;

use std::{error::Error, num::NonZeroU64, time::Duration};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
};
use umls_client::{
    config::Config,
    search::{
        self, ConceptSnapshot, Content, Page, Response, Search, core::Term, return_type::Concept,
    },
};
use umls_http::reqwest;

type ConceptPage<'a> =
    Page<Content<ConceptSnapshot>, Search<'a, umls_http::reqwest::Client, Concept>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Search,
    Results,
}

struct App<'a> {
    input: String,
    active_term: Option<String>,
    page: Option<ConceptPage<'a>>,
    status: String,
    focus: Focus,
    table_state: TableState,
    should_quit: bool,
}

impl App<'_> {
    fn new() -> Self {
        Self {
            input: String::new(),
            active_term: None,
            page: None,
            status: "Type a search term and press Enter.".into(),
            focus: Focus::Search,
            table_state: TableState::default(),
            should_quit: false,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = common::Args::parse();
    let config = common::config_from_args(args)?;

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &config).await;
    ratatui::restore();
    result
}

async fn run(terminal: &mut DefaultTerminal, config: &Config) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    let client = reqwest::Client::default();

    while !app.should_quit {
        terminal.draw(|frame| render(frame, &mut app))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        if key.code == KeyCode::Esc {
            app.should_quit = true;
            continue;
        }

        if key.code == KeyCode::Tab {
            app.focus = match app.focus {
                Focus::Search if app.page.is_some() => Focus::Results,
                Focus::Results => Focus::Search,
                Focus::Search => Focus::Search,
            };
            continue;
        }

        match app.focus {
            Focus::Search => match key.code {
                KeyCode::Enter => search(&mut app, config, &client, terminal).await,
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Char(character) => app.input.push(character),
                _ => {}
            },
            Focus::Results => match key.code {
                KeyCode::Down | KeyCode::Char('j') => select_next_result(&mut app),
                KeyCode::Up | KeyCode::Char('k') => select_previous_result(&mut app),
                KeyCode::Home => select_first_result(&mut app),
                KeyCode::End => select_last_result(&mut app),
                KeyCode::Char('n') => {
                    next_page(&mut app, terminal).await;
                }
                KeyCode::Char('p') => {
                    previous_page(&mut app, terminal).await;
                }
                KeyCode::Char('/') => app.focus = Focus::Search,
                KeyCode::Char('q') => app.should_quit = true,
                _ => {}
            },
        }
    }

    Ok(())
}

async fn search<'a>(
    app: &mut App<'a>,
    config: &'a Config,
    client: &reqwest::Client,
    terminal: &mut DefaultTerminal,
) {
    let raw_term = app.input.trim().to_owned();
    if raw_term.is_empty() {
        app.status = "Enter a non-empty search term.".into();
        return;
    }

    let term = match Term::try_new(&raw_term) {
        Ok(term) => term,
        Err(error) => {
            app.status = format!("Invalid search term: {error}");
            return;
        }
    };

    app.status = format!("Searching for {raw_term:?}…");
    let _ = terminal.draw(|frame| render(frame, app));

    match search::Builder::new(term)
        .concept(config)
        .with_client(client.clone())
        .get()
        .await
    {
        Ok(response) => match response.into_page() {
            Ok(page) => {
                let result_count = page.count().map_or(0, NonZeroU64::get);
                app.active_term = Some(raw_term.to_owned());
                app.status = if result_count == 0 {
                    format!("No concepts found for {raw_term:?}.")
                } else {
                    format!("Found {result_count} concepts for {raw_term:?}.")
                };
                app.focus = Focus::Results;
                app.page = Some(page);
                select_first_result(app);
            }
            Err(error) => app.status = format!("Could not read search results: {error}"),
        },
        Err(error) => app.status = format!("Search failed: {error}"),
    }
}

async fn next_page(app: &mut App<'_>, terminal: &mut DefaultTerminal) {
    let Some(page) = app.page.as_ref() else {
        app.status = "Search before requesting another page.".into();
        return;
    };

    if !page.has_next() {
        app.status = "Already at the last page; there is no next page.".into();
        return;
    }

    app.status = "Loading the next page…".into();
    let _ = terminal.draw(|frame| render(frame, app));

    let page = app.page.take().expect("page was checked above");
    match page.next() {
        Ok(search) => match search.get().await {
            Ok(response) => replace_page(app, response, "Loaded the next page."),
            Err(error) => app.status = format!("Could not load the page: {error}"),
        },
        Err(error) => {
            app.status = format!("Could not load the next page: {error}");
        }
    }
}

async fn previous_page(app: &mut App<'_>, terminal: &mut DefaultTerminal) {
    let Some(page) = app.page.as_ref() else {
        app.status = "Search before requesting another page.".into();
        return;
    };

    if !page.has_previous() {
        app.status = "Already at the first page; there is no previous page.".into();
        return;
    }

    app.status = "Loading the previous page…".into();
    let _ = terminal.draw(|frame| render(frame, app));

    let page = app.page.take().expect("page was checked above");
    match page.previous() {
        Ok(search) => match search.get().await {
            Ok(response) => replace_page(app, response, "Loaded the previous page."),
            Err(error) => {
                app.status = format!("Could not load the page: {error}");
            }
        },
        Err(error) => {
            app.status = format!("Could not load the previous page: {error}");
        }
    }
}

fn replace_page<'a>(
    app: &mut App<'a>,
    response: Response<
        search::raw::SearchResponse,
        Search<'a, umls_http::reqwest::Client, Concept>,
    >,
    success_message: &str,
) {
    match response.into_page() {
        Ok(page) => {
            app.page = Some(page);
            select_first_result(app);
            app.status = success_message.into();
        }
        Err(error) => app.status = format!("Could not read search results: {error}"),
    }
}

fn select_next_result(app: &mut App<'_>) {
    let result_count = app
        .page
        .as_ref()
        .map_or(0, |page| page.content().list().len());
    if result_count == 0 {
        app.table_state.select(None);
        return;
    }

    let next = app
        .table_state
        .selected()
        .map_or(0, |selected| (selected + 1).min(result_count - 1));
    app.table_state.select(Some(next));
}

fn select_previous_result(app: &mut App<'_>) {
    let result_count = app
        .page
        .as_ref()
        .map_or(0, |page| page.content().list().len());
    if result_count == 0 {
        app.table_state.select(None);
        return;
    }

    let previous = app
        .table_state
        .selected()
        .map_or(0, |selected| selected.saturating_sub(1));
    app.table_state.select(Some(previous));
}

fn select_first_result(app: &mut App<'_>) {
    let has_results = app
        .page
        .as_ref()
        .is_some_and(|page| !page.content().list().is_empty());
    app.table_state.select(has_results.then_some(0));
}

fn select_last_result(app: &mut App<'_>) {
    let last = app
        .page
        .as_ref()
        .and_then(|page| page.content().list().len().checked_sub(1));
    app.table_state.select(last);
}

fn render(frame: &mut Frame<'_>, app: &mut App<'_>) {
    let [search_area, table_area, status_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_search(frame, app, search_area);
    render_results(frame, app, table_area);
    render_status(frame, app, status_area);

    frame.render_widget(
        Paragraph::new(match app.focus {
            Focus::Search => "Type query  •  Enter search  •  Tab results  •  Esc quit",
            Focus::Results => {
                "↑/↓ or j/k scroll  •  n next page  •  p previous page  •  / search  •  q quit"
            }
        })
        .style(Style::default().fg(Color::DarkGray)),
        help_area,
    );
}

fn render_search(frame: &mut Frame<'_>, app: &App<'_>, area: Rect) {
    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().title(" Search ").borders(Borders::ALL))
        .style(if app.focus == Focus::Search {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        });
    frame.render_widget(input, area);

    if app.focus == Focus::Search {
        let cursor_offset = u16::try_from(app.input.chars().count()).unwrap_or(u16::MAX);
        frame.set_cursor_position((area.x.saturating_add(1 + cursor_offset), area.y + 1));
    }
}

fn render_results(frame: &mut Frame<'_>, app: &mut App<'_>, area: Rect) {
    let header = Row::new(["#", "CUI", "Name", "Root source", "Semantic types"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows = app
        .page
        .as_ref()
        .into_iter()
        .flat_map(|page| page.content().list().iter().enumerate())
        .map(|(index, concept)| {
            let semantic_types = concept
                .semantic_types()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            Row::new([
                Cell::from((index + 1).to_string()),
                Cell::from(concept.cui().to_owned()),
                Cell::from(concept.name().to_owned()),
                Cell::from(concept.root_source().to_owned()),
                Cell::from(semantic_types),
            ])
        });

    let page_title = app.page.as_ref().map_or_else(
        || " Results ".to_owned(),
        |page| {
            let total = page
                .total_pages()
                .map_or_else(|| "0".to_owned(), |value| value.to_string());
            format!(" Results — page {} of {total} ", page.number())
        },
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(10),
            Constraint::Percentage(35),
            Constraint::Length(16),
            Constraint::Percentage(35),
        ],
    )
    .header(header)
    .column_spacing(1)
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ")
    .block(Block::default().title(page_title).borders(Borders::ALL));

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_status(frame: &mut Frame<'_>, app: &App<'_>, area: Rect) {
    let active_term = app
        .active_term
        .as_deref()
        .map_or_else(String::new, |term| format!(" [{term}]"));

    frame.render_widget(
        Paragraph::new(Line::from(app.status.as_str()))
            .block(
                Block::default()
                    .title(format!(" Status{active_term} "))
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}
