use std::fmt;

#[derive(Debug)]
pub struct AurBuilderError {
    pub message: String,
}

impl AurBuilderError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for AurBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AurBuilderError: {}", self.message)
    }
}
