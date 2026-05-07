use crate::utils::color::Color;

pub struct GridBufferInstance {}

impl GridBufferInstance {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct GridCell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
}

pub struct GridBuffer {}

impl GridBuffer {
    pub fn new() -> Self {
        Self {}
    }
}
