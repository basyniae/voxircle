use std::collections::VecDeque;
use std::fmt::Debug;

/// Vector whose index is a continuous interval of the integers (Z)
#[derive(Debug, Clone)]
pub struct ZVec<T> {
    pub data: VecDeque<T>,
    minimum: isize,
    maximum: isize,
}
#[allow(dead_code)]
impl<T: Clone + Debug> ZVec<T> {
    pub fn new(data: VecDeque<T>, minimum: isize) -> Self {
        let data_length = data.len();

        Self {
            data,
            minimum,
            maximum: minimum + data_length as isize - 1, // we have data.len() = max - min + 1
        }
    }

    pub fn get(&self, index: isize) -> Option<T> {
        assert_eq!(self.maximum - self.minimum + 1, self.data.len() as isize);

        if index > self.maximum || index < self.minimum {
            None
        } else {
            Some(self.data[(index - self.minimum) as usize].clone())
        }
    }

    pub fn get_mut(&mut self, index: isize) -> Option<&mut T> {
        if index < self.minimum || index > self.maximum {
            None
        } else {
            self.data.get_mut((index - self.minimum) as usize)
        }
    }

    pub fn set(&mut self, index: isize, value: T) {
        self.data[(index - self.minimum) as usize] = value;
    }

    /// Resize the ZVec so that new_min is the new minimum, removing data or filling with the default as necessary
    pub fn resize_with_min(&mut self, new_min: isize, value: T) {
        if new_min < self.minimum {
            for _ in new_min..self.minimum {
                self.data.push_front(value.clone());
            }
        } else if new_min > self.minimum {
            for _ in self.minimum..new_min {
                self.data.pop_front();
            }
        }
        self.minimum = new_min;
        // else do nothing
    }

    /// Resize the ZVec so that new_max is the new maximum, removing data or filling with the default as necessary
    pub fn resize_with_max(&mut self, new_max: isize, value: T) {
        if new_max > self.maximum {
            for _ in self.maximum..new_max {
                self.data.push_back(value.clone());
            }
        } else if new_max < self.maximum {
            for _ in new_max..self.maximum {
                self.data.pop_back();
            }
        }
        self.maximum = new_max;
        // else do nothing
    }

    /// Resize the ZVec to the new range, removing data or filling with the default as necessary
    pub fn resize(&mut self, new_min: isize, new_max: isize, default: T) {
        self.resize_with_min(new_min, default.clone());
        self.resize_with_max(new_max, default);
    }

    pub fn get_minimum(&self) -> isize {
        self.minimum
    }

    pub fn get_maximum(&self) -> isize {
        self.maximum
    }
}
