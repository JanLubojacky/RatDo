use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
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
                        KeyCode::Char('p') => {
                            if !app.todos.is_empty() {
                                app.toggle_picking_mode();
                            }
                        }
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
    // Create a layout without the input area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1), // Title
                Constraint::Min(1),    // Todos list
                Constraint::Length(3), // Help
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title
    let title = Paragraph::new("[ To Do 🐀]")
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default());
    f.render_widget(title, chunks[0]);

    // Todos
    let todos: Vec<ListItem> = app
        .todos
        .iter()
        .enumerate() // Get index with item
        .map(|(i, todo)| {
            let status = if todo.completed { "[x]" } else { "[ ]" };

            let content = if app.picking_mode && Some(i) == app.state.selected() {
                // Show a moving indicator when in picking mode and this is the selected todo
                format!(" {} {}", status, todo.description)
            } else {
                format!(" {} {}", status, todo.description)
            };

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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if app.picking_mode {
                    "Moving Todo (Navigate with j/k)"
                } else {
                    "Todos"
                }),
        )
        .highlight_style(if app.picking_mode {
            // Use a different highlight style when picking
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::LightCyan)
        })
        .highlight_symbol(if app.picking_mode {
            " >>" // Different symbol when picking
        } else {
            " > "
        });

    f.render_stateful_widget(todos, chunks[1], &mut app.state);

    // Help
    let help_text = match app.input_mode {
        InputMode::Normal => {
            if app.picking_mode {
                "p: Exit Move Mode | j/k: Move Item Up/Down"
            } else {
                "q: Quit | e: Edit | a: Add | d: Delete | p: Move | Space: Toggle | j/k: Navigate"
            }
        }
        InputMode::Editing => "Esc: Cancel | Enter: Save",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, chunks[2]);

    // Render the input popup only when in editing mode
    if let InputMode::Editing = app.input_mode {
        // Create a centered popup for the input
        let area = f.area();
        let popup_width = 60.min(area.width.saturating_sub(4));
        let popup_height = 3;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = ratatui::layout::Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Create a clear background for the popup
        let clear = ratatui::widgets::Clear;
        f.render_widget(clear, popup_area);

        // Input popup
        let input_title = if app.edit_mode {
            "Edit Todo"
        } else {
            "Add Todo"
        };
        let input = Paragraph::new(app.current_input.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(input_title));
        f.render_widget(input, popup_area);

        // Set cursor position within the popup
        f.set_cursor_position((
            popup_area.x + app.current_input.len() as u16 + 1,
            popup_area.y + 1,
        ));
    }
}
