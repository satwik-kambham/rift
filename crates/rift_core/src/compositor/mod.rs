use crate::buffer::{grid::GridBufferInstance, rope::RopeBufferInstance};

pub mod grid;

pub struct Compositor {
    pub output_buffer: grid::CompositorGridBuffer,
    root: CompositorNode,
}

impl Compositor {
    pub fn new() -> Self {
        Self {
            output_buffer: grid::CompositorGridBuffer::new(),
            root: CompositorNode::Empty,
        }
    }

    pub fn render(&mut self) {}
}

pub enum CompositorNode {
    Empty,
    RopeBufferInstance(RopeBufferInstance),
    GridBufferInstance(GridBufferInstance),
}
