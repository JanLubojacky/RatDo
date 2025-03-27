use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::error::Error;
use std::io;

// Import our own modules
mod todo;
use todo::{App, InputMode};

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    app.load_todos()?;

    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            app.save_todos()?;
                            return Ok(());
                        }
                        KeyCode::Char('e') => {
                            if !app.todos.is_empty() {
                                app.start_editing();
                            }
                        }
                        KeyCode::Char('a') => {
                            app.input_mode = InputMode::Editing;
                            app.edit_mode = false; // Changed to false for adding new todos
                            app.current_input = String::new();
                        }
                        KeyCode::Char('d') => app.delete_todo(),
                        KeyCode::Char(' ') => app.toggle_todo(),
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Char('j') => app.next(),
                        KeyCode::Char('k') => app.previous(),
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            if app.edit_mode && !app.current_input.is_empty() {
                                app.update_todo();
                            } else if !app.current_input.is_empty() {
                                app.add_todo();
                            }
                            app.input_mode = InputMode::Normal;
                            app.edit_mode = false;
                        }
                        KeyCode::Char(c) => {
                            app.current_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.current_input.pop();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.edit_mode = false;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    // Create a layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.area()); // Changed from size() to area()

    // Title
    let title = Paragraph::new("Todo App")
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default());
    f.render_widget(title, chunks[0]);

    // Input
    let input = Paragraph::new(app.current_input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);

    // Set cursor position
    if let InputMode::Editing = app.input_mode {
        // Changed from set_cursor to set_cursor_position with a tuple
        f.set_cursor_position((
            chunks[1].x + app.current_input.len() as u16 + 1,
            chunks[1].y + 1,
        ));
    }

    // Todos
    let todos: Vec<ListItem> = app
        .todos
        .iter()
        .map(|todo| {
            let status = if todo.completed { "[x]" } else { "[ ]" };
            let content = format!("{} {}", status, todo.description);
            let style = if todo.completed {
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(content, style))
        })
        .collect();

    let todos = List::new(todos)
        .block(Block::default().borders(Borders::ALL).title("Todos"))
        .highlight_style(Style::default().fg(Color::LightCyan))
        .highlight_symbol("> ");

    f.render_stateful_widget(todos, chunks[2], &mut app.state);

    // Help
    let help_text = match app.input_mode {
        InputMode::Normal => {
            "q: Quit | e: Edit | a: Add | d: Delete | Space: Toggle | ↑↓: Navigate"
        }
        InputMode::Editing => "Esc: Cancel | Enter: Save",
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, chunks[3]);
}
