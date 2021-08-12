use std::mem::MaybeUninit;

use legion_ecs::prelude::Read;
use physics::prelude::{nalgebra::Vector4, RigidBody, RigidBodyHandle};
use scene_utils::components::{
    AmbientLight, Camera, DirectionalLight, PhongMaterial, PointLight, SpotLight,
};

use super::{fragment_shader, vertex_shader};

pub struct DrawMarker;

pub type CameraEntity = (Read<Camera>, Read<RigidBodyHandle>);
pub type AmbientLightEntity = (Read<DrawMarker>, Read<AmbientLight>);
pub type DirectionalLightEntity = (Read<DrawMarker>, Read<DirectionalLight>);
pub type PointLightEntity = (Read<DrawMarker>, Read<PointLight>, Read<RigidBodyHandle>);
pub type SpotLightEntity = (Read<DrawMarker>, Read<SpotLight>, Read<RigidBodyHandle>);
pub type DrawableEntity<MeshID> = (Read<DrawMarker>, Read<MeshID>, Read<RigidBodyHandle>);

pub fn make_vertex_uniforms(
    projection: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
) -> vertex_shader::ty::Data {
    vertex_shader::ty::Data { projection, view }
}

pub fn make_mesh_fragment_uniforms(
    PhongMaterial {
        ambient,
        diffuse,
        specular,
        specular_power,
    }: PhongMaterial,
) -> fragment_shader::ty::DataMesh {
    use fragment_shader::ty::*;
    DataMesh {
        material: PhongMaterial {
            ambient: ambient.into(),
            diffuse: diffuse.into(),
            specular: specular.into(),
            specular_power: specular_power.into(),
        },
    }
}

pub fn make_world_fragment_uniforms(
    (camera_component, camera_body): (Camera, RigidBody),
    AmbientLight { color }: AmbientLight,
    dir_lights: Vec<DirectionalLight>,
    point_lights: Vec<(PointLight, RigidBody)>,
    spot_lights: Vec<(SpotLight, RigidBody)>,
) -> fragment_shader::ty::DataWorld {
    use fragment_shader::ty::*;
    unsafe {
        fragment_shader::ty::DataWorld {
            num_dir_lights: dir_lights.len() as u32,
            num_point_lights: point_lights.len() as u32,
            num_spot_lights: spot_lights.len() as u32,
            ambient_light: AmbientLight {
                color: color.clone().into(),
            },
            dir_lights: partially_init_array(
                |l| DirectionalLight {
                    view_direction: (camera_body.position().to_matrix().transpose()
                        * l.direction.fixed_resize::<4, 1>(0.0))
                    .fixed_resize::<3, 1>(0.0)
                    .into(),
                    color: l.color.into(),
                    _dummy0: Default::default(),
                    _dummy1: Default::default(),
                },
                dir_lights,
            ),
            point_lights: partially_init_array(
                |(light, body)| PointLight {
                    view_position: camera_body
                        .position()
                        .inv_mul(body.position())
                        .translation
                        .into(),
                    color: light.color.into(),
                    attenuation: Attenuation {
                        constant: light.attenuation.constant,
                        linear: light.attenuation.linear,
                        exponent: light.attenuation.quadratic,
                    },
                    _dummy0: Default::default(),
                    _dummy1: Default::default(),
                    _dummy2: Default::default(),
                },
                point_lights,
            ),
            spot_lights: partially_init_array(
                |(light, body)| SpotLight {
                    opening_angle_rad: light.opening_angle,
                    view_position: camera_body
                        .position()
                        .inv_mul(body.position())
                        .translation
                        .into(),
                    view_direction: (camera_body
                        .position()
                        .inv_mul(body.position())
                        .inverse()
                        .to_matrix()
                        .transpose()
                        * Vector4::from([0.0, 0.0, -1.0, 0.0]))
                    .fixed_resize::<3, 1>(0.0)
                    .into(),
                    color: light.color.into(),
                    attenuation: Attenuation {
                        constant: light.attenuation.constant,
                        linear: light.attenuation.linear,
                        exponent: light.attenuation.quadratic,
                    },
                    _dummy0: Default::default(),
                    _dummy1: Default::default(),
                    _dummy2: Default::default(),
                    _dummy3: Default::default(),
                    _dummy4: Default::default(),
                },
                spot_lights,
            ),
            _dummy0: Default::default(),
            _dummy1: Default::default(),
            _dummy2: Default::default(),
            _dummy3: Default::default(),
        }
    }
}

/// Creates an uninitialized array on the stack, and then moves the contents
/// of the input collection into this array, until either the array is full,
/// or the collection ends.
///
/// If the size of the collection is less than the size of the array, then
/// part of the array remains uninitialized (even if the return type states the opposite).
///
/// Dropping uninitialized structures will cause undefined behavior, if they contain references
/// as fields.
unsafe fn partially_init_array<T, U, const N: usize>(
    into: impl Fn(T) -> U,
    mut ts: impl IntoIterator<Item = T>,
) -> [U; N] {
    let mut arr: [U; N] = MaybeUninit::uninit().assume_init();
    ts.into_iter()
        .take(N)
        .map(into)
        .enumerate()
        .for_each(|(i, u)| arr[i] = u);
    arr
}
