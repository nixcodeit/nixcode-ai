#[derive(Debug, Clone, Copy)]
pub enum InputMode {
    Normal,
    Insert,
    Command,
}

impl InputMode {
    pub fn to_string(&self) -> &str {
        match self {
            InputMode::Normal => "Normal",
            InputMode::Insert => "Insert",
            InputMode::Command => "Command",
        }
    }
}
