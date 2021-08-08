use vulkano::impl_vertex;

#[derive(Clone, Default, Debug)]
pub(super) struct InstanceData {
    world: [[f32; 4]; 4],
}

impl_vertex!(InstanceData, world);

impl From<[[f32; 4]; 4]> for InstanceData {
    fn from(world: [[f32; 4]; 4]) -> Self {
        Self { world }
    }
}
