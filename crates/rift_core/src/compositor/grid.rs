use bitflags::bitflags;

use crate::utils::color::Color;

bitflags! {
    #[derive(Default, Clone)]
    pub struct CellAttributes: u32 {
        const NONE = 0;
        const BOLD = 1 << 0;
        const ITALICS = 1 << 1;
        const UNDERLINE = 1 << 2;
    }
}

#[derive(Clone)]
pub struct CompositorGridCell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
    pub attributes: CellAttributes,
}

impl Default for CompositorGridCell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: Color::WHITE,
            bg: Color::BLACK,
            attributes: CellAttributes::default(),
        }
    }
}

pub struct CompositorGridBuffer {
    pub rows: usize,
    pub columns: usize,
    pub grid: Vec<CompositorGridCell>,
}

impl CompositorGridBuffer {
    pub fn new() -> Self {
        Self {
            rows: 16,
            columns: 16,
            grid: vec![Default::default(); 16 * 16],
        }
    }

    pub fn slice(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> CompositorGridSlice<'_> {
        assert!(x + width <= self.columns);
        assert!(y + height <= self.rows);

        CompositorGridSlice {
            buffer: self,
            top_left_x: x,
            top_left_y: y,
            width,
            height,
        }
    }
}

pub struct CompositorGridSlice<'a> {
    buffer: &'a mut CompositorGridBuffer,

    top_left_x: usize,
    top_left_y: usize,
    width: usize,
    height: usize,
}

impl<'a> CompositorGridSlice<'a> {}
