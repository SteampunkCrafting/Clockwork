use scene_utils::mesh_vertex::ColoredVertex;
use vulkano::impl_vertex;

#[derive(Clone, Default, Debug)]
pub(super) struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
}

impl_vertex!(Vertex, position, normal, color);

impl From<ColoredVertex> for Vertex {
    fn from(
        ColoredVertex {
            position,
            normal,
            color,
        }: ColoredVertex,
    ) -> Self {
        Self {
            position,
            normal,
            color,
        }
    }
}
