//! Browse UMLS concept definitions in a terminal UI.
//!
//! Configure with UMLS_BASE_URL, UMLS_API_KEY, and optionally UMLS_VERSION,
//! or use the corresponding command-line flags (see --help).

mod common;

use std::{error::Error, time::Duration};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Cell, Paragraph, Row, Table, TableState, Wrap},
};
use umls_client::{
    atom::Batch,
    concept::cui::Cui,
    config::Config,
    definition::{self, Definition, DefinitionDto},
    search::{Content, Page},
};
use umls_http::reqwest;

type DefinitionPage =
    Page<Content<DefinitionDto>, Definition<reqwest::Client, Batch<definition::Config>>>;

#[derive(PartialEq, Eq)]
enum Focus {
    Input,
    Results,
}

struct App {
    input: String,
    active_cui: Option<Cui>,
    page: Option<DefinitionPage>,
    focus: Focus,
    selection: TableState,
    detail_scroll: u16,
    status: String,
}

impl App {
    fn new() -> Self {
        Self {
            input: String::new(),
            active_cui: None,
            page: None,
            focus: Focus::Input,
            selection: TableState::default(),
            detail_scroll: 0,
            status: "Enter a CUI (for example C0010200), then press Enter.".into(),
        }
    }

    fn select(&mut self, next: bool) {
        let count = self
            .page
            .as_ref()
            .map_or(0, |page| page.content().list().len());
        self.selection.select(if count == 0 {
            None
        } else {
            Some(self.selection.selected().map_or(0, |index| {
                if next {
                    (index + 1).min(count - 1)
                } else {
                    index.saturating_sub(1)
                }
            }))
        });
        self.detail_scroll = 0;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = common::config_from_args(common::Args::parse())?;
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &config).await;
    ratatui::restore();
    result
}

async fn run(terminal: &mut DefaultTerminal, config: &Config) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    let client = reqwest::Client::default();
    loop {
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
        if key.code == KeyCode::Esc
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            return Ok(());
        }
        if key.code == KeyCode::Tab {
            app.focus = if app.focus == Focus::Input {
                Focus::Results
            } else {
                Focus::Input
            };
            continue;
        }
        match app.focus {
            Focus::Input => match key.code {
                KeyCode::Enter => match app.input.trim().parse::<Cui>() {
                    Ok(cui) => load_page(&mut app, cui, 1, config, &client, terminal).await?,
                    Err(error) => app.status = format!("Invalid CUI: {error}"),
                },
                KeyCode::Backspace => {
                    app.input.pop();
                }
                // CUIs contain ASCII letters and digits only.
                KeyCode::Char(c) if c.is_ascii_alphanumeric() => app.input.push(c),
                _ => {}
            },
            Focus::Results => match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('/') => app.focus = Focus::Input,
                KeyCode::Down | KeyCode::Char('j') => app.select(true),
                KeyCode::Up | KeyCode::Char('k') => app.select(false),
                KeyCode::PageDown => app.detail_scroll = app.detail_scroll.saturating_add(5),
                KeyCode::PageUp => app.detail_scroll = app.detail_scroll.saturating_sub(5),
                KeyCode::Char('n') | KeyCode::Right => {
                    change_page(&mut app, true, config, &client, terminal).await?;
                }
                KeyCode::Char('p') | KeyCode::Left => {
                    change_page(&mut app, false, config, &client, terminal).await?;
                }
                _ => {}
            },
        }
    }
}

async fn change_page(
    app: &mut App,
    next: bool,
    config: &Config,
    client: &reqwest::Client,
    terminal: &mut DefaultTerminal,
) -> Result<(), Box<dyn Error>> {
    let (Some(page), Some(cui)) = (&app.page, &app.active_cui) else {
        app.status = "Enter a CUI first.".into();
        return Ok(());
    };
    if (next && !page.has_next()) || (!next && !page.has_previous()) {
        app.status = if next {
            "Already at the last page."
        } else {
            "Already at the first page."
        }
        .into();
        return Ok(());
    }
    let number = if next {
        page.number().get() + 1
    } else {
        page.number().get() - 1
    };
    load_page(app, cui.clone(), number, config, client, terminal).await
}

