#[derive(Debug, Clone, Copy)]
pub struct ColoredVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
pub struct TexturedVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture_coord: [f32; 2],
}
