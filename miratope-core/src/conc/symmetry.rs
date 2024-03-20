//! The code used to get the symmetry of a polytope and do operations based on that.

use std::{collections::{BTreeMap, HashSet}, vec, iter::FromIterator};

use crate::{
    abs::{Ranked, flag::{FlagIter, Flag}},
    conc::Concrete,
    float::Float,
    group::Group,
    geometry::{Matrix, Point, PointOrd, Subspace},
    Polytope,
};

use vec_like::*;

use super::ConcretePolytope;

impl Flag {
    /// Outputs a sequence of vertices obtained from applying a fixed sequence of flag changes to a flag.
    /// Used for computing the elements of a symmetry group. 
    fn vertex_sequence(&mut self, p: &Concrete) -> Matrix<f64> {
        let rank = p.rank();
        let mut basis = Matrix::<f64>::zeros(rank-1,rank-1);
        let mut columns = basis.column_iter_mut();
        let vertex = &p.vertices[self[1]];

        columns.next().unwrap().copy_from(&vertex);
        for mut col in columns {
            for r in 1..rank {
                self.change_mut(&p.abs, r);
            }
            let vertex = &p.vertices[self[1]];
            col.copy_from(&vertex);
        }

        basis
    }
}

impl Concrete {
    /// Computes the symmetry group of a polytope, including reflections.
    pub fn get_symmetry_group(&mut self) -> Option<(Group<vec::IntoIter<Matrix<f64>>>, Vec<Vec<usize>>)> {
        return self.get_symmetry_group_with(false);
    }

    /// Computes the symmetry group of a polytope, along with a list of vertex mappings with the option to exclude reflections or not.
    pub fn get_symmetry_group_with(&mut self, only_rotations: bool) -> Option<(Group<vec::IntoIter<Matrix<f64>>>, Vec<Vec<usize>>)> {
        self.element_sort();

        // Iterate through flags until we find a non-hemi facet.
        let mut facet_idx: usize = 0;
        while facet_idx < self.facet_count() {
            let facet_space = Subspace::from_points(
                self.abs.element_and_vertices(self.rank()-1, facet_idx).unwrap().0.iter().map(|x| &self.vertices[*x])
            );
            if facet_space.distance(&Point::zeros(self.dim().unwrap())) > f64::EPS {
                break;
            }
            facet_idx += 1;
        }
        if facet_idx == self.facet_count() {
            println!("Symmetry calculation failed. All facets pass through the origin.");
            return None
        }

        let mut base_flag = Flag::with_capacity(self.rank() + 1);
        base_flag.push(0); // maximal element
        let mut idx = facet_idx;
        base_flag.push(idx); // facet

        for r in (1..=self.rank()-1).rev() {
            idx = self[(r, idx)].subs[0];
            base_flag.push(idx);
        }

        base_flag.reverse();

        let flag_iter = FlagIter::new(&self.abs);
        let (types, types_map_back) = &self.element_types_common();

        let mut vertices_pointord = Vec::<PointOrd<f64>>::new();
        for v in &self.vertices {
            vertices_pointord.push(PointOrd::new(v.clone()));
        }
        let vertices = BTreeMap::from_iter((vertices_pointord).into_iter().zip(0..));
        let mut vertex_map: Vec<Vec<usize>> = Vec::new();

        // Sets of elements' vertex sets.
        let mut elements = vec![Vec::<HashSet<Vec<usize>>>::new(); self.rank()];
        for i in 2..self.rank() {
            let mut of_this_rank = vec![HashSet::<Vec<usize>>::new(); types[i].len()];
            for j in 0..self[i].len() {
                let mut vertices = self.abs.element_vertices(i, j).unwrap();
                vertices.sort_unstable();
                of_this_rank[types_map_back[i][j]].insert(vertices);
            }
            elements[i] = of_this_rank;
        }

        let base_basis = base_flag.clone().vertex_sequence(&self);
        let base_basis_inverse = base_basis.clone().try_inverse().unwrap();

        let mut group = Vec::<Matrix<f64>>::new();

        'a: for flag in flag_iter {
            if flag
                .iter()
                .enumerate()
                .map(|(r, x)| (types_map_back[r][*x] != types_map_back[r][base_flag[r]]) as usize)
                .sum::<usize>() == 0 // this checks if all the elements in the flag have the same types as the ones in the base flag, else it skips it
            {

                // calculate isometry
                let basis = flag.clone().vertex_sequence(&self);
                let isometry = basis * &base_basis_inverse;

                if only_rotations {
                    if isometry.determinant() <= 0. {
                        continue;
                    }
                }

                // check if vertices match up
                let mut vertex_map_row = vec![0; self.vertices.len()];
                for vertex in &vertices {
                    let new_vertex = PointOrd::new(isometry.clone() * vertex.0.matrix());
                    match vertices.get(&new_vertex) {
                        Some(idx) => {
                            vertex_map_row[*vertex.1] = *idx;
                        }
                        None => {
                            continue 'a;
                        }
                    }
                }

                // check if elements match up
                for rank in 2..self.rank() {
                    for idx in 0..types[rank].len() {
                        let mut new_element_vertices: Vec<usize> = self.abs.element_vertices(rank, types[rank][idx].example).unwrap().iter().map(|x| vertex_map_row[*x]).collect();
                        new_element_vertices.sort_unstable();
                        if !elements[rank][idx].contains(&new_element_vertices) {
                            continue 'a;
                        }
                    }
                }

                // add to group if so
                group.push(isometry);
                vertex_map.push(vertex_map_row);
            }
        }

