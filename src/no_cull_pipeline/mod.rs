//! Configures a render pipeline without
//! [backface culling](https://en.wikipedia.org/wiki/Back-face_culling), needed
//! so that most of the non-convex polytopes work properly.

use std::default::Default;
use std::any::TypeId;
use bevy::{
    asset::{Assets, UntypedHandle, UntypedAssetId, uuid::Uuid,},
    ecs::bundle::Bundle,
    prelude::{Draw, GlobalTransform, RenderPipelines, Transform, Visible, Shader,},
    render::{
        render_graph::base::MainPass,
        shader::{Shader, ShaderStage, ShaderStages},
        render_resource::*,
    },
};
use bevy::core_pipeline::core_3d::graph::Node3d;
use bevy::prelude::Visibility;
use crate::mesh::{HandledMaterial, HandledMesh};

//This constant is probably not needed. Find a way to remove it
pub const NO_CULL_PIPELINE_HANDLE: UntypedHandle =
    UntypedHandle::Weak(UntypedAssetId::Uuid {
        type_id: TypeId::of::<RenderPipelineDescriptor>(),
        uuid: Uuid::from_u128(0x7CAE7047DEE79C847CAE7047DEE79C84)
    });
//this function will probably need a bunch of extra work. Also, do something to shaders
pub fn build_no_cull_pipeline(shaders: &mut Assets<Shader>) -> RenderPipelineDescriptor {
    RenderPipelineDescriptor {
        label: None,
        layout: vec![],
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
        vertex: VertexState{    //"forward.vert" needs to go in this somewhere
            shader: Default::default(),
            shader_defs: vec![],
            entry_point: Default::default(),
            buffers: vec![],
        },
        fragment: Some(FragmentState{   //"forward.frag" needs to go in this somewhere
            shader: Default::default(),
            shader_defs: vec![],
            entry_point: Default::default(),
            targets: vec![Some(ColorTargetState {
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
                write_mask: ColorWrites::ALL,
            })],
        }),

        push_constant_ranges: vec![],
        multisample: Default::default(),
        zero_initialize_workgroup_memory: false,
    }
}

#[derive(Bundle)] //the items here are changed to the most likely counterparts. It's probably not the right way to do this
pub struct PbrNoBackfaceBundle {
    pub mesh: HandledMesh,
    pub material: HandledMaterial,
    pub main_pass: Node3d::MainOpaquePass,
    pub draw: Draw,
    pub visible: Visibility,
    pub render_pipelines: RenderPipeline, //was RenderPipelines, that s is probably important
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for PbrNoBackfaceBundle {
    fn default() -> Self {
        Self {
            render_pipelines: RenderPipeline::from_pipelines(vec![RenderPipeline::new(
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
