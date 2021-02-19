#![allow(dead_code)]

//! A renderer for polytopes. Still in alpha.
//!
//! ## What can Miratope do now?
//! Miratope can generate these classes of polytopes, among others:
//! * Simplexes
//! * Hypercubes
//! * Orthoplexes
//! * Product prisms
//! * Polyhedral antiprisms
//! * Cupolae, cuploids and cupolaic blends
//!
//! Miratope can also read and export OFF files and GGB files.
//!
//! ## FAQ
//! ### How do I use Miratope?
//! Miratope doesn't have an interface yet, so you'll have to use the Console to write down JavaScript commands.
//!
//! Most of the cool generating commands are on the `Build` class. For example, to generate a uniform octagrammic antiprism and render it to the screen, you can use `Build.uniformAntiprism(8, 3).renderTo(mainScene);`.
//!
//! Here's some other commands to try out:
//! ```javascript
//! //Renders a cube on screen.
//! Build.hypercube(3).renderTo(mainScene);
//!
//! //OFF file for a pentagon-pentagram duoprism.
//! Product.prism(Build.regularPolygon(5), Build.regularPolygon(5, 2)).saveAsOFF();
//!
//! //Exports a hexadecachoral prism as a GeoGebra file.
//! Build.cross(4).extrudeToPrism(1).saveAsGGB();
//! ```
//!
//! ### Where do I get these "OFF files"?
//! The OFF file format is a format for storing certain kinds of geometric shapes. Although not in widespread use, it has become the standard format for those who investigate polyhedra and polytopes. It was initially meant for the [Geomview software](https://people.sc.fsu.edu/~jburkardt/data/off/off.html), and was later adapted for the [Stella software](https://www.software3d.com/StellaManual.php?prod=stella4D#import). Miratope uses a further generalization of the Stella OFF format for any amount of dimensions.
//!
//! Miratope does not yet include a library of OFF files. Nevertheless, many of them can be downloaded from [OfficialURL's personal collection](https://drive.google.com/drive/u/0/folders/1nQZ-QVVBfgYSck4pkZ7he0djF82T9MVy). Eventually, they'll be browsable from Miratope itself.
//!
//! ### Why does my OFF file not render?
//! Provisionally, your OFF file is being loaded into the variable `P`. You have to manually render it using the command `P.renderTo(mainScene);`.
//!
//! Note that at the moment, this works only for 3D OFF files, and can be somewhat buggy.
//!
//! ### How do I clear the scene?
//! Use `mainScene.clear();`.
//!
//! ## What's next?
//! There are lots of planned features for Miratope, some more ambitious than others. You can look at the complete list, along with some ideas on how to implement them [here](https://docs.google.com/document/d/1IEoXR4vmOPELFKosRMIDfDN_M4oaUGWDExdqqDpCwfU/edit?usp=sharing).
//!
//! The most immediate changes will probably be the following:
//! * Greater camera control
//! * Vertex and edge toggling
//! * Projection type options
//!
//! Longer term but more substantial changes include:
//! * Localization
//! * A minimal working interface
//! * 4D+ rendering
//! * Different fill types for faces
//! * Creation of a dedicated file format and a polytope library
//! * More operations on polytopes

use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::camera::Camera;
use bevy::render::pipeline::PipelineDescriptor;
use no_cull_pipeline::PbrNoBackfaceBundle;
use polytope::shapes::*;
use polytope::*;

mod no_cull_pipeline;
mod polytope;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(spin_camera.system())
        .run();
}

const WIREFRAME_SELECTED_MATERIAL: HandleUntyped =
    HandleUntyped::weak_from_u64(StandardMaterial::TYPE_UUID, 0x82A3A5DD3A34CC21);
const WIREFRAME_UNSELECTED_MATERIAL: HandleUntyped =
    HandleUntyped::weak_from_u64(StandardMaterial::TYPE_UUID, 0x82A3A5DD3A34CC22);

fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
) {
    let poly: Polytope = antiprism(12, 2);

    pipelines.set_untracked(
        no_cull_pipeline::NO_CULL_PIPELINE_HANDLE,
        no_cull_pipeline::build_no_cull_pipeline(&mut shaders),
    );

    materials.set_untracked(
        WIREFRAME_SELECTED_MATERIAL,
        Color::rgb_u8(126, 192, 236).into(),
    );

    let wf_unselected = materials.set(
        WIREFRAME_UNSELECTED_MATERIAL,
        Color::rgb_u8(56, 68, 236).into(),
    );

    commands
        .spawn(PbrNoBackfaceBundle {
            mesh: meshes.add(poly.get_mesh()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
            ..Default::default()
        })
        .with_children(|cb| {
            cb.spawn(PbrNoBackfaceBundle {
                mesh: meshes.add(poly.get_wireframe()),
                material: wf_unselected,
                ..Default::default()
            });
        })
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(-2.0, 2.5, 2.0)),
            ..Default::default()
        })
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(-2.0, 2.5, 5.0))
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        });
}

fn spin_camera(mut query: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    const SPIN_RATE: f32 = std::f32::consts::PI * 2.0 / 3.0;

    for mut tf in query.iter_mut() {
        tf.translation = Quat::from_rotation_y(time.delta_seconds() * SPIN_RATE) * tf.translation;
        tf.look_at(Vec3::zero(), Vec3::unit_y());
    }
}
