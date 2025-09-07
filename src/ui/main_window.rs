//! The systems that update the main window.

use super::config::{MeshColor, WfColor};
use super::right_panel::ElementTypesRes;
use super::{camera::ProjectionType, top_panel::SectionState};
use crate::mesh::Renderable;
use crate::Concrete;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContextSettings;
use miratope_core::abs::Ranked;
use crate::no_cull_pipeline::{HandledMaterial, HandledMesh, TwoSidedMaterial};

/// The plugin in charge of the Miratope main window, and of drawing the
/// polytope onto it.
pub struct MainWindowPlugin;

impl Plugin for MainWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_visible)
            .add_systems(Update, update_scale_factor)
            .add_systems(PostUpdate, update_changed_polytopes)
            .add_systems(PostUpdate, update_changed_color)
            .init_resource::<PolyName>();
    }
}

#[derive(Resource)]
pub struct PolyName(pub String);

impl Default for PolyName {
    fn default() -> PolyName {
        PolyName("default".to_string())
    }
}

pub fn update_visible(
    keyboard: Res<'_, ButtonInput<KeyCode>>,
    mut polies_vis: Query<'_, '_, &mut Visibility, With<Concrete>>,
    mut wfs_vis: Query<'_, '_, &mut Visibility, Without<Concrete>>,
) {
    if keyboard.get_pressed().count() == 1 {
        if keyboard.just_pressed(KeyCode::KeyV) {
            if let Some(visible) = polies_vis.iter_mut().next() {
                let vis =visible.into_inner();
                match vis{
                    Visibility::Inherited => {} //should never happen
                    Visibility::Hidden => { *vis = Visibility::Visible }
                    Visibility::Visible => { *vis = Visibility::Hidden }
                }
            }
        }

        if keyboard.just_pressed(KeyCode::KeyB) {
            if let Some(visible) = wfs_vis.iter_mut().next() {
                let vis =visible.into_inner();
                match vis {
                    Visibility::Inherited => {} //should never happen
                    Visibility::Hidden => { *vis = Visibility::Visible }
                    Visibility::Visible => { *vis = Visibility::Hidden }
                }
            }
        }
    }
}

/// Resizes the UI when the screen is resized.
pub fn update_scale_factor(mut egui_settings: Query<'_, '_, &mut EguiContextSettings>, window_query: Query<'_, '_, &Window, With<PrimaryWindow>>) {
    if let Ok(window) = window_query.single() {
        egui_settings.single_mut().unwrap().scale_factor = 1.0 / window.scale_factor();
    }
}

/// Updates polytopes after an operation.
pub fn update_changed_polytopes(
    mut meshes: ResMut<'_, Assets<Mesh>>,
    polies: Query<'_, '_, (&Concrete, &HandledMesh, &Children), Changed<Concrete>>,
    wfs: Query<'_, '_, &HandledMesh, Without<Concrete>>,
    mut window_query: Query<'_, '_, &mut Window, With<PrimaryWindow>>,
    mut section_state: ResMut<'_, SectionState>,
    mut element_types: ResMut<'_, ElementTypesRes>,
    name: Res<'_, PolyName>,

    orthogonal: Res<'_, ProjectionType>,
) -> Result {
    for (poly, mesh_handle, children) in polies.iter() {
        if cfg!(debug_assertions) {
            poly.assert_valid();
        }

        if !element_types.main_updating {
            element_types.main = false;
        } else {
            element_types.main_updating = false;
        }

        *meshes.get_mut(&mesh_handle.0).unwrap() = poly.mesh(*orthogonal);

        // Updates all wireframes.
        for child in children.iter() {
            let wf_handle = &wfs.get(child)?.0;
            *meshes.get_mut(wf_handle).unwrap() = poly.wireframe(*orthogonal);
        }

        // We reset the cross-section view if we didn't use it to change the polytope.
        if !section_state.is_changed() {
            section_state.close();
        }

        window_query
            .single_mut()?
            .title = format!("{} - Miratope v{}", name.0, env!("CARGO_PKG_VERSION"));

    }
    Ok(())
}

pub fn update_changed_color(
    mut materials: ResMut<'_, Assets<TwoSidedMaterial>>,
    mut polies: Query<'_, '_, &HandledMaterial, With<Concrete>>,
    mut wfs: Query<'_, '_, &HandledMaterial, Without<Concrete>>,
    mesh_color: Res<'_, MeshColor>,
    wf_color: Res<'_, WfColor>,
) {
    if let Some(material_handle) = polies.iter_mut().next() {
        *materials.get_mut(&material_handle.0).unwrap() = TwoSidedMaterial {
            color: LinearRgba::from(mesh_color.0),
            ..Default::default()
        };
    }
    if let Some(wf_handle) = wfs.iter_mut().next() {
        *materials.get_mut(&wf_handle.0).unwrap() = TwoSidedMaterial {
            color: LinearRgba::from(wf_color.0),
            ..Default::default()
        }
    }
}