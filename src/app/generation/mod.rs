// For outputting the bitmatrices + size. Always solid, we do interior removal in preprocessing. Bunch of algorithms

use std::fmt::Display;

mod centerpoint;
mod conservative;
mod contained;
mod empty;
pub mod percentage; // want it public because we use the circle intersection area as a widget
pub mod shape;
mod square;
pub mod squircle;