async fn load_page(
    app: &mut App,
    cui: Cui,
    number: u64,
    config: &Config,
    client: &reqwest::Client,
    terminal: &mut DefaultTerminal,
) -> Result<(), Box<dyn Error>> {
    app.status = format!("Loading definitions for {cui}, page {number}…");
    terminal.draw(|frame| render(frame, app))?;
    let batch = Batch::new(
        25.try_into()?,
        number.try_into()?,
        definition::Config::builder(cui.clone()).build(),
    );
    match Definition::new(batch).with_client(client.clone()).fetch(config).await {
        Ok(response) => {
            let page = response.into_page();
            let count = page.content().list().len();
            app.status = if count == 0 {
                format!("No definitions found for {cui}.")
            } else {
                format!("Loaded {count} definitions for {cui}.")
            };
            app.page = Some(page);
            app.active_cui = Some(cui);
            app.selection = TableState::default().with_selected((count > 0).then_some(0));
            app.detail_scroll = 0;
            app.focus = Focus::Results;
        }
        // Retain the previous page and its CUI so a failed request can be retried.
        Err(_) => app.status = "Request failed. Check your connection, CUI, and API configuration; retry with Enter or n/p.".into(),
    }
    Ok(())
}

fn render(frame: &mut Frame<'_>, app: &mut App) {
    let [input_area, table_area, detail_area, status_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    let input_width = input_area.width.saturating_sub(2) as usize;
    let visible_input = &app.input[app
        .input
        .len()
        .saturating_sub(input_width.saturating_sub(1))..];
    frame.render_widget(
        Paragraph::new(visible_input)
            .block(Block::bordered().title(" CUI — Enter to fetch "))
            .style(Style::default().fg(if app.focus == Focus::Input {
                Color::Yellow
            } else {
                Color::Gray
            })),
        input_area,
    );
    if app.focus == Focus::Input && input_area.width > 2 && input_area.height > 2 {
        frame.set_cursor_position((
            input_area.x + 1 + visible_input.len() as u16,
            input_area.y + 1,
        ));
    }
    let title = app.page.as_ref().map_or_else(
        || " Definitions ".into(),
        |page| {
            format!(
                " {} — page {} of {} — previous: {} / next: {} ",
                app.active_cui.as_ref().expect("loaded page has a CUI"),
                page.number(),
                page.total_pages().map_or(0, |n| n.get()),
                if page.has_previous() { "p" } else { "—" },
                if page.has_next() { "n" } else { "—" }
            )
        },
    );
    let rows = app
        .page
        .as_ref()
        .into_iter()
        .flat_map(|page| page.content().list())
        .map(|definition| {
            Row::new([
                Cell::from(definition.root_source().to_string()),
                Cell::from(
                    definition
                        .value()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" "),
                ),
            ])
        });
    frame.render_stateful_widget(
        Table::new(
            rows,
            [Constraint::Percentage(20), Constraint::Percentage(80)],
        )
        .header(Row::new(["Root source", "Definition"]).style(Style::default().fg(Color::Cyan)))
        .block(Block::bordered().title(title))
        .row_highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("▶ "),
        table_area,
        &mut app.selection,
    );
    let selected = app.page.as_ref().and_then(|page| {
        app.selection
            .selected()
            .and_then(|index| page.content().list().get(index))
    });
    let detail = Paragraph::new(selected.map_or(
        "Select a result to read its full definition.",
        DefinitionDto::value,
    ))
    .block(Block::bordered().title(" Full definition — PgUp/PgDn scroll "))
    .wrap(Wrap { trim: false });
    frame.render_widget(detail.scroll((app.detail_scroll, 0)), detail_area);
    frame.render_widget(
        Paragraph::new(app.status.as_str())
            .wrap(Wrap { trim: true })
            .block(Block::bordered().title(" Status ")),
        status_area,
    );
    frame.render_widget(Paragraph::new(match app.focus {
        Focus::Input => "Enter fetch • Backspace edit • Tab results • Esc/Ctrl-C quit",
        Focus::Results => "↑/↓ select • n/→ next page • p/← previous page • PgUp/PgDn read\nTab or / edit CUI • q/Esc quit",
    }), help_area);
}