        unsafe {
            Some((Group::new(&self.rank()-1, group.into_iter()), vertex_map))
        }
    }

    /// Computes the rotation subgroup of a polytope, along with a list of vertex mappings.
    pub fn get_rotation_group(&mut self) -> Option<(Group<vec::IntoIter<Matrix<f64>>>, Vec<Vec<usize>>)> {
        return self.get_symmetry_group_with(true);
    }

    /// Fills in the vertex map.
    /// A vertex map is an array of (group element, vertex index) with values being the index of the vertex after applying the transformation.
    pub fn get_vertex_map(&mut self, group: Group<vec::IntoIter<Matrix<f64>>>) -> Vec<Vec<usize>> {
        let mut vertices = Vec::<PointOrd<f64>>::new();
        for v in &self.vertices {
            vertices.push(PointOrd::new(v.clone()));
        }
        let vertices = BTreeMap::from_iter((vertices).into_iter().zip(0..));

        let mut vertex_map: Vec<Vec<usize>> = Vec::new();

        for isometry in group {
            let mut vertex_map_row = Vec::<usize>::new();
            for vertex in &vertices {
                let new_vertex = PointOrd::new(isometry.clone() * vertex.0.matrix());
                match vertices.get(&new_vertex) {
                    Some(idx) => {
                        vertex_map_row.push(*idx);
                    }
                    None => {
                        unreachable!();
                    }
                }
            }
            vertex_map.push(vertex_map_row);
        }
        vertex_map
    }
}

/// A set of vertices.
pub struct Vertices(pub Vec<Point<f64>>);

impl Vertices {
    /// Uses the provided symmetry group on the vertices, also outputs the new vertex map.
    pub fn copy_by_symmetry(&self, group: Group<vec::IntoIter<Matrix<f64>>>) -> (Self, Vec<Vec<usize>>) {
        let mut vertices = BTreeMap::<PointOrd<f64>, usize>::new();
        let mut vertices_vec = Vec::new();
        let mut c = 0;

        for vertex in self.0.clone() {
            if vertices.get(&PointOrd::new(vertex.clone())).is_none() {
                for isometry in group.clone() {
                    let new_vertex = PointOrd::<f64>::new(isometry.clone() * vertex.clone());
                    if vertices.get(&new_vertex).is_none() {
                        vertices.insert(new_vertex.clone(), c);
                        vertices_vec.push(new_vertex);
                        c += 1;
                    }
                }
            }
        }

        let mut vertex_map: Vec<Vec<usize>> = Vec::new();

        for isometry in group {
            let mut vertex_map_row = Vec::<usize>::new();
            for vertex in &vertices_vec {
                let new_vertex = PointOrd::new(isometry.clone() * vertex.matrix());
                match vertices.get(&new_vertex) {
                    Some(idx) => {
                        vertex_map_row.push(*idx);
                    }
                    None => {
                        unreachable!();
                    }
                }
            }
            vertex_map.push(vertex_map_row);
        }
        
        (
            Vertices(Vec::from_iter(vertices_vec.into_iter().map(|point| point.0))),
            vertex_map,
        )
    }
}