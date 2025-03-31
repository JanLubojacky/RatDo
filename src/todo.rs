// [src/todo.rs]
use chrono::{DateTime, Local};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

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

pub enum InputMode {
    Normal,
    Editing,
}

// Modify the App struct to track when we're in "pick mode"
pub struct App {
    pub todos: Vec<Todo>,
    pub state: ListState,
    pub input_mode: InputMode,
    pub current_input: String,
    pub edit_mode: bool,
    pub picking_mode: bool, // New field to track when we're in picking mode
}

impl App {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            todos: Vec::new(),
            state,
            input_mode: InputMode::Normal,
            current_input: String::new(),
            edit_mode: false,
            picking_mode: false,
        }
    }

    // Toggle picking mode
    pub fn toggle_picking_mode(&mut self) {
        self.picking_mode = !self.picking_mode;
    }

    // Override next and previous to handle moving todos when in picking mode
    pub fn next(&mut self) {
        if self.todos.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.todos.len() - 1 {
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

            // Don't attempt to move if there's only one item
            if self.todos.len() > 1 {
                // Handle wrap-around case
                if current == self.todos.len() - 1 && i == 0 {
                    // Move from end to beginning
                    let todo = self.todos.remove(current);
                    self.todos.insert(0, todo);
                } else {
                    // Standard case - swap with the next item
                    self.todos.swap(current, i);
                }
            }
        }

        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.todos.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.todos.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        // Move the todo if we're in picking mode
        if self.picking_mode && i != self.state.selected().unwrap_or(0) {
            let current = self.state.selected().unwrap_or(0);

            // Don't attempt to move if there's only one item
            if self.todos.len() > 1 {
                // Handle wrap-around case
                if current == 0 && i == self.todos.len() - 1 {
                    // Move from beginning to end
                    let todo = self.todos.remove(0);
                    self.todos.push(todo);
                } else {
                    // Standard case - swap with the previous item
                    self.todos.swap(current, i);
                }
            }
        }

        self.state.select(Some(i));
    }

    pub fn add_todo(&mut self) {
        let todo = Todo::new(self.current_input.clone());
        self.todos.push(todo);
        self.current_input.clear();
    }

    pub fn delete_todo(&mut self) {
        if let Some(selected) = self.state.selected() {
            if !self.todos.is_empty() && selected < self.todos.len() {
                self.todos.remove(selected);
                if selected > 0 && selected == self.todos.len() {
                    self.state.select(Some(selected - 1));
                }
            }
        }
    }

    pub fn toggle_todo(&mut self) {
        if let Some(selected) = self.state.selected() {
            if !self.todos.is_empty() && selected < self.todos.len() {
                // Store the previous completion state
                let was_completed = self.todos[selected].completed;

                // Toggle the completion status
                self.todos[selected].completed = !self.todos[selected].completed;

                // If todo was just marked as completed, move to the next item
                if !was_completed && self.todos[selected].completed {
                    self.next();
                }
            }
        }
    }

    pub fn start_editing(&mut self) {
        if let Some(selected) = self.state.selected() {
            if !self.todos.is_empty() && selected < self.todos.len() {
                self.current_input = self.todos[selected].description.clone();
                self.input_mode = InputMode::Editing;
                self.edit_mode = true;
            }
        }
    }

    pub fn update_todo(&mut self) {
        if let Some(selected) = self.state.selected() {
            if !self.todos.is_empty() && selected < self.todos.len() {
                self.todos[selected].description = self.current_input.clone();
                self.current_input.clear();
            }
        }
    }

    pub fn load_todos(&mut self) -> io::Result<()> {
        let path = Path::new("todos.json");
        if path.exists() {
            let content = fs::read_to_string(path)?;
            self.todos = serde_json::from_str(&content).unwrap_or_default();
        }
        Ok(())
    }

    pub fn save_todos(&self) -> io::Result<()> {
        let json = serde_json::to_string(&self.todos)?;
        fs::write("todos.json", json)?;
        Ok(())
    }
}
