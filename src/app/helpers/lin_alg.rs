// TODO: Implement 2D matrices and vectors w/ appropriate operations and constructions
//  then refactor algorithm with these structs

use std::ops;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl From<[f64; 2]> for Vec2 {
    fn from(value: [f64; 2]) -> Self {
        Vec2 {
            x: value[0],
            y: value[1],
        }
    }
}

impl Vec2 {
    /// Norm squared
    pub fn normsq(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2)
    }

    /// Norm
    pub fn norm(&self) -> f64 {
        self.normsq().sqrt()
    }

    /// Inner product
    pub fn ip(&self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }
    
    pub fn onenorm(&self) -> f64 {
        self.x.abs()+self.y.abs()
    }
    
    pub fn infnorm(&self) -> f64 {
        f64::max(self.x.abs(), self.y.abs())
    }
}

impl ops::Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::from([self.x + rhs.x, self.y + rhs.y])
    }
}

impl ops::Sub<Vec2> for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::from([self.x - rhs.x, self.y - rhs.y])
    }
}

impl ops::Mul<Vec2> for f64 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2::from([self * rhs.x, self * rhs.y])
    }
}

/// Order: [[a,b], [c,d]] (row column as usual)
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Mat2 {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
}

impl From<[f64; 4]> for Mat2 {
    fn from(value: [f64; 4]) -> Mat2 {
        Mat2 {
            a: value[0],
            b: value[1],
            c: value[2],
            d: value[3],
        }
    }
}

impl Mat2 {
    pub fn from_rows(row1: Vec2, row2: Vec2) -> Mat2 {
        Mat2::from([row1.x, row1.y, row2.x, row2.y])
    }

    pub fn from_columns(col1: Vec2, col2: Vec2) -> Mat2 {
        Mat2::from([col1.x, col2.x, col1.y, col2.y])
    }

    pub fn diagonal(a: f64, d: f64) -> Mat2 {
        Mat2::from([a, 0.0, 0.0, d])
    }
    
    /// Determinant
    pub fn det(&self) -> f64 {
        self.a * self.d - self.b * self.c
    }

    pub fn transpose(&self) -> Mat2 {
        Mat2::from([self.a, self.c, self.b, self.d])
    }

    pub fn inverse(&self) -> Option<Mat2> {
        if self.det() != 0.0 {
            Some(1.0 / self.det() * Mat2::from([self.d, -self.b, -self.c, self.a]))
        } else {
            None
        }
    }
}

impl ops::Add<Mat2> for Mat2 {
    type Output = Mat2;

    fn add(self, rhs: Mat2) -> Self::Output {
        Mat2::from([
            self.a + rhs.a,
            self.b + rhs.b,
            self.c + rhs.c,
            self.d + rhs.d,
        ])
    }
}

impl ops::Mul<Mat2> for f64 {
    type Output = Mat2;

    fn mul(self, rhs: Mat2) -> Self::Output {
        Mat2::from([self * rhs.a, self * rhs.b, self * rhs.c, self * rhs.d])
    }
}

impl ops::Mul<Vec2> for Mat2 {
    type Output = Vec2;

    fn mul(self, vec: Vec2) -> Self::Output {
        Vec2::from([
            self.a * vec.x + self.b * vec.y,
            self.c * vec.x + self.c * vec.y,
        ])
    }
}

impl ops::Mul<Mat2> for Mat2 {
    type Output = Mat2;

    fn mul(self, rhs: Mat2) -> Self::Output {
        Mat2::from([
            self.a * rhs.a + self.b * rhs.c,
            self.a * rhs.b + self.b * rhs.d,
            self.c * rhs.a + self.d * rhs.c,
            self.c * rhs.b + self.d * rhs.d,
        ])
    }
}
