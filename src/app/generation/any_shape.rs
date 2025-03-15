use std::fmt::{Display, Formatter};

// No logic here. All the switching happens in app.rs
#[derive(PartialEq, Clone, Copy)]
pub enum AnyShape {
    Squircle,
    Line,
}

impl Display for AnyShape {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:}", self.name())
    }
}

impl AnyShape {
    pub fn list_all_shapes() -> Vec<AnyShape> {
        vec![Self::Squircle, Self::Line]
    }

    pub fn name(&self) -> String {
        match self {
            AnyShape::Squircle => "Squircle".to_string(),
            AnyShape::Line => "Line".to_string(),
        }
    }
}
