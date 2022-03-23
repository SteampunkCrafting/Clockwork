use self::inner_state::{generate_pipeline, InnerState};
use asset_storage::asset_storage::AssetStorageKey;
use graphics::{state::GraphicsState, vulkano_layer::VulkanoLayer};
use kernel::{
    abstract_runtime::{CallbackSubstate, EngineState},
    graphics::{
        scene::{Lights, PrimaryCamera, SceneObjects},
        scene_object::{Camera, Material, Mesh},
        AmbientLight, DirectionalLight, PointLight, RenderingLayerKey, Scene, SceneObject,
        SpotLight,
    },
    util::init_state::InitState,
};
use scene_utils::prelude::{PhongMaterialStorage, TexturedMeshStorage};
use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    descriptor_set::PersistentDescriptorSet,
    image::view::ImageView,
};

pub struct StaticMeshDrawer<
    StateT,
    SceneT,
    AssetT,
    LayerT,
    InstanceT,
    CameraT,
    ALightT,
    DLightT,
    PLightT,
    SLightT,
> where
    StateT: CallbackSubstate<SceneT>
        + CallbackSubstate<TexturedMeshStorage<AssetT>>
        + CallbackSubstate<PhongMaterialStorage<AssetT>>,
    SceneT: Scene<LayerKey = LayerT>
        + SceneObjects<InstanceT>
        + PrimaryCamera<CameraT>
        + Lights<ALightT, DLightT, PLightT, SLightT>,
    InstanceT: SceneObject + Mesh<AssetT> + Material<AssetT>,
    CameraT: Camera,
    AssetT: AssetStorageKey,
    LayerT: RenderingLayerKey,
    ALightT: AmbientLight,
    DLightT: DirectionalLight,
    PLightT: PointLight,
    SLightT: SpotLight,
{
    layer_id: LayerT,
    inner: InitState<(), InnerState<AssetT>>,
    _phantom: PhantomData<(
        StateT,
        SceneT,
        AssetT,
        LayerT,
        InstanceT,
        CameraT,
        ALightT,
        DLightT,
        PLightT,
        SLightT,
    )>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct DrawMarker;

impl<StateT, LayerT, AssetT, SceneT, InstanceT, CameraT, ALightT, DLightT, PLightT, SLightT>
    StaticMeshDrawer<
        StateT,
        SceneT,
        AssetT,
        LayerT,
        InstanceT,
        CameraT,
        ALightT,
        DLightT,
        PLightT,
        SLightT,
    >
where
    StateT: CallbackSubstate<SceneT>
        + CallbackSubstate<TexturedMeshStorage<AssetT>>
        + CallbackSubstate<PhongMaterialStorage<AssetT>>,
    SceneT: Scene<LayerKey = LayerT>
        + SceneObjects<InstanceT>
        + PrimaryCamera<CameraT>
        + Lights<ALightT, DLightT, PLightT, SLightT>,
    InstanceT: SceneObject + Mesh<AssetT> + Material<AssetT>,
    CameraT: Camera,
    AssetT: AssetStorageKey,
    LayerT: RenderingLayerKey,
    ALightT: AmbientLight,
    DLightT: DirectionalLight,
    PLightT: PointLight,
    SLightT: SpotLight,
{
    pub fn new(layer_id: LayerT) -> Self {
        Self {
            layer_id,
            inner: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<
        StateT,
        AssetT,
        LayerT,
        SceneT,
        InstanceType,
        CameraType,
        AmbientLightType,
        DirectionalLightType,
        PointLightType,
        SpotLightType,
    > VulkanoLayer<StateT>
    for StaticMeshDrawer<
        StateT,
        SceneT,
        AssetT,
        LayerT,
        InstanceType,
        CameraType,
        AmbientLightType,
        DirectionalLightType,
        PointLightType,
        SpotLightType,
    >
where
    StateT: CallbackSubstate<SceneT>
        + CallbackSubstate<TexturedMeshStorage<AssetT>>
        + CallbackSubstate<PhongMaterialStorage<AssetT>>,
    SceneT: Scene<LayerKey = LayerT>
        + SceneObjects<InstanceType>
        + PrimaryCamera<CameraType>
        + Lights<AmbientLightType, DirectionalLightType, PointLightType, SpotLightType>,
    InstanceType: SceneObject + Mesh<AssetT> + Material<AssetT>,
    CameraType: Camera,
    AssetT: AssetStorageKey,
    LayerT: RenderingLayerKey,
    AmbientLightType: AmbientLight,
    DirectionalLightType: DirectionalLight,
    PointLightType: PointLight,
    SpotLightType: SpotLight,
{
    fn draw(
        &mut self,
        engine_state: &EngineState<StateT>,
        graphics_state: &GraphicsState,
    ) -> vulkano::command_buffer::SecondaryAutoCommandBuffer {
        let Self {
            layer_id, inner, ..
        } = self;
        let InnerState {
            buffered_meshes,
            pipeline,
            vertex_uniform_pool,
            fragment_uniform_mesh_pool,
            fragment_uniform_world_pool,
            texture_sampler,
            default_texture,
        } = inner.get_init_mut();
        let GraphicsState {
            subpass,
            device,
            queue,
            ..
        } = graphics_state;

        engine_state
            .start_access()
            .get(|scene: &SceneT| {
                /* ---- GETTING SCENE INFO ---- */
                let camera = scene.primary_camera(layer_id.clone());
                let entities = scene.scene_objects(layer_id.clone());
                let ambient_light = scene.ambient_light(layer_id.clone());
                let dir_lights = scene.directional_lights(layer_id.clone());
                let point_lights = scene.point_lights(layer_id.clone());
                let spot_lights = scene.spot_lights(layer_id.clone());

                /* ---- CREATING INSTANCE HASH MAP ---- */
                let instances_map = {
                    let mut instances = HashMap::new();
                    entities.for_each(|entity| {
                        instances
                            .entry(entity.mesh_id())
                            .or_insert_with(|| Vec::new())
                            .push(entity);
                    });
                    instances
                }
                .into_iter()
                /* ---- BUFFERING NEW MESHES AND MATERIALS ---- */
                .map(|(mesh_id, instances)| {
                    (
                        mesh_id.clone(),
                        instances,
                        buffered_meshes
                            .entry(mesh_id.clone())
                            .or_insert_with(|| {
                                engine_state
                                    .start_access()
                                    .get(|materials: &PhongMaterialStorage<_>| {
                                        materials.get(mesh_id.clone()).clone()
                                    })
                                    .then_get_zip(|meshes: &TexturedMeshStorage<_>| {
                                        meshes.get(mesh_id.clone()).clone()
                                    })
                                    .map(|(material, mesh)| {
                                        (graphics_state, &*mesh.lock(), &*material.lock()).into()
                                    })
                                    .finish()
                            })
                            .clone(),
                    )
                });

                /* ---- SETTING UP WORLD UNIFORMS ---- */
                let vertex_uniform_set = {
                    let mut set = PersistentDescriptorSet::start(
                        pipeline
                            .layout()
                            .descriptor_set_layouts()
                            .get(0)
                            .unwrap()
                            .clone(),
                    );
                    set.add_buffer(Arc::new(
                        vertex_uniform_pool
                            .next(inner_state::make_vertex_uniforms(
                                camera.projection_matrix(),
                                camera.view_matrix(),
                            ))
                            .unwrap(),
                    ))
                    .unwrap();
                    Arc::new(set.build().unwrap())
                };
                let fragment_world_uniform_set = {
                    let mut set = PersistentDescriptorSet::start(
                        pipeline
                            .layout()
                            .descriptor_set_layouts()
                            .get(1)
                            .unwrap()
                            .clone(),
                    );
                    set.add_buffer(Arc::new(
                        fragment_uniform_world_pool
                            .next(inner_state::make_world_fragment_uniforms(
                                camera,
                                ambient_light,
                                dir_lights,
                                point_lights,
                                spot_lights,
                            ))
                            .unwrap(),
                    ))
                    .unwrap();
                    Arc::new(set.build().unwrap())
                };

                /* ---- SETTING UP PER-INSTANCE UNIFORMS AND WRITING DRAW CALLS ---- */
                let mut cmd_builder = AutoCommandBufferBuilder::secondary_graphics(
                    device.clone(),
                    queue.family(),
                    CommandBufferUsage::OneTimeSubmit,
                    subpass.clone(),
                )
                .unwrap();
                instances_map.into_iter().fold(
                    cmd_builder
                        .bind_pipeline_graphics(pipeline.clone())
                        .bind_descriptor_sets(
                            vulkano::pipeline::PipelineBindPoint::Graphics,
                            pipeline.layout().clone(),
                            0,
                            (
                                vertex_uniform_set.clone(),
                                fragment_world_uniform_set.clone(),
                            ),
                        ),
                    |cmd, (mesh_id, instances, buffered_mesh)| {
                        let instance_count = instances.len() as u32;
                        let fragment_mesh_uniform_set = {
                            let mut set = PersistentDescriptorSet::start(
                                pipeline
                                    .layout()
                                    .descriptor_set_layouts()
                                    .get(2)
                                    .unwrap()
                                    .clone(),
                            );
                            set.add_buffer(Arc::new(
                                fragment_uniform_mesh_pool
                                    .next(inner_state::make_mesh_fragment_uniforms(
                                        engine_state
                                            .start_access()
                                            .get(|materials: &PhongMaterialStorage<_>| {
                                                materials.get(mesh_id).lock().clone()
                                            })
                                            .finish(),
                                    ))
                                    .unwrap(),
                            ))
                            .unwrap();
                            Arc::new(set.build().unwrap())
                        };
                        let fragment_mesh_texture_set = {
                            let mut set = PersistentDescriptorSet::start(
                                pipeline
                                    .layout()
                                    .descriptor_set_layouts()
                                    .get(3)
                                    .unwrap()
                                    .clone(),
                            );
                            set.add_sampled_image(
                                Arc::new(
                                    ImageView::new(
                                        buffered_mesh
                                            .texture
                                            .as_ref()
                                            .map(Clone::clone)
                                            .unwrap_or_else(|| default_texture.clone()),
                                    )
                                    .unwrap(),
                                ),
                                texture_sampler.clone(),
                            )
                            .unwrap();
                            Arc::new(set.build().unwrap())
                        };
                        cmd.bind_descriptor_sets(
                            vulkano::pipeline::PipelineBindPoint::Graphics,
                            pipeline.layout().clone(),
                            2,
                            (
                                fragment_mesh_uniform_set.clone(),
                                fragment_mesh_texture_set.clone(),
                            ),
                        )
                        .bind_vertex_buffers(
                            0,
                            (
                                buffered_mesh.vertices.clone(),
                                CpuAccessibleBuffer::from_iter(
                                    graphics_state.device.clone(),
                                    BufferUsage::all(),
                                    false,
                                    instances
                                        .into_iter()
                                        .map(|entity| entity.world_matrix().as_ref().clone()),
                                )
                                .unwrap(),
                            ),
                        )
                        .bind_index_buffer(buffered_mesh.indices.clone())
                        .draw_indexed(buffered_mesh.index_count, instance_count, 0, 0, 0)
                        .unwrap()
                    },
                );
                cmd_builder.build().unwrap()
            })
            .finish()
    }

    fn initialization(&mut self, _: &EngineState<StateT>, graphics_state: &GraphicsState) {
        self.inner.initialize(|_| graphics_state.into());
    }

    fn window_resize(&mut self, _: &EngineState<StateT>, graphics_state: &GraphicsState) {
        self.inner.get_init_mut().pipeline = generate_pipeline(graphics_state)
    }

    fn termination(&mut self, _: &EngineState<StateT>, _: &GraphicsState) {}
}

mod buffered_mesh;
mod inner_state;
mod state_requirements;
