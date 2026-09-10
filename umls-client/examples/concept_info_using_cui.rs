//! An interactive terminal UI for retrieving a UMLS concept by CUI.

mod common;

use std::{error::Error, time::Duration};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    prelude::Stylize,
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use umls_http::reqwest;
use umls_client::{
    concept::{self, Concept, cui::Cui},
    config::Config,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Search,
    Details,
}

struct App {
    input: String,
    concept: Option<Concept>,
    status: String,
    focus: Focus,
    scroll: u16,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            input: String::new(),
            concept: None,
            status: "Enter a CUI such as C0010200 and press Enter.".into(),
            focus: Focus::Search,
            scroll: 0,
            should_quit: false,
        }
    }

    fn scroll_down(&mut self) {
        let last_line = self
            .concept
            .as_ref()
            .map_or(0, |concept| concept_line_count(concept).saturating_sub(1));
        self.scroll = self.scroll.saturating_add(1).min(last_line);
    }

    fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
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
        terminal.draw(|frame| render(frame, &app))?;

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
                Focus::Search if app.concept.is_some() => Focus::Details,
                Focus::Details => Focus::Search,
                Focus::Search => Focus::Search,
            };
            continue;
        }

        match app.focus {
            Focus::Search => match key.code {
                KeyCode::Enter => fetch_concept(&mut app, config, &client, terminal).await,
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Char(character) => app.input.push(character),
                _ => {}
            },
            Focus::Details => match key.code {
                KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                KeyCode::Home => app.scroll = 0,
                KeyCode::End => {
                    app.scroll = app
                        .concept
                        .as_ref()
                        .map_or(0, |concept| concept_line_count(concept).saturating_sub(1));
                }
                KeyCode::Char('/') => app.focus = Focus::Search,
                KeyCode::Char('q') => app.should_quit = true,
                _ => {}
            },
        }
    }

    Ok(())
}

async fn fetch_concept(
    app: &mut App,
    config: &Config,
    client: &reqwest::Client,
    terminal: &mut DefaultTerminal,
) {
    let raw_cui = app.input.trim().to_owned();
    let cui = match Cui::new(&raw_cui) {
        Ok(cui) => cui,
        Err(error) => {
            app.status = format!("Invalid CUI: {error}");
            return;
        }
    };

    app.status = format!("Loading {cui}…");
    let _ = terminal.draw(|frame| render(frame, app));

    match concept::Info::new(cui)
        .with_client(client.clone())
        .fetch(config)
        .await
    {
        Ok(concept) => {
            app.status = format!("Loaded {}.", concept.ui);
            app.concept = Some(concept);
            app.scroll = 0;
            app.focus = Focus::Details;
        }
        Err(error) => app.status = format!("Could not load concept: {error}"),
    }
}

fn render(frame: &mut Frame<'_>, app: &App) {
    let [search_area, details_area, status_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_search(frame, app, search_area);
    render_details(frame, app, details_area);
    render_status(frame, app, status_area);

    frame.render_widget(
        Paragraph::new(match app.focus {
            Focus::Search => "Type CUI  •  Enter fetch  •  Tab details  •  Esc quit",
            Focus::Details => "↑/↓ or j/k scroll  •  Home/End  •  / or Tab search  •  q quit",
        })
        .style(Style::default().fg(Color::DarkGray)),
        help_area,
    );
}

fn render_search(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().title(" CUI ").borders(Borders::ALL))
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

fn render_details(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let text = app
        .concept
        .as_ref()
        .map_or_else(|| Text::from("No concept loaded."), concept_text);

    let details = Paragraph::new(text)
        .block(Block::default().title(" Concept ").borders(Borders::ALL))
        .scroll((app.scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(details, area);
}

fn render_status(frame: &mut Frame<'_>, app: &App, area: Rect) {
    frame.render_widget(
        Paragraph::new(app.status.as_str())
            .block(Block::default().title(" Status ").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn concept_text(concept: &Concept) -> Text<'static> {
    let mut lines = vec![
        field("CUI", &concept.ui),
        field("Name", &concept.name),
        field("Class", &format!("{:?}", concept.class_type)),
        field("Status", &concept.status),
        field("Suppressible", &concept.suppressible.to_string()),
        field("Date added", &concept.date_added),
        field("Major revision date", &concept.major_revision_date),
        field("Atom count", &concept.atom_count.to_string()),
        field("Attribute count", &concept.attribute_count.to_string()),
        field("CV member count", &concept.cv_member_count.to_string()),
        field("Relation count", &concept.relation_count.to_string()),
        Line::from(""),
        Line::styled(
            "Semantic types",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if concept.semantic_types.is_empty() {
        lines.push(Line::from("  None"));
    } else {
        lines.extend(
            concept
                .semantic_types
                .iter()
                .map(|semantic_type| Line::from(format!("  • {}", semantic_type.name))),
        );
    }

    Text::from(lines)
}

fn field(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        format!("{label}: ").bold().cyan(),
        value.to_owned().into(),
    ])
}

fn concept_line_count(concept: &Concept) -> u16 {
    let fixed_lines = 13_u16;
    let semantic_lines = u16::try_from(concept.semantic_types.len().max(1)).unwrap_or(u16::MAX);
    fixed_lines.saturating_add(semantic_lines)
}
