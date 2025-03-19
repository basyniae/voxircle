use std::fmt::{Display, Formatter};

/// Enum for selecting which shape we're interested in in App (provided no logic by itself, all that
///  is done in `app.rs`
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ShapeType {
    Squircle,
    Line,
}

impl Display for ShapeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:}", self.name())
    }
}

impl ShapeType {
    /// For use in combobox
    pub fn list_all_shape_types() -> Vec<ShapeType> {
        vec![Self::Squircle, Self::Line]
    }

    pub fn name(&self) -> String {
        match self {
            ShapeType::Squircle => "Squircle".to_string(),
            ShapeType::Line => "Line".to_string(),
        }
    }
}
