use asset_storage::asset_storage::AssetStorageKey;
use kernel::graphics::light_components::*;
use kernel::math::Vec3;
use kernel::*;
use kernel::{
    abstract_runtime::Delegate,
    graphics::{
        scene_object::{Camera, Material, Mesh},
        scene_object_components::ProjectionMatrix,
        AmbientLight, DirectionalLight, Light, PointLight, SceneObject, SpotLight,
    },
    math::Mat4,
};
use physics::prelude::RigidBody;

struct Body(RigidBody);
impl SceneObject for Body {
    fn world_matrix(&self) -> kernel::math::Mat4 {
        Into::<[[f32; 4]; 4]>::into(self.0.position().to_matrix()).into()
    }

    fn view_matrix(&self) -> kernel::math::Mat4 {
        Into::<[[f32; 4]; 4]>::into(self.0.position().inverse().to_matrix()).into()
    }

    fn normal_matrix(&self) -> kernel::math::Mat4 {
        Into::<[[f32; 4]; 4]>::into(self.0.position().inverse().to_matrix().transpose()).into()
    }
}

#[derive(Delegate)]
#[delegate(SceneObject, target = "body")]
pub struct SceneInstance<I: AssetStorageKey> {
    asset_id: I,
    body: Body,
}

impl<I: AssetStorageKey> From<(I, RigidBody)> for SceneInstance<I> {
    fn from((asset_id, body): (I, RigidBody)) -> Self {
        Self {
            asset_id,
            body: Body(body),
        }
    }
}

impl<I: AssetStorageKey> Mesh<I> for SceneInstance<I> {
    fn mesh_id(&self) -> I {
        self.asset_id.clone()
    }
}

impl<I: AssetStorageKey> Material<I> for SceneInstance<I> {
    fn material_id(&self) -> I {
        self.asset_id.clone()
    }
}

#[derive(Delegate)]
#[delegate(SceneObject, target = "body")]
#[delegate(ProjectionMatrix, target = "camera")]
pub struct SceneCamera {
    camera: scene::components::Camera,
    body: Body,
}
impl From<(scene::components::Camera, RigidBody)> for SceneCamera {
    fn from((camera, body): (scene::components::Camera, RigidBody)) -> Self {
        Self {
            camera,
            body: Body(body),
        }
    }
}
impl Camera for SceneCamera {}

#[derive(Delegate)]
#[delegate(AmbientLight)]
#[delegate(Color)]
#[delegate(Light)]
pub struct SceneAmbientLight {
    inner: scene::components::AmbientLight,
}
impl From<scene::components::AmbientLight> for SceneAmbientLight {
    fn from(inner: scene::components::AmbientLight) -> Self {
        Self { inner }
    }
}

#[derive(Delegate)]
#[delegate(DirectionalLight)]
#[delegate(Direction)]
#[delegate(Color)]
#[delegate(Light)]
pub struct SceneDirectionalLight {
    inner: scene::components::DirectionalLight,
}
impl From<scene::components::DirectionalLight> for SceneDirectionalLight {
    fn from(inner: scene::components::DirectionalLight) -> Self {
        Self { inner }
    }
}

#[derive(Delegate)]
#[delegate(SceneObject, target = "body")]
#[delegate(Attenuation, target = "inner")]
#[delegate(Color, target = "inner")]
#[delegate(Intensity, target = "inner")]
#[delegate(Light, target = "inner")]
pub struct ScenePointLight {
    inner: scene::components::PointLight,
    body: Body,
}
impl From<(scene::components::PointLight, RigidBody)> for ScenePointLight {
    fn from((inner, b): (scene::components::PointLight, RigidBody)) -> Self {
        Self {
            inner,
            body: Body(b),
        }
    }
}
impl PointLight for ScenePointLight {}

#[derive(Delegate)]
#[delegate(SceneObject, target = "body")]
#[delegate(Attenuation, target = "inner")]
#[delegate(Color, target = "inner")]
#[delegate(Intensity, target = "inner")]
#[delegate(Light, target = "inner")]
#[delegate(OpeningAngle, target = "inner")]
pub struct SceneSpotLight {
    inner: scene::components::SpotLight,
    body: Body,
}
impl From<(scene::components::SpotLight, RigidBody)> for SceneSpotLight {
    fn from((inner, body): (scene::components::SpotLight, RigidBody)) -> Self {
        Self {
            inner,
            body: Body(body),
        }
    }
}
impl SpotLight for SceneSpotLight {}
