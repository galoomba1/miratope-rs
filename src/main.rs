#![deny(
    missing_docs,
    nonstandard_style,
    unused_parens,
    unused_qualifications,
    rust_2018_idioms,
    rust_2018_compatibility,
    future_incompatible,
    missing_copy_implementations
)]

//! A tool for building and visualizing polytopes. Still in alpha development.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use miratope_core::file::FromFile;

use ui::{
    camera::{CameraInputEvent, ProjectionType},
    MiratopePlugins,
};

use crate::mesh::Renderable;

mod mesh;
mod ui;

/// The link to the [Polytope Wiki](https://polytope.miraheze.org/wiki/).
pub const WIKI_LINK: &str = "https://polytope.miraheze.org/wiki/";

/// The floating-point type for the entire application. Can be either `f32` or
/// `f64`, and it should compile the same.
type Float = f64;

/// A [`Concrete`](miratope_core::conc::Concrete) polytope with the floating
/// type for the application.
type Concrete = miratope_core::conc::Concrete;

/// A [`Point`](miratope_core::geometry::Point) with the floating type
/// for the application.
type Point = miratope_core::geometry::Point<f64>;

/// A [`Vector`](miratope_core::geometry::Vector) with the floating
/// type for the application.
type Vector = miratope_core::geometry::Vector<f64>;

/// A [`Hypersphere`](miratope_core::geometry::Hypersphere) with the
/// floating type for the application.
type Hypersphere = miratope_core::geometry::Hypersphere<f64>;

/// A [`Hyperplane`](miratope_core::geometry::Hyperplane) with the
/// floating type for the application.
type Hyperplane = miratope_core::geometry::Hyperplane<f64>;

/// The default epsilon value throughout the application.
const EPS: Float = <Float as miratope_core::float::Float>::EPS;

/// Loads all of the necessary systems for the application to run.
fn main() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "full"); }
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(MiratopePlugins)
        .add_systems(Startup, setup)
        .run();
}

/// Initializes the scene.
fn setup(
    mut commands: Commands<'_, '_>,
    mut meshes: ResMut<'_, Assets<Mesh>>,
    mut materials: ResMut<'_, Assets<StandardMaterial>>,
) { // The error seems to be in this function.
    // Default polytope.
    let poly = Concrete::from_off(include_str!("default.off")).unwrap();


    // Selected object (unused as of yet).
    materials.add( StandardMaterial {
        base_color: Color::srgb_u8(126, 192, 255),
        double_sided: true,
        cull_mode: None,
        ..Default::default()
    });

    // Wireframe material. (WIREFRAME UNSELECTED MATERIAL)
    let wf_material = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(150, 150, 150),
        double_sided: true,
        cull_mode: None,
        ..Default::default()
    });

    // Mesh material.
    let mesh_material = materials.add(
        StandardMaterial {
        base_color: Color::srgb_u8(255, 255, 255),
        double_sided: true,
        cull_mode: None,
        ..Default::default()
    });

    // Camera configuration.
    let mut cam_anchor = Default::default();
    let mut cam = Default::default();
    CameraInputEvent::reset(&mut cam_anchor, &mut cam);

    commands
        // Mesh
        .spawn((
            Mesh3d(meshes.add(poly.mesh(ProjectionType::Perspective))),
            MeshMaterial3d(mesh_material),
            Transform::default(),
            Visibility::Visible,
        ))
        // Wireframe
        .with_children(|cb| {
            cb.spawn((
                Mesh3d(meshes.add(poly.wireframe(ProjectionType::Perspective))),
                MeshMaterial3d(wf_material),
                Transform::default(),
                Visibility::Visible,
            ));
        })
        // Polytope
        .insert(poly);

    // Camera anchor
    commands
        .spawn((GlobalTransform::default(), cam_anchor, InheritedVisibility::VISIBLE))
        .with_children(|cb| {
            // Camera
            cb.spawn((
                Camera3d::default(),
                cam,
                Msaa::Sample4,

            ));
            // Light sources
            cb.spawn((
                Transform::from_translation(Vec3::new(-5., 5., 5.)),
                PointLight::default(),
            ));
            cb.spawn((
                Transform::from_translation(Vec3::new(5., 5., 5.)),
                PointLight::default(),
            ));
            cb.spawn((
                Transform::from_translation(Vec3::new(0., 5., -5.)),
                PointLight::default(),
            ));
        });
}

