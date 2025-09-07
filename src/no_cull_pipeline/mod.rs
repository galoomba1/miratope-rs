//! Configures a render pipeline without
//! [backface culling](https://en.wikipedia.org/wiki/Back-face_culling), needed
//! so that most of the non-convex polytopes work properly.

use std::default::Default;
use bevy::{
    ecs::bundle::Bundle,
    prelude::{GlobalTransform, Transform, },
    render::render_resource::*,
};
use bevy::asset::{Asset, Handle};
use bevy::color::LinearRgba;
use bevy::image::Image;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::{AlphaMode, Component, Material, Mesh, TypePath, Visibility};
use bevy::render::mesh::MeshVertexBufferLayoutRef;
//paths to the shaders
const VERTEX_SHADER_ASSET_PATH: &str = "forward.vert";
const FRAGMENT_SHADER_ASSET_PATH: &str = "forward.frag";

/// [Handle<Mesh>] of a [Mesh] used in a Query.
/// Needs to be a Component (and a Newtype) to do so.
#[derive(Component)]
pub struct HandledMesh(pub(crate) Handle<Mesh>);

impl Default for HandledMesh{
    fn default() -> Self { HandledMesh(Default::default()) }
}

/// [Handle<StandardMaterial>] of an [TwoSidedMaterial] used in a Query.
/// Needs to be a Component (and a Newtype) to do so.
#[derive(Component)]
pub struct HandledMaterial(pub(crate) Handle<TwoSidedMaterial>);

impl Default for HandledMaterial{
    fn default() -> Self { HandledMaterial(Default::default()) }
}

// Code adapted from https://bevy.org/examples/shaders/shader-material-glsl/
/// Material that allows for rendering of both sides of each face
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TwoSidedMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

impl Material for TwoSidedMaterial {
    fn vertex_shader() -> ShaderRef {
        VERTEX_SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        FRAGMENT_SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    // Bevy assumes by default that vertex shaders use the "vertex" entry point
    // and fragment shaders use the "fragment" entry point (for WGSL shaders).
    // GLSL uses "main" as the entry point, so we must override the defaults here
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();
        Ok(())
    }
}

impl Default for TwoSidedMaterial{
    fn default() -> Self {
        TwoSidedMaterial{
            color: Default::default(),
            color_texture: None,
            alpha_mode: Default::default(),
        }
    }
}

#[derive(Bundle)] //the items here are changed to the most likely counterparts. It's probably not the right way to do this
pub struct PbrNoBackfaceBundle {
    pub mesh: HandledMesh,
    pub material: HandledMaterial,
    pub visible: Visibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for PbrNoBackfaceBundle {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            visible: Default::default(),
            material: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

