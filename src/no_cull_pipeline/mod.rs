//! Configures a render pipeline without
//! [backface culling](https://en.wikipedia.org/wiki/Back-face_culling), needed
//! so that most of the non-convex polytopes work properly.

use bevy::{
    asset::{Assets, Handle, UntypedHandle},
    ecs::bundle::Bundle,
    prelude::{Draw, GlobalTransform, RenderPipelines, StandardMaterial, Transform, Visible},
    render::{
        mesh::Mesh,
        pipeline::*,
        render_graph::base::MainPass,
        shader::{Shader, ShaderStage, ShaderStages},
        texture::TextureFormat,
    },
};
use bevy::prelude::Shader;
use bevy::render::render_resource::*;

pub const NO_CULL_PIPELINE_HANDLE: UntypedHandle =
    UntypedHandle::Weak(PipelineDescriptor::TYPE_UUID, 0x7CAE7047DEE79C84); //The whole UUID system seems to have been completely overhauled

pub fn build_no_cull_pipeline(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
    PipelineDescriptor {
        primitive: PrimitiveState {
            front_face: FrontFace::Ccw,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState {
                front: StencilFaceState::IGNORE,
                back: StencilFaceState::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
            bias: DepthBiasState {
                constant: 0,
                slope_scale: 0.0,
                clamp: 0.0,
            },
        }),
        color_target_states: vec![ColorTargetState {
            format: Default::default(),
            blend: Some(BlendState {
                alpha: BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                color: BlendComponent {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
                },
            }),
            write_mask: ColorWrite::ALL,
        }],
        ..PipelineDescriptor::new(ShaderStages {
            vertex: shaders.add(Shader::from_glsl(
                ShaderStage::Vertex,
                include_str!("forward.vert"),
            )),
            fragment: Some(shaders.add(Shader::from_glsl(
                ShaderStage::Fragment,
                include_str!("forward.frag"),
            ))),
        })
    }
}

#[derive(Bundle)]
pub struct PbrNoBackfaceBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for PbrNoBackfaceBundle {
    fn default() -> Self {
        Self {
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                NO_CULL_PIPELINE_HANDLE.typed(),
            )]),
            mesh: Default::default(),
            visible: Default::default(),
            material: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}
