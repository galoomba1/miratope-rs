//! Manages the memory tab.

use std::cmp::*;

use bevy::prelude::{Query, Res, ResMut, Resource};
use bevy_egui::{egui, EguiContext};

use crate::{
    ui::config::SlotsPerPage,
    Concrete
};

use super::main_window::PolyName;

/// Represents the memory slots to store polytopes.
#[derive(Default, Resource)]
pub struct Memory {
    pub slots: Vec<Option<(Concrete, Option<String>)>>,
    pub start_page: usize,
    pub end_page: usize
}

impl std::ops::Index<usize> for Memory {
    type Output = Option<(Concrete, Option<String>)>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.slots[index]
    }
}

/// The label for the `n`-th memory slot.
pub fn slot_label(n: usize) -> String {
    format!("polytope {}", n)
}

impl Memory {
    /// Returns the length of the memory vector.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Returns an iterator over the memory slots.
    pub fn iter(&self) -> std::slice::Iter<'_, Option<(Concrete, Option<String>)>> {
        self.slots.iter()
    }

    /// Appends an element.
    pub fn push(&mut self, a: (Concrete, Option<String>)) {
        self.slots.push(Some(a));
    }

    /// Shows the memory menu in a specified Ui.
    pub fn show(
        &mut self,
        query: &mut Query<'_, '_, &mut Concrete>,
        poly_name: &mut ResMut<'_, PolyName>,
        slots_per_page: &mut ResMut<'_, SlotsPerPage>,
        egui_ctx: &Res<'_, EguiContext>,
        open: &mut bool
    ) {
        let spp = slots_per_page.0;
        self.start_page = if self.len() < spp {0} else {min(self.start_page, self.len()-spp)};
        self.end_page = min(self.start_page + spp, self.len());
        egui::Window::new("Memory")
            .open(open)
            .scroll(true)
            .default_width(260.0)
            .show(egui_ctx.get(), |ui| {
            egui::containers::ScrollArea::vertical().show(ui, |ui| {
                
                ui.horizontal(|ui| {
                    if ui.button("Clear memory").clicked() {
                        self.slots.clear();
                    }
        
                    if ui.button("Add slot").clicked() {
                        self.slots.push(None);
                    }
                    
                    ui.add_space(20.);
                    ui.label("Slots per page:");
                    ui.add(
                        egui::DragValue::new(&mut slots_per_page.0)
                        .speed(0.04)
                        .range(1..=usize::MAX)
                    );
                });
    
                ui.separator();
    
                for idx in self.start_page..self.end_page {
                    if idx >= self.len() {continue}
                    let slot = &mut self.slots[idx];
                    match slot {
                        // Shows an empty slot.
                        None => {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", idx));
                                ui.label("Empty");

                                if ui.button("Save").clicked() {
                                    if let Some(p) = query.iter_mut().next() {
                                        *slot = Some((p.clone(), Some(poly_name.0.clone())));
                                    }
                                }
                             });
                        }

                        // Shows a slot with a polytope on it.
                        Some((poly, label)) => {
                            let mut clear = false;

                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", idx));
                                let name = match label {
                                    None => {
                                        slot_label(idx)
                                    }
                                    
                                    Some(name) => {
                                        name.to_string()
                                    }
                                };

                                ui.label(&name);

                                // Clones a polytope from memory.
                                if ui.button("Load").clicked() {
                                    *query.iter_mut().next().unwrap() = poly.clone();
                                    poly_name.0 = name.clone();
                                }

                                // Swaps the current polytope with the one on memory.
                                if ui.button("Swap").clicked() {
                                    std::mem::swap(query.iter_mut().next().unwrap().as_mut(), poly);
                                    *label = Some(poly_name.0.clone());
                                    poly_name.0 = name;
                                }

                                // Clones a polytope into memory.
                                if ui.button("Save").clicked() {
                                    *poly = query.iter_mut().next().unwrap().clone();
                                    *label = Some(poly_name.0.clone());
                                }

                                // Clears a polytope from memory.
                                if ui.button("Clear").clicked() {
                                    clear = true;
                                }
                            });

                            if clear {
                                *slot = None;
                            }
                        }
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    let len = self.len(); // there's probably a better way to do this but idk rust
                    ui.add(
                        egui::DragValue::new(&mut self.start_page)
                        .suffix(format!(" - {}", self.end_page as isize - 1))
                        .speed(0.4)
                        .range(0..=if len < spp {0} else {len-spp})
                    );
                    ui.label(format!(
                        "/  {}",
                        self.len()
                    ));
                });
            });
        });
    }
}
