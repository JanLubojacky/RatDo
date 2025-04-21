use chrono::{DateTime, Local};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Todo {
    pub description: String,
    pub completed: bool,
    pub created_at: DateTime<Local>,
}

impl Todo {
    pub fn new(description: String) -> Self {
        Self {
            description,
            completed: false,
            created_at: Local::now(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TodoPage {
    pub name: String,
    pub todos: Vec<Todo>,
}

impl TodoPage {
    pub fn new(name: String) -> Self {
        Self {
            name,
            todos: Vec::new(),
        }
    }
}

pub enum InputMode {
    Normal,
    Editing,
    PageSelect,
}

// Modify the App struct to track when we're in "pick mode"
pub struct App {
    pub pages: Vec<TodoPage>,
    pub current_page_index: usize,
    pub state: ListState,
    pub page_select_state: ListState,
    pub input_mode: InputMode,
    pub current_input: String,
    pub edit_mode: bool,
    pub picking_mode: bool,
    pub show_page_selector: bool,
}

impl App {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        let mut page_select_state = ListState::default();
        page_select_state.select(Some(0));

        // Create a default page
        let default_page = TodoPage::new("Default".to_string());
        let pages = vec![default_page];

        Self {
            pages,
            current_page_index: 0,
            state,
            page_select_state,
            input_mode: InputMode::Normal,
            current_input: String::new(),
            edit_mode: false,
            picking_mode: false,
            show_page_selector: false,
        }
    }

    // Current page accessor
    pub fn current_page(&self) -> &TodoPage {
        &self.pages[self.current_page_index]
    }

    // Current todos accessor
    pub fn todos(&self) -> &Vec<Todo> {
        &self.current_page().todos
    }

    // Current todos mutable accessor
    pub fn todos_mut(&mut self) -> &mut Vec<Todo> {
        &mut self.pages[self.current_page_index].todos
    }

    // Add a new page
    pub fn add_page(&mut self, name: String) {
        if !name.is_empty() && !self.pages.iter().any(|p| p.name == name) {
            let new_page = TodoPage::new(name);
            self.pages.push(new_page);
            self.current_page_index = self.pages.len() - 1;

            // Update page select state
            self.page_select_state.select(Some(self.current_page_index));
        }
    }

    // Select a page by name
    pub fn select_page_by_name(&mut self, name: &str) -> bool {
        if let Some(index) = self.pages.iter().position(|p| p.name == name) {
            self.current_page_index = index;
            self.page_select_state.select(Some(index));

            // Reset todo selection for the new page
            let todo_count = self.todos().len();
            if todo_count > 0 {
                self.state.select(Some(0));
            } else {
                self.state.select(None);
            }

            true
        } else {
            false
        }
    }

    // Navigate to next page
    pub fn next_page(&mut self) {
        if !self.pages.is_empty() {
            let i = match self.page_select_state.selected() {
                Some(i) => {
                    if i >= self.pages.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };

            self.current_page_index = i;
            self.page_select_state.select(Some(i));

            // Reset todo selection for the new page
            let todo_count = self.todos().len();
            if todo_count > 0 {
                self.state.select(Some(0));
            } else {
                self.state.select(None);
            }
        }
    }

    // Navigate to previous page
    pub fn previous_page(&mut self) {
        if !self.pages.is_empty() {
            let i = match self.page_select_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.pages.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };

            self.current_page_index = i;
            self.page_select_state.select(Some(i));

            // Reset todo selection for the new page
            let todo_count = self.todos().len();
            if todo_count > 0 {
                self.state.select(Some(0));
            } else {
                self.state.select(None);
            }
        }
    }

    // Toggle page selector visibility
    pub fn toggle_page_selector(&mut self) {
        self.show_page_selector = !self.show_page_selector;

        if self.show_page_selector {
            self.input_mode = InputMode::PageSelect;
            self.page_select_state.select(Some(self.current_page_index));
        } else {
            self.input_mode = InputMode::Normal;
        }
    }

    // Toggle picking mode
    pub fn toggle_picking_mode(&mut self) {
        self.picking_mode = !self.picking_mode;
    }

    // Override next and previous to handle moving todos when in picking mode
    pub fn next(&mut self) {
        let todos = self.todos();
        if todos.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= todos.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        // Move the todo if we're in picking mode
        if self.picking_mode && i != self.state.selected().unwrap_or(0) {
            let current = self.state.selected().unwrap_or(0);
            let todos = self.todos_mut();

            // Don't attempt to move if there's only one item
            if todos.len() > 1 {
                // Handle wrap-around case
                if current == todos.len() - 1 && i == 0 {
                    // Move from end to beginning
                    let todo = todos.remove(current);
                    todos.insert(0, todo);
                } else {
                    // Standard case - swap with the next item
                    todos.swap(current, i);
                }
            }
        }

        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let todos = self.todos();
        if todos.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    todos.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        // Move the todo if we're in picking mode
        if self.picking_mode && i != self.state.selected().unwrap_or(0) {
            let current = self.state.selected().unwrap_or(0);
            let todos = self.todos_mut();

            // Don't attempt to move if there's only one item
            if todos.len() > 1 {
                // Handle wrap-around case
                if current == 0 && i == todos.len() - 1 {
                    // Move from beginning to end
                    let todo = todos.remove(0);
                    todos.push(todo);
                } else {
                    // Standard case - swap with the previous item
                    todos.swap(current, i);
                }
            }
        }

        self.state.select(Some(i));
    }

    pub fn add_todo(&mut self) {
        let todo = Todo::new(self.current_input.clone());
        self.todos_mut().push(todo);
        self.current_input.clear();
    }

    pub fn delete_todo(&mut self) {
        if let Some(selected) = self.state.selected() {
            let todos = self.todos_mut();
            if !todos.is_empty() && selected < todos.len() {
                todos.remove(selected);
                if selected > 0 && selected == todos.len() {
                    self.state.select(Some(selected - 1));
                }
            }
        }
    }

    pub fn toggle_todo(&mut self) {
        if let Some(selected) = self.state.selected() {
            let todos = self.todos_mut();
            if !todos.is_empty() && selected < todos.len() {
                // Store the previous completion state
                let was_completed = todos[selected].completed;

                // Toggle the completion status
                todos[selected].completed = !todos[selected].completed;

                // If todo was just marked as completed, move to the next item
                if !was_completed && todos[selected].completed {
                    self.next();
                }
            }
        }
    }

    pub fn start_editing(&mut self) {
        if let Some(selected) = self.state.selected() {
            let todos = self.todos();
            if !todos.is_empty() && selected < todos.len() {
                self.current_input = todos[selected].description.clone();
                self.input_mode = InputMode::Editing;
                self.edit_mode = true;
            }
        }
    }

    pub fn update_todo(&mut self) {
        if let Some(selected) = self.state.selected() {
            // Clone first to avoid borrowing issues
            let current_input_clone = self.current_input.clone();
            self.current_input.clear();

            let todos = self.todos_mut();
            if !todos.is_empty() && selected < todos.len() {
                todos[selected].description = current_input_clone;
            }
        }
    }

    fn get_config_path() -> io::Result<PathBuf> {
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;

        Ok(PathBuf::from(home)
            .join(".config")
            .join("ratdo")
            .join("todos.json"))
    }

    pub fn load_todos(&mut self) -> io::Result<()> {
        let path = Self::get_config_path()?;

        if path.exists() {
            let content = fs::read_to_string(path)?;
            self.pages = serde_json::from_str(&content).unwrap_or_else(|_| {
                // Handle backward compatibility with old format
                let old_todos: Vec<Todo> = serde_json::from_str(&content).unwrap_or_default();
                let default_page = TodoPage {
                    name: "Default".to_string(),
                    todos: old_todos,
                };
                vec![default_page]
            });

            // Ensure we have at least one page
            if self.pages.is_empty() {
                self.pages.push(TodoPage::new("Default".to_string()));
            }

            // Set initial selection
            if !self.todos().is_empty() {
                self.state.select(Some(0));
            }
            self.page_select_state.select(Some(0));

            // Reset current page index in case it's invalid
            self.current_page_index = 0;
        }
        Ok(())
    }

    pub fn save_todos(&self) -> io::Result<()> {
        let path = Self::get_config_path()?;

        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string(&self.pages)?;
        fs::write(path, json)?;
        Ok(())
    }

    // Get a list of page names - helpful for CLI "show" command
    pub fn page_names(&self) -> Vec<String> {
        self.pages.iter().map(|p| p.name.clone()).collect()
    }

    // Create a new page if it doesn't exist and select it
    pub fn create_or_select_page(&mut self, name: &str) {
        if !self.select_page_by_name(name) {
            self.add_page(name.to_string());
        }
    }
}
