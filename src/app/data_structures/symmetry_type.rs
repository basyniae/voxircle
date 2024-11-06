use std::fmt::{Display, Formatter};

//
#[derive(Debug)]
pub enum SymmetryType {
    ReflectionHorizontal,
    ReflectionVertical,
    ReflectionDiagonalUp,
    ReflectionDiagonalDown,
    ReflectionsCardinals,
    ReflectionsDiagonals,
    ReflectionsAll,
    RotationHalf,
    RotationQuarter,
    NoSymmetry,
}

impl Display for SymmetryType {
    // Used as "Symmetry type: format("{}", symmetry_type)
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SymmetryType::ReflectionHorizontal => write!(f, "Reflection along horizontal line"),
            SymmetryType::ReflectionVertical => write!(f, "Reflection along vertical line"),
            SymmetryType::ReflectionDiagonalUp => write!(f, "Reflection along up 45° diagonal"),
            SymmetryType::ReflectionDiagonalDown => write!(f, "Reflection along down 45° diagonal"),
            SymmetryType::ReflectionsCardinals => {
                write!(f, "Reflection along horizontal and vertical lines")
            }
            SymmetryType::ReflectionsDiagonals => write!(f, "Reflection along both 45° diagonals"),
            SymmetryType::ReflectionsAll => write!(
                f,
                "Reflections along horizontal, vertical, and 45° diagonal lines"
            ),
            SymmetryType::RotationHalf => write!(f, "Rotation by 180°, or mirroring in a point"),
            SymmetryType::RotationQuarter => write!(f, "Rotation by 90°"),
            SymmetryType::NoSymmetry => write!(f, "No symmetry"),
        }
    }
}
