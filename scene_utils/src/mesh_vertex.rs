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

impl From<obj::TexturedVertex> for TexturedVertex {
    fn from(
        obj::TexturedVertex {
            position,
            normal,
            texture: [tx, ty, ..],
        }: obj::TexturedVertex,
    ) -> Self {
        Self {
            position,
            normal,
            texture_coord: [tx, ty],
        }
    }
}

impl From<obj::Vertex> for TexturedVertex {
    fn from(obj::Vertex { position, normal }: obj::Vertex) -> Self {
        Self {
            position,
            normal,
            texture_coord: Default::default(),
        }
    }
}
