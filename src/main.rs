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
use std::env;
use std::error::Error;
use std::io;

// Import our own modules
mod todo;
use todo::{App, InputMode};

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Create app instance
    let mut app = App::new();
    app.load_todos()?;

    // Handle CLI commands
    if args.len() > 1 {
        match args[1].as_str() {
            "show" => {
                // Print available todo pages and exit
                println!("Available todo pages:");
                for (idx, name) in app.page_names().iter().enumerate() {
                    println!("  {}: {}", idx + 1, name);
                }
                return Ok(());
            }
            page_name => {
                // Command is a page name - create or select that page
                app.create_or_select_page(page_name);
                // Continue to the TUI
            }
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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
                            if !app.todos().is_empty() {
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
                            if !app.todos().is_empty() {
                                app.toggle_picking_mode();
                            }
                        }
                        KeyCode::Char('P') => {
                            // Toggle page selector
                            app.toggle_page_selector();
                        }
                        KeyCode::Tab => {
                            // Switch to next page
                            app.next_page();
                        }
                        KeyCode::BackTab => {
                            // Switch to previous page
                            app.previous_page();
                        }
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Char('j') => app.next(),
                        KeyCode::Char('k') => app.previous(),
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            if app.show_page_selector && !app.current_input.is_empty() {
                                // Add a new page
                                app.add_page(app.current_input.clone());
                                app.current_input.clear();
                                app.show_page_selector = false;
                                app.input_mode = InputMode::Normal;
                            } else if app.edit_mode && !app.current_input.is_empty() {
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
                            app.show_page_selector = false;
                        }
                        _ => {}
                    },
                    InputMode::PageSelect => match key.code {
                        KeyCode::Enter => {
                            // Select the highlighted page
                            if let Some(selected) = app.page_select_state.selected() {
                                app.current_page_index = selected;
                                app.show_page_selector = false;
                                app.input_mode = InputMode::Normal;
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Char('a') => {
                            // Create a new page from the page selector
                            app.input_mode = InputMode::Editing;
                            app.edit_mode = false;
                            app.current_input = String::new();
                            // Keep page selector flag true
                        }
                        KeyCode::Char('d') => {
                            // Delete the selected page (if there's more than one)
                            if app.pages.len() > 1 {
                                if let Some(selected) = app.page_select_state.selected() {
                                    app.pages.remove(selected);

                                    // Adjust current page index if needed
                                    if selected >= app.pages.len() {
                                        app.page_select_state.select(Some(app.pages.len() - 1));
                                    } else {
                                        app.page_select_state.select(Some(selected));
                                    }

                                    // Update current_page_index to match the new selection
                                    app.current_page_index =
                                        app.page_select_state.selected().unwrap_or(0);

                                    // Reset todo selection for the new page
                                    let todo_count = app.todos().len();
                                    if todo_count > 0 {
                                        app.state.select(Some(0));
                                    } else {
                                        app.state.select(None);
                                    }
                                }
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            // Navigate down in page list
                            if !app.pages.is_empty() {
                                let i = match app.page_select_state.selected() {
                                    Some(i) => {
                                        if i >= app.pages.len() - 1 {
                                            0
                                        } else {
                                            i + 1
                                        }
                                    }
                                    None => 0,
                                };
                                app.page_select_state.select(Some(i));
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            // Navigate up in page list
                            if !app.pages.is_empty() {
                                let i = match app.page_select_state.selected() {
                                    Some(i) => {
                                        if i == 0 {
                                            app.pages.len() - 1
                                        } else {
                                            i - 1
                                        }
                                    }
                                    None => 0,
                                };
                                app.page_select_state.select(Some(i));
                            }
                        }
                        KeyCode::Esc | KeyCode::Char('P') => {
                            // Exit page select mode
                            app.show_page_selector = false;
                            app.input_mode = InputMode::Normal;
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
                Constraint::Length(1), // Title
                Constraint::Min(1),    // Todos list
                Constraint::Length(3), // Help
            ]
            .as_ref(),
        )
        .split(f.area());

    // Title with page name
    let page_name = &app.current_page().name;
    let title = Paragraph::new(format!("[ To Do 🐀: {} ]", page_name))
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default());
    f.render_widget(title, chunks[0]);

    // Todos
    let todos: Vec<ListItem> = app
        .todos()
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
            Style::default().fg(Color::LightYellow)
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
                "q: Quit | e: Edit | a: Add | d: Delete | P: Page List | Tab/Shift+Tab: Switch Page | p: Move | Space: Toggle | j/k: Navigate"
            }
        }
        InputMode::Editing => {
            if app.show_page_selector {
                "Esc: Cancel | Enter: Create Page"
            } else {
                "Esc: Cancel | Enter: Save"
            }
        }
        InputMode::PageSelect => {
            "Esc: Cancel | Enter: Select Page | n/a: New Page | d: Delete Page | j/k: Navigate"
        }
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help, chunks[2]);

    // Render the page selector if active
    if app.show_page_selector {
        // Create a centered popup for the page selector
        let area = f.area();
        let popup_width = area.width.min(50).max(30);
        let popup_height = app.pages.len() as u16 + 2;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = ratatui::layout::Rect::new(popup_x, popup_y, popup_width, popup_height);

        // Create a clear background for the popup
        let clear = ratatui::widgets::Clear;
        f.render_widget(clear, popup_area);

        // Create page items
        let page_items: Vec<ListItem> = app
            .pages
            .iter()
            .map(|page| {
                ListItem::new(Span::styled(
                    &page.name,
                    if page.name == app.current_page().name {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    },
                ))
            })
            .collect();

        // Page list widget
        let pages_list = List::new(page_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Select Page (n/a: New, d: Delete)"),
            )
            .highlight_style(Style::default().fg(Color::LightYellow))
            .highlight_symbol(" > ");

        f.render_stateful_widget(pages_list, popup_area, &mut app.page_select_state);
    }

    // Render the input popup when in editing mode
    if let InputMode::Editing = app.input_mode {
        if !app.show_page_selector {
            // Create a centered popup for the input
            let area = f.area();
            let popup_width = area.width.saturating_sub(40);
            let popup_height = 3;
            let popup_x = (area.width.saturating_sub(popup_width)) / 2;
            let popup_y = (area.height.saturating_sub(popup_height)) / 2;

            let popup_area =
                ratatui::layout::Rect::new(popup_x, popup_y, popup_width, popup_height);

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
        } else {
            // Show the page creation popup
            let area = f.area();
            let popup_width = 40;
            let popup_height = 3;
            let popup_x = (area.width.saturating_sub(popup_width)) / 2;
            let popup_y = (area.height.saturating_sub(popup_height)) / 2 - 5;

            let popup_area =
                ratatui::layout::Rect::new(popup_x, popup_y, popup_width, popup_height);

            // Create a clear background for the popup
            let clear = ratatui::widgets::Clear;
            f.render_widget(clear, popup_area);

            // New page popup
            let input = Paragraph::new(app.current_input.as_str())
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("New Page Name"),
                );
            f.render_widget(input, popup_area);

            // Set cursor position within the popup
            f.set_cursor_position((
                popup_area.x + app.current_input.len() as u16 + 1,
                popup_area.y + 1,
            ));
        }
    }
}
