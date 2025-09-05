//! Contains the methods that take a polytope and turn it into a mesh.

use std::collections::HashMap;

use crate::ui::camera::ProjectionType;
use crate::{Concrete, Float, Point, EPS};

use bevy::{
    prelude::Mesh,
    render::{mesh::Indices, mesh::PrimitiveTopology},
};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::{Component, Handle, StandardMaterial};
use lyon::{math::point, path::Path, tessellation::*};
use miratope_core::conc::cycle::{Cycle, CycleList};
use miratope_core::{
    abs::{ElementList, Ranked},
    conc::ConcretePolytope,
    geometry::{Subspace, Vector},
};

use vec_like::*;

/// [Handle<Mesh>] of a [Mesh] used in a Query.
/// Needs to be a Component (and a Newtype) to do so.
#[derive(Component)]
pub struct HandledMesh(pub(crate) Handle<Mesh>);

impl Default for HandledMesh{
    fn default() -> Self { HandledMesh(Default::default()) }
}

/// [Handle<StandardMaterial>] of an [StandardMaterial] used in a Query.
/// Needs to be a Component (and a Newtype) to do so.
#[derive(Component)]
pub struct HandledMaterial(pub(crate) Handle<StandardMaterial>);

impl Default for HandledMaterial{
    fn default() -> Self { HandledMaterial(Default::default()) }
}

/// Attempts to turn the cycle into a 2D path, which can then be given to
/// the tessellator. Uses the specified vertex list to grab the coordinates
/// of the vertices on the path.
///
/// If the cycle isn't 2D, we return `None`.
pub fn path(cycle: &Cycle, vertices: &[Point]) -> Option<Path> {
    let mut builder = Path::builder();
    let cycle_iter = cycle.iter().map(|&idx| &vertices[idx]);

    // We don't bother with any polygons that aren't in 2D space.
    let s = Subspace::from_points_with(cycle_iter.clone(), 2)?;
    if s.rank() != 2 {
        return None
    }

    let mut flat_points = cycle_iter.map(|p| s.flatten(&p));

    let path_point = |v: &Point| point(v[0] as f32, v[1] as f32);

    // We build a path from the polygon.
    let v = flat_points.next().unwrap();
    builder.begin(path_point(&v));

    for v in flat_points {
        builder.line_to(path_point(&v));
    }

    builder.end(true);

    Some(builder.build())
}

/// Represents a triangulation of the faces of a [`Concrete`]. It stores the
/// vertex indices that make up the triangulation of the polytope, as well as
/// the extra vertices that may be needed to represent it.
struct Triangulation {
    /// Extra vertices that might be needed for the triangulation.
    extra_vertices: Vec<Point>,

    /// Indices of the vertices that make up the triangles.
    triangles: Vec<u32>,
}

impl Triangulation {
    /// Creates a new triangulation from a polytope.
    fn new(polytope: &Concrete) -> Self {
        let mut extra_vertices = Vec::new();
        let mut triangles = Vec::new();
        let empty_els = ElementList::new();

        // Either returns a reference to the element list of a given rank, or
        // returns a reference to an empty element list.
        let elements_or = |r| polytope.get_element_list(r).unwrap_or(&empty_els);

        let edges = elements_or(2);
        let faces = elements_or(3);

        let concrete_vertex_len = polytope.vertices.len() as u32;

        // We render each face separately.
        for face in faces {
            // We tesselate this path.
            let cycles = CycleList::from_edges(face.subs.iter().map(|&i| &edges[i].subs));
            for cycle in cycles {
                if let Some(path) = path(&cycle, &polytope.vertices) {
                    let mut geometry: VertexBuffers<_, u32> = VertexBuffers::new();

                    // Configures all of the options of the tessellator.
                    FillTessellator::new()
                        .tessellate_with_ids(
                            path.id_iter(),
                            &path,
                            None,
                            &FillOptions::with_fill_rule(Default::default(), FillRule::NonZero)
                                .with_tolerance(EPS as f32),
                            &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex<'_>| {
                                vertex.sources().next().unwrap()
                            }),
                        )
                        .unwrap();

                    // Maps EndpointIds to the indices in the original vertex list.
                    let mut id_to_idx = Vec::new();
                    for idx in cycle {
                        id_to_idx.push(idx);
                    }

                    // We map the output vertices to the original ones, and add any
                    // extra vertices that may be needed.
                    let mut vertex_hash = HashMap::new();

                    for (new_id, vertex_source) in geometry.vertices.into_iter().enumerate() {
                        let new_id = new_id as u32;

                        match vertex_source {
                            // This is one of the concrete vertices of the polytope.
                            VertexSource::Endpoint { id } => {
                                vertex_hash.insert(new_id, id_to_idx[id.to_usize()] as u32);
                            }

                            // This is a new vertex that has been added to the tesselation.
                            VertexSource::Edge { from, to, t } => {
                                let from = &polytope.vertices[id_to_idx[from.to_usize()]];
                                let to = &polytope.vertices[id_to_idx[to.to_usize()]];

                                let t = t as Float;
                                let p = from * (1.0 - t) + to * t;

                                vertex_hash
                                    .insert(new_id, concrete_vertex_len + extra_vertices.len() as u32);

                                extra_vertices.push(p);
                            }
                        }
                    }

                    // Add all of the new indices we've found onto the triangle vector.
                    for new_idx in geometry
                        .indices
                        .iter()
                        .map(|idx| *vertex_hash.get(idx).unwrap())
                    {
                        triangles.push(new_idx);
                    }
                }
            }
        }

        Self {
            extra_vertices,
            triangles,
        }
    }
}

