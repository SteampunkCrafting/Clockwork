use std::mem::MaybeUninit;

use legion_ecs::prelude::Read;
use physics::prelude::{nalgebra::Vector4, RigidBody, RigidBodyHandle};
use scene_utils::components::{
    AmbientLight, Camera, DirectionalLight, PhongMaterial, PointLight, SpotLight,
};

use super::{fragment_shader, vertex_shader};

pub struct DrawMarker;

pub type CameraEntity = (Read<Camera>, Read<RigidBodyHandle>);
pub type DrawableEntity<MeshID> = (Read<DrawMarker>, Read<MeshID>, Read<RigidBodyHandle>);

pub fn make_vertex_uniforms(
    projection: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
) -> vertex_shader::ty::Data {
    vertex_shader::ty::Data { projection, view }
}

pub fn make_fragment_uniforms(
    (camera_component, camera_body): (Camera, RigidBody),
    PhongMaterial {
        ambient,
        diffuse,
        specular,
        specular_power,
    }: PhongMaterial,
    AmbientLight { color }: AmbientLight,
    dir_lights: &mut dyn Iterator<Item = DirectionalLight>,
    point_lights: &mut dyn Iterator<Item = (PointLight, RigidBody)>,
    spot_lights: &mut dyn Iterator<Item = (SpotLight, RigidBody)>,
) -> fragment_shader::ty::Data {
    use fragment_shader::ty::*;

    const DIR_LIGHTS_NUM: usize = 32;
    const POINT_LIGHTS_NUM: usize = 32;
    const SPOT_LIGHTS_NUM: usize = 32;

    fragment_shader::ty::Data {
        num_dir_lights: dir_lights.size_hint().0 as u32,
        num_point_lights: point_lights.size_hint().0 as u32,
        num_spot_lights: spot_lights.size_hint().0 as u32,

        material: PhongMaterial {
            ambient: ambient.into(),
            diffuse: diffuse.into(),
            specular: specular.into(),
            specular_power: specular_power.into(),
        },

        ambient_light: AmbientLight {
            color: color.clone().into(),
        },

        dir_lights: {
            let mut arr: [DirectionalLight; DIR_LIGHTS_NUM] =
                unsafe { MaybeUninit::uninit().assume_init() };
            dir_lights
                .take(DIR_LIGHTS_NUM)
                .map(|light| DirectionalLight {
                    view_direction: (camera_body.position().to_matrix().transpose()
                        * light.direction.fixed_resize::<4, 1>(0.0))
                    .fixed_resize::<3, 1>(0.0)
                    .into(),
                    color: light.color.into(),
                    _dummy0: Default::default(),
                    _dummy1: Default::default(),
                })
                .enumerate()
                .for_each(|(i, light)| arr[i] = light);
            arr
        },

        point_lights: {
            let mut arr: [PointLight; POINT_LIGHTS_NUM] =
                unsafe { MaybeUninit::uninit().assume_init() };
            point_lights
                .take(POINT_LIGHTS_NUM)
                .map(|(light, body)| PointLight {
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
                })
                .enumerate()
                .for_each(|(i, light)| arr[i] = light);
            arr
        },

        spot_lights: {
            let mut arr: [SpotLight; SPOT_LIGHTS_NUM] =
                unsafe { MaybeUninit::uninit().assume_init() };
            spot_lights
                .take(SPOT_LIGHTS_NUM)
                .map(|(light, body)| SpotLight {
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
                })
                .enumerate()
                .for_each(|(i, light)| arr[i] = light);
            arr
        },

        _dummy0: Default::default(),
        _dummy1: Default::default(),
        _dummy2: Default::default(),
        _dummy3: Default::default(),
        _dummy4: Default::default(),
    }
}
