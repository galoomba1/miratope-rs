use std::{collections::{BTreeMap, HashMap, HashSet}, iter::FromIterator};

use disjoint_sets::UnionFind;

use petgraph::{graph::UnGraph, stable_graph::{StableGraph, NodeIndex}, algo::has_path_connecting};

use crate::{cox::{Cox, cd::{Cd, Edge, Node}}, Polytope, geometry::{Matrix, MatrixOrd}, abs::{valid, AbstractBuilder}};

use super::Concrete;

fn is_subset(small: Vec<usize>, big: Vec<usize>) -> bool {
    let mut ca = 0;
    let mut cb = 0;
    while ca < small.len() {
        while small[ca] >= big[cb] {
            if cb < big.len() {
                cb += 1;
            } else {
                return false;
            }
        }
        if small[ca] == big[cb] {
            ca += 1;
        } else {
            return false;
        }
    }
    return true;
}

impl Concrete {
    fn wythoff(cd: Cd) -> Self {
        // An auxiliary graph to check which subsets of the CD are degenerate (have components with no ringed nodes).
        let mut stable = StableGraph::from(cd.0.clone());
        for i in stable.clone().edge_indices() {
            if stable.edge_weight(i).unwrap().num == 2 {
                stable.remove_edge(i);
            }
        }

        // Store which nodes are ringed and unringed.
        let mut unringed = Vec::new();
        let mut ringed = Vec::new();
        for i in stable.node_indices() {
            match *stable.node_weight(i).unwrap() {
                Node::Unringed => unringed.push(i.index()),
                _ => ringed.push(i.index()),
            };
        }

        let rank = cd.node_count();

        // Check which subsets of the CD are valid (not degenerate).
        let mut valid_subsets = Vec::new();
        for r in 2..rank {
            let mut valid_subsets_row = Vec::new();
            let mut subset: Vec<usize> = (0..r).collect();
            let mut c = r-1;
            while subset[0] <= rank-r {
                let mut valid = true;
                for node in &subset {
                    let mut valid2 = false;
                    for node2 in &ringed {
                        if has_path_connecting(&stable, NodeIndex::new(*node), NodeIndex::new(*node2), None) {
                            valid2 = true;
                            break;
                        }
                    }
                    if !valid2 {
                        valid = false;
                        break;
                    }
                }
                if valid {
                    valid_subsets_row.push(subset.clone());
                }

                while subset[c] >= rank-r+c {
                    c -= 1;
                }
                subset[c] += 1;
                for i in c..r {
                    subset[c] = subset[c-1] + 1;
                }
            }
            valid_subsets.push(valid_subsets_row);
        }

        let group = cd.cox().group().unwrap();
        let order = group.clone().count();
        let generator_point = cd.generator().unwrap();
        let reflections = cd.cox().generators().unwrap(); // generators of the group

        // Map matrices to their indices to check adjacency later.
        let group_map_back: BTreeMap<MatrixOrd<f64>, usize> = BTreeMap::from_iter(
            group.clone().enumerate().map(|(i, e)| {(MatrixOrd::from(e), i)})
        );

        // Check which matrices differ only by a right multiplication with a generator.
        let mut adjacent_elements = Vec::new();
        for element in group {
            let mut row = Vec::new();
            for generator in &reflections {
                row.push(*group_map_back.get(&MatrixOrd::from(&element * generator)).unwrap());
            }
            adjacent_elements.push(row);
        }

        // Disjoint set data structure to identify the matrices that map to the same vertex.
        let mut vertices_djs = UnionFind::new(order);

        for (a, element) in adjacent_elements.iter().enumerate() {
            for r in &unringed {
                vertices_djs.union(a, element[*r]);
            }
        }

        // Index vertices from 0 to n-1, and store which matrices map to which vertex.
        let mut reindex_reps = HashMap::new();
        let mut vertex_idxs = Vec::new();

        for i in 0..order {
            let rep = vertices_djs.find(i);
            match reindex_reps.get(&rep) {
                Some(new) => vertex_idxs.push(*new),
                None => {
                    vertex_idxs.push(reindex_reps.len());
                    reindex_reps.insert(rep, reindex_reps.len());
                },
            };
        }

        // Disjoint set data structure to find the higher elements.
        let mut djs = Vec::new();
        for row in &valid_subsets {
            let mut djs_row = Vec::new();
            for subset in row {
                let mut djs_row_row = UnionFind::new(order);
                for (idx, element) in adjacent_elements.iter().enumerate() {
                    for r in subset {
                        djs_row_row.union(idx, element[*r]);
                    }
                }
                djs_row.push(djs_row_row);
            }
            djs.push(djs_row);
        }

        let mut builder = AbstractBuilder::new();
        builder.push_min();
        builder.push_vertices(reindex_reps.len());

        let mut element_sets = Vec::new();
        let mut duplicate_remover = HashSet::new();
        
        let mut edge_idxs = Vec::new();
        let mut cur: usize = 0;
        let mut edge_sets = Vec::new();

        for subset in valid_subsets[0] {
            let mut edge_sets_row: Vec<Vec<usize>> = Vec::new();
            let mut edge_idxs_row = Vec::new();
            let mut reindex_reps: HashMap<usize, usize> = HashMap::new();
            for i in 0..order {
                let rep = djs[0][subset[0]].find(i);
                match reindex_reps.get(&rep) {
                    Some(idx) => {
                        edge_idxs_row.push(*idx);
                        edge_sets_row[*idx].push(i);
                    },
                    None => {
                        reindex_reps.insert(rep, cur);
                        edge_idxs_row.push(cur);
                        edge_sets_row.push(vec![i]);
                        cur += 1;
                    },
                };
            }
            edge_idxs.push(edge_idxs_row);
            edge_sets.push(edge_sets_row);
        }

        Concrete::point()
    }
}