/// Generates normals from a set of vertices by just projecting radially from
/// the origin.
fn normals(vertices: &[[f32; 3]]) -> Vec<[f32; 3]> {
    vertices
        .iter()
        .map(|n| {
            let sq_norm = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
            if sq_norm < EPS as f32 {
                [0.0, 0.0, 0.0]
            } else {
                let norm = sq_norm.sqrt();
                n.map(|c| c / norm)
            }
        })
        .collect()
}

/// Returns an empty mesh.
fn empty_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0; 3]])
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0; 3]])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0; 2]])
        .with_inserted_indices(Indices::U16(Vec::new()))
}

/// Gets the coordinates of the vertices, after projecting down into 3D.
fn vertex_coords<'a, I: Iterator<Item = &'a Point>>(
    poly: &Concrete,
    vertices: I,
    projection_type: ProjectionType,
) -> Vec<[f32; 3]> {
    let dim = poly.dim_or();

    // Returns the ith coordinate of p, or 0 if it doesn't exist.
    let coord = |p: &Point, i: usize| p.get(i).copied().unwrap_or_default();

    // If the polytope is at most 3D, we just embed it into 3D space.
    if projection_type.is_orthogonal() || dim <= 3 {
        vertices.map(|p| [0, 1, 2].map(|i| coord(p, i) as f32)).collect()
    }
    // Else, we project it down.
    else {
        // Distance from the projection planes.
        let mut direction = Vector::zeros(dim);
        direction[3] = 1.0;

        let (min, max) = poly.minmax(direction).unwrap();
        let dist = (min as f32 - 1.0).abs().max(max as f32 + 1.0).abs();

        vertices
            .map(|p| {
                // We scale the first three coordinates accordingly.
                let factor: f32 = p.iter().skip(3).map(|&x| x as f32 + dist).product();
                [0, 1, 2].map(|i| coord(p, i) as f32 / factor)
            })
            .collect()
    }
}

/// A trait for a polytope for which we can build a mesh.
pub trait Renderable: ConcretePolytope {
    /// Builds the mesh of a polytope.
    fn mesh(&self, projection_type: ProjectionType) -> Mesh {
        // If there's no vertices, returns an empty mesh.
        if self.vertex_count() == 0 {
            return empty_mesh();
        }

        // Triangulates the polytope's faces, projects the vertices of both the
        // polytope and the triangulation.
        let triangulation = Triangulation::new(self.con());
        let vertices = vertex_coords(
            self.con(),
            self.vertices()
                .iter()
                .chain(triangulation.extra_vertices.iter()),
            projection_type,
        );

        // Builds the actual mesh.
        Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 1.0]; vertices.len()])
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals(&vertices))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_indices(Indices::U32(triangulation.triangles))
    }

    /// Builds the wireframe of a polytope.
    fn wireframe(&self, projection_type: ProjectionType) -> Mesh {
        let vertex_count = self.vertex_count();

        // If there's no vertices, returns an empty mesh.
        if vertex_count == 0 {
            return empty_mesh();
        }

        let edge_count = self.edge_count();

        // We add a single vertex so that Miratope doesn't crash.
        let vertices = vertex_coords(self.con(), self.vertices().iter(), projection_type);
        let mut indices = Vec::with_capacity(edge_count * 2);

        // Adds the edges to the wireframe.
        if let Some(edges) = self.get_element_list(2) {
            for edge in edges {
                debug_assert_eq!(
                    edge.subs.len(),
                    2,
                    "Edge must have exactly 2 elements, found {}.",
                    edge.subs.len()
                );

                indices.push(edge.subs[0] as u16);
                indices.push(edge.subs[1] as u16);
            }
        }

        // Sets the mesh attributes.
        Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals(&vertices))
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0; 2]; vertex_count])
            .with_inserted_indices(Indices::U16(indices))
    }
}

impl<U: ConcretePolytope> Renderable for U {}
