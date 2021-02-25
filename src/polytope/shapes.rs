use gcd::Gcd;
use nalgebra::Dynamic;
use std::{collections::HashMap, f64::consts::PI as PI64};

use super::{Element, ElementList, Matrix, Point, Polytope};

/// Generates matrices for rotations by the first `num` multiples of `angle`
/// through the xy plane.
fn rotations(angle: f64, num: usize, dim: usize) -> Vec<Matrix> {
    let mut rotations = Vec::with_capacity(num);
    let dim = Dynamic::new(dim);
    let mut matrix = nalgebra::Matrix::identity_generic(dim, dim);
    let mut rotation = nalgebra::Matrix::identity_generic(dim, dim);

    // The first rotation matrix.
    let (s, c) = angle.sin_cos();
    rotation[(0, 0)] = c;
    rotation[(1, 0)] = s;
    rotation[(0, 1)] = -s;
    rotation[(1, 1)] = c;

    // Generates the other rotation matrices from r.
    for _ in 0..num {
        rotations.push(matrix.to_homogeneous());
        matrix *= &rotation;
    }

    rotations
}

/// Applies a list of transformations to a polytope and creates a compound from
/// all of the copies of the polytope this generates.
pub fn compound(p: Polytope, trans: Vec<Matrix>) -> Polytope {
    let comps = trans.len();
    let el_nums = p.el_nums();
    let mut elements = Vec::with_capacity(p.elements.len());

    // The vertices, turned into homogeneous points.
    let vertices = p
        .vertices
        .into_iter()
        .map(|v| v.push(1.0))
        .collect::<Vec<_>>();

    // Applies the transformations to the vertex list.
    let vertices = trans
        .into_iter()
        .flat_map(|m| vertices.iter().map(move |v| m.clone() * v))
        .map(|v| {
            // Removes the extra homogeneous coordinate.
            let row = v.nrows() - 1;
            v.remove_row(row)
        })
        .collect();

    // Copies and offsets the element list once for each new component.
    for (d, els) in p.elements.iter().enumerate() {
        let sub_num = el_nums[d];
        let el_num = el_nums[d + 1];
        let mut new_els = Vec::with_capacity(el_num * comps);

        for comp in 0..comps {
            let offset = comp * sub_num;

            for el in els.iter() {
                new_els.push(el.iter().map(|i| i + offset).collect())
            }
        }

        elements.push(new_els);
    }

    Polytope::new(vertices, elements)
}

/// Generates the unique 0D polytope.
pub fn point() -> Polytope {
    let vertices = vec![];
    let elements = vec![];

    Polytope::new(vertices, elements)
}

/// Generates a dyad, the unique non-compound 1D polytope.
pub fn dyad() -> Polytope {
    let vertices = vec![vec![-0.5].into(), vec![0.5].into()];
    let elements = vec![vec![vec![0, 1]]];

    Polytope::new(vertices, elements)
}

/// Builds a polygon from the vertices in order.
pub fn polygon(vertices: Vec<Point>) -> Polytope {
    let n = vertices.len();
    let mut edges = Vec::with_capacity(n);
    let mut component = Vec::with_capacity(n);

    for k in 0..n {
        edges.push(vec![k, (k + 1) % n]);
        component.push(k);
    }

    Polytope::new(vertices, vec![edges, vec![component]])
}

/// Generates a semiregular polygon, with order n rotational symmetry and winding number d.
/// Bowties correspond to shapes where `d == 0`.
pub fn semiregular_polygon(mut n: usize, d: usize, mut len_a: f64, mut len_b: f64) -> Polytope {
    let comp_num;
    let comp_angle;

    let regular = len_a == 0.0 || len_b == 0.0;
    let vertex_num = if regular { n } else { 2 * n };
    let mut vertices = Vec::with_capacity(vertex_num);

    // Bowties are a special case that must be considered separately.
    if d == 0 {
        if len_a > len_b {
            std::mem::swap(&mut len_a, &mut len_b);
        }

        let (a, b) = (len_a / 2.0, (len_b * len_b - len_a * len_a).sqrt() / 2.0);

        vertices = vec![
            vec![a, b].into(),
            vec![-a, -b].into(),
            vec![a, -b].into(),
            vec![-a, b].into(),
        ];

        comp_num = n / 2;
        comp_angle = PI64 / comp_num as f64;
    } else {
        // Builds the triangle from three adjacent vertices, and finds its side lengths and angles.
        let gamma = PI64 * (1.0 - (2.0 * d as f64) / (n as f64));
        let len_c = (len_a * len_a + len_b * len_b - 2.0 * len_a * len_b * gamma.cos()).sqrt();
        let mut alpha =
            ((len_b * len_b + len_c * len_c - len_a * len_a) / (2.0 * len_b * len_c)).acos();
        let mut beta =
            ((len_c * len_c + len_a * len_a - len_b * len_b) / (2.0 * len_c * len_a)).acos();
        let radius = gamma / (2.0 * gamma.sin());

        // Fixes the angles in case anything goes wrong in the calculation.
        let theta = 2.0 * PI64 * (d as f64) / (n as f64);
        if alpha.is_nan() {
            alpha = theta;
        }
        if beta.is_nan() {
            beta = theta;
        }

        comp_num = n.gcd(d);
        n /= comp_num;

        // Adds vertices.
        let mut angle = 0f64;
        for _ in 0..n {
            if len_a != 0.0 {
                vertices.push(vec![angle.cos() * radius, angle.sin() * radius].into());
                angle += alpha;
            }

            if len_b != 0.0 {
                vertices.push(vec![angle.cos() * radius, angle.sin() * radius].into());
                angle += beta;
            }
        }

        comp_angle = 2.0 * PI64 / (n as f64 * comp_num as f64);
    }

    compound(polygon(vertices), rotations(comp_angle, comp_num, 2))
}

/// Generates a regular polygon with Schläfli symbol {n / d}.
pub fn regular_polygon(n: usize, d: usize) -> Polytope {
    if d == 0 {
        panic!("Invalid parameter d = 0.")
    }

    semiregular_polygon(n, d, 1.0, 0.0)
}

/// Generates a regular tetrahedron with unit edge length.
pub fn tet() -> Polytope {
    let x = 2.0_f64.sqrt() / 4.0;

    let vertices = vec![
        vec![x, x, x].into(),
        vec![-x, -x, x].into(),
        vec![-x, x, -x].into(),
        vec![x, -x, -x].into(),
    ];
    let edges = vec![
        vec![0, 1],
        vec![0, 2],
        vec![0, 3],
        vec![1, 2],
        vec![1, 3],
        vec![2, 3],
    ];
    let faces = vec![vec![0, 1, 3], vec![0, 2, 4], vec![1, 2, 5], vec![3, 4, 5]];
    let components = vec![vec![0, 1, 2, 3]];

    Polytope::new(vertices, vec![edges, faces, components])
}

/// Generates a cube with unit edge length.
pub fn cube() -> Polytope {
    let x = 0.5;

    let vertices = vec![
        vec![x, x, x].into(),
        vec![x, x, -x].into(),
        vec![x, -x, -x].into(),
        vec![x, -x, x].into(),
        vec![-x, x, x].into(),
        vec![-x, x, -x].into(),
        vec![-x, -x, -x].into(),
        vec![-x, -x, x].into(),
    ];
    let edges = vec![
        vec![0, 1],
        vec![1, 2],
        vec![2, 3],
        vec![3, 0],
        vec![4, 5],
        vec![5, 6],
        vec![6, 7],
        vec![7, 4],
        vec![0, 4],
        vec![1, 5],
        vec![2, 6],
        vec![3, 7],
    ];
    let faces = vec![
        vec![0, 1, 2, 3],
        vec![4, 5, 6, 7],
        vec![0, 4, 8, 9],
        vec![1, 5, 9, 10],
        vec![2, 6, 10, 11],
        vec![3, 7, 11, 8],
    ];
    let components = vec![vec![0, 1, 2, 3, 4, 5]];

    Polytope::new(vertices, vec![edges, faces, components])
}

/// Generates an octahedron with unit edge length.
pub fn oct() -> Polytope {
    let x = 1.0 / 2.0_f64.sqrt();

    let vertices = vec![
        vec![x, 0.0, 0.0].into(),
        vec![-x, 0.0, 0.0].into(),
        vec![0.0, x, 0.0].into(),
        vec![0.0, 0.0, x].into(),
        vec![0.0, -x, 0.0].into(),
        vec![0.0, 0.0, -x].into(),
    ];
    let edges = vec![
        vec![0, 2],
        vec![0, 3],
        vec![0, 4],
        vec![0, 5],
        vec![1, 2],
        vec![1, 3],
        vec![1, 4],
        vec![1, 5],
        vec![2, 3],
        vec![3, 4],
        vec![4, 5],
        vec![5, 2],
    ];
    let faces = vec![
        vec![0, 1, 8],
        vec![4, 5, 8],
        vec![1, 2, 9],
        vec![5, 6, 9],
        vec![2, 3, 10],
        vec![6, 7, 10],
        vec![3, 0, 11],
        vec![7, 4, 11],
    ];
    let components = vec![vec![0, 1, 2, 3, 4, 5, 6, 7]];

    Polytope::new(vertices, vec![edges, faces, components])
}

/// Creates an [antiprism](https://polytope.miraheze.org/wiki/Antiprism)
/// with unit base edge length and a given height.
pub fn antiprism_with_height(mut n: usize, d: usize, height: f64) -> Polytope {
    let component_num = n.gcd(d);
    let theta = PI64 / (n as f64) * (d as f64);
    n /= component_num;
    let r = theta.sin() * 2.0;
    let mut height = height / 2.0;

    let mut vertices = Vec::with_capacity(2 * n);
    let mut edges = Vec::with_capacity(4 * n);
    let mut faces = Vec::with_capacity(2 * n + 2);
    let mut components = vec![Vec::with_capacity(2 * n + 2)];

    for k in 0..(2 * n) {
        // Generates vertices.
        let angle = (k as f64) * theta;
        vertices.push(vec![angle.cos() / r, angle.sin() / r, height].into());
        height *= -1.0;

        // Generates edges.
        edges.push(vec![k, (k + 1) % (2 * n)]);
        edges.push(vec![k, (k + 2) % (2 * n)]);

        // Generates faces.
        faces.push(vec![2 * k, 2 * k + 1, (2 * k + 2) % (4 * n)]);

        // Generates component.
        components[0].push(k);
    }

    let (mut base1, mut base2) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for k in 0..n {
        base1.push(4 * k + 1);
        base2.push(4 * k + 3);
    }
    faces.push(base1);
    faces.push(base2);

    components[0].push(2 * n);
    components[0].push(2 * n + 1);

    // Compounds of antiprisms with antiprismatic symmetry must be handled
    // differently than compounds of antiprisms with prismatic symmetry.
    let angle = if d / component_num % 2 == 0 {
        theta
    } else {
        theta * 2.0
    };
    compound(
        Polytope::new(vertices, vec![edges, faces, components]),
        rotations(angle / component_num as f64, component_num, 3),
    )
}

/// Creates a uniform [antiprism](https://polytope.miraheze.org/wiki/Antiprism)
/// with unit edge length.
pub fn antiprism(n: usize, d: usize) -> Polytope {
    let angle = PI64 / (n as f64) * (d as f64);
    let x = 2.0 * angle.cos();
    let height = ((1.0 + x) / (2.0 + x)).sqrt();

    if height.is_nan() {
        panic!("Uniform antiprism could not be built from these parameters.");
    }

    antiprism_with_height(n, d, height)
}

/// Projects a [`Point`] onto the hyperplane defined by a vector of [`Points`][`Point`].
pub fn project(p: &Point, h: Vec<Point>) -> Point {
    const EPS: f64 = 1e-9;

    let mut h = h.iter();
    let o = h.next().unwrap();
    let mut basis: Vec<Point> = Vec::new();

    for q in h {
        let mut q = q - o;

        for b in &basis {
            q -= b * (q.dot(&b)) / b.norm_squared();
        }

        if q.norm() > EPS {
            basis.push(q);
        }
    }

    let mut p = p - o;

    for b in &basis {
        p -= b * (p.dot(&b)) / b.norm_squared();
    }

    p
}

/// Builds the vertices of a dual polytope from its facets.
fn dual_vertices(vertices: &[Point], elements: &[ElementList], o: &Point) -> Vec<Point> {
    const EPS: f64 = 1e-9;

    let rank = elements.len();
    let facets = &elements[rank - 2];
    let ridges = &elements[rank - 3];

    // Gets the unique sub-elements from a list of elements.
    let unique_subs = |els: &Vec<&Vec<usize>>| -> Element {
        let mut uniq = HashMap::new();

        for &el in els {
            for &sub in el {
                uniq.insert(sub, ());
            }
        }

        uniq.keys().cloned().collect()
    };

    // We find the indices of the vertices on the facet.
    facets
        .iter()
        .map(|f| {
            // We repeatedly retrieve the next subelements of the facets until we get to the vertices.
            let mut els = f.iter().map(|&el| &ridges[el]).collect();
            for d in (0..(rank - 3)).rev() {
                let uniq = unique_subs(&els);
                els = uniq.iter().map(|&el| &elements[d][el]).collect();
            }

            // We project the dual center onto the hyperplane defined by the vertices.
            let el = unique_subs(&els);
            let h = el.iter().map(|&v| vertices[v].clone()).collect();
            let v = project(o, h);
            let s = v.norm_squared();

            // We avoid division by 0.
            if s < EPS {
                panic!("Facet passes through the dual center.")
            }

            v / s
        })
        .collect()
}

/// Builds the dual polytope of `p`. Uses `o` as the center for reciprocation.
pub fn dual_with_center(p: &Polytope, o: &Point) -> Polytope {
    let el_nums = p.el_nums();

    let vertices = &p.vertices;
    let elements = &p.elements;

    let du_vertices = dual_vertices(vertices, elements, o);
    let mut du_elements = Vec::with_capacity(elements.len());

    // Builds the dual incidence graph.
    let mut elements = elements.iter().enumerate().rev();
    elements.next();

    for (d, els) in elements {
        let c = el_nums[d];
        let mut du_els = Vec::with_capacity(c);

        for _ in 0..c {
            du_els.push(vec![]);
        }

        for (i, el) in els.iter().enumerate() {
            for &sub in el {
                let du_el = &mut du_els[sub];
                du_el.push(i);
            }
        }

        du_elements.push(du_els);
    }

    Polytope::new_wo_comps(du_vertices, du_elements)
}

/// Builds the dual polytope of `p`. Uses the origin as the center for reciprocation.
pub fn dual(p: &Polytope) -> Polytope {
    let dim = p.dimension();
    let mut o = Vec::with_capacity(dim);
    o.resize(dim, 0.0);

    dual_with_center(p, &o.into())
}

fn duoprism_vertices(p: &[Point], q: &[Point]) -> Vec<Point> {
    let dimension = p[0].len() + q[0].len();
    let mut vertices = Vec::with_capacity(p.len() * q.len());

    for pv in p {
        for qv in q {
            let (pv, qv) = (pv.into_iter(), qv.into_iter());
            let mut v = Vec::with_capacity(dimension);

            for &c in pv {
                v.push(c);
            }
            for &c in qv {
                v.push(c);
            }

            vertices.push(v.into());
        }
    }

    vertices
}

/// Creates a [duoprism](https://polytope.miraheze.org/wiki/Duoprism)
/// from two given polytopes.
///
/// Duoprisms are usually defined in terms of Cartesian products, but this
/// definition only makes sense in the context of convex polytopes. For general
/// polytopes, a duoprism may be inductively built as a polytope whose facets
/// are the "prism products" of the elements of the first polytope times those
/// of the second, where the prism product of two points is simply the point
/// resulting from concatenating their coordinates.
pub fn duoprism(p: &Polytope, q: &Polytope) -> Polytope {
    let (p_rank, q_rank) = (p.rank(), q.rank());
    let (p_vertices, q_vertices) = (&p.vertices, &q.vertices);
    let (p_elements, q_elements) = (&p.elements, &q.elements);
    let (p_el_nums, q_el_nums) = (p.el_nums(), q.el_nums());

    let rank = p_rank + q_rank;

    if p_rank == 0 {
        return q.clone();
    }
    if q_rank == 0 {
        return p.clone();
    }

    let vertices = duoprism_vertices(&p_vertices, &q_vertices);
    let mut elements = Vec::with_capacity(rank);
    for _ in 0..rank {
        elements.push(Vec::new());
    }

    // The elements of a given rank are added in order vertex × facet, edge ×
    // ridge, ...
    //
    // el_nums[m][n] will memoize the number of elements of rank m generated
    // by these products up to those of type n-element × (m - n)-element.
    let mut el_nums = Vec::with_capacity(rank);
    for m in 0..(p_rank + 1) {
        el_nums.push(Vec::new());

        for n in 0..(q_rank + 1) {
            if m == 0 || n == q_rank {
                el_nums[m].push(0);
            } else {
                let idx = el_nums[m - 1][n + 1] + p_el_nums[m - 1] * q_el_nums[n + 1];
                el_nums[m].push(idx);
            }
        }
    }

    // Gets the index of the prism product of the i-th m-element times the j-th
    // n-element.
    let get_idx = |m: usize, i: usize, n: usize, j: usize| -> usize {
        let offset = i * q_el_nums[n] + j;

        el_nums[m][n] + offset
    };

    // For each of the element lists of p (including vertices):
    for m in 0..(p_rank + 1) {
        // For each of the element lists of q (including vertices):
        for n in 0..(q_rank + 1) {
            // We'll multiply the m-elements times the n-elements inside of this loop.

            // We already took care of vertices.
            if m == 0 && n == 0 {
                continue;
            }

            // For each m-element:
            for i in 0..p_el_nums[m] {
                // For each n-element:
                for j in 0..q_el_nums[n] {
                    let mut els = Vec::new();

                    // The prism product of the i-th m-element A and the j-th n-element B
                    // has the products of A with the facets of B and B with the facets of
                    // A as facets.

                    // Points don't have facets.
                    if m != 0 {
                        let p_els = &p_elements[m - 1];
                        let p_el = &p_els[i];

                        for &p_sub in p_el {
                            els.push(get_idx(m - 1, p_sub, n, j));
                        }
                    }

                    // Points don't have facets.
                    if n != 0 {
                        let q_els = &q_elements[n - 1];
                        let q_el = &q_els[j];

                        for &q_sub in q_el {
                            els.push(get_idx(m, i, n - 1, q_sub));
                        }
                    }

                    elements[m + n - 1].push(els);
                }
            }
        }
    }

    Polytope::new(vertices, elements)
}

pub fn prism_with_height(p: &Polytope, h: f64) -> Polytope {
    let mut dyad = dyad();
    dyad.scale(h);

    duoprism(p, &dyad)
}

pub fn prism(p: &Polytope) -> Polytope {
    prism_with_height(p, 1.0)
}

pub fn multiprism(polytopes: &[&Polytope]) -> Polytope {
    let mut r = point();

    for &p in polytopes {
        r = duoprism(&p, &r);
    }

    r
}

fn pyramid_vertices(p: &[Point], q: &[Point], h: f64) -> Vec<Point> {
    let (p_dimension, q_dimension) = (
        match p.get(0) {
            Some(v) => v.len(),
            None => 0,
        },
        match q.get(0) {
            Some(v) => v.len(),
            None => 0,
        },
    );

    let tegum = h == 0.0;
    let dimension = p_dimension + q_dimension + tegum as usize;

    let mut vertices = Vec::with_capacity(p.len() + q.len());

    for vq in q {
        let mut v = Vec::with_capacity(dimension);
        let pad = p_dimension;

        v.resize(pad, 0.0);

        for &c in vq.iter() {
            v.push(c);
        }

        if !tegum {
            v.push(h / 2.0);
        }

        vertices.push(v.into());
    }
    for vp in p {
        let mut v = Vec::with_capacity(dimension);

        for &c in vp.iter() {
            v.push(c);
        }

        v.resize(p_dimension + q_dimension, 0.0);

        if !tegum {
            v.push(-h / 2.0);
        }

        vertices.push(v.into());
    }

    vertices
}

/// Builds a duopyramid with a given height, or a duotegum if the height is 0.
pub fn duopyramid_with_height(p: &Polytope, q: &Polytope, height: f64) -> Polytope {
    let (p_rank, q_rank) = (p.rank(), q.rank());
    let (p_vertices, q_vertices) = (&p.vertices, &q.vertices);
    let (p_elements, q_elements) = (&p.elements, &q.elements);
    let (p_el_nums, q_el_nums) = (p.el_nums(), q.el_nums());

    let tegum = height == 0.0;
    let rank = p_rank + q_rank + !tegum as usize;

    let (m_max, n_max) = (p_rank + !tegum as usize + 1, q_rank + !tegum as usize + 1);

    if tegum {
        if p_rank == 0 {
            return q.clone();
        }

        if q_rank == 0 {
            return p.clone();
        }
    }

    let vertices = pyramid_vertices(&p_vertices, &q_vertices, height);
    let mut elements = Vec::with_capacity(rank);
    for _ in 0..rank {
        elements.push(Vec::new());
    }

    // The elements of a given rank are added in order nullitope × facet, vertex
    // × ridge, ...
    //
    // el_nums[m][n] will memoize the number of elements of rank m - 1
    // generated by these products up to those of type (n - 1)-element ×
    // (m - n)-element.
    let mut el_nums = Vec::with_capacity(rank);
    for m in 0..m_max {
        el_nums.push(Vec::new());

        for (n, &q_el_num) in q_el_nums.iter().enumerate().take(n_max) {
            if m == 0 {
                el_nums[m].push(0);
            } else {
                let p_el_num = if m == 1 { 1 } else { p_el_nums[m - 2] };

                let idx = el_nums[m - 1][n + 1] + p_el_num * q_el_num;
                el_nums[m].push(idx);
            }
        }

        el_nums[m].push(0);
    }

    // Gets the index of the prism product of the i-th m-element times the j-th
    // n-element.
    let get_idx = |m: usize, i: usize, n: usize, j: usize| -> usize {
        let q_el_nums_n = if n == 0 { 1 } else { q_el_nums[n - 1] };
        let offset = i * q_el_nums_n + j;

        el_nums[m][n] + offset
    };

    // For each of the element lists of p (including vertices & the nullitope):
    for m in 0..m_max {
        let p_el_nums_m = if m == 0 { 1 } else { p_el_nums[m - 1] };

        // For each of the element lists of q (including vertices & the nullitope):
        for n in 0..n_max {
            let q_el_nums_n = if n == 0 { 1 } else { q_el_nums[n - 1] };

            // We'll multiply the (m - 1)-elements with the (n - 1)-elements inside of this loop.

            // We already took care of vertices.
            if m + n < 2 {
                continue;
            }

            // For each m-element:
            for i in 0..p_el_nums_m {
                // For each n-element:
                for j in 0..q_el_nums_n {
                    let mut els = Vec::new();

                    // The prism product of the i-th m-element A and the j-th n-element B
                    // has the products of A with the facets of B and B with the facets of
                    // A as facets.

                    // Nullitopes don't have facets.
                    if m != 0 {
                        if m > 1 {
                            let p_els = &p_elements[m - 2];
                            let p_el = &p_els[i];

                            for &p_sub in p_el {
                                els.push(get_idx(m - 1, p_sub, n, j));
                            }
                        }
                        // Dealing with a vertex
                        else {
                            els.push(get_idx(m - 1, 0, n, j));
                        }
                    }

                    // Nullitopes don't have facets.
                    if n != 0 {
                        if n > 1 {
                            let q_els = &q_elements[n - 2];
                            let q_el = &q_els[j];

                            for &q_sub in q_el {
                                els.push(get_idx(m, i, n - 1, q_sub));
                            }
                        }
                        // Dealing with a vertex
                        else {
                            els.push(get_idx(m, i, n - 1, 0));
                        }
                    }

                    elements[m + n - 2].push(els);
                }
            }
        }
    }

    if tegum {
        // We take special care of the components.
        // These are simply the pyramid products of the two polytopes' facets.
        // For each m-element:
        let (m, n) = (p_rank + 1, q_rank + 1);
        let (p_el_nums_m, q_el_nums_n) = (p_el_nums[m - 1], q_el_nums[n - 1]);

        // For each component of p:
        for i in 0..p_el_nums_m {
            // For each component of q:
            for j in 0..q_el_nums_n {
                let mut els = Vec::new();

                // The prism product of the i-th m-element A and the j-th n-element B
                // has the products of A with the facets of B and B with the facets of
                // A as facets.

                let (p_els, q_els) = (&p_elements[m - 2], &q_elements[n - 2]);
                let (p_el, q_el) = (&p_els[i], &q_els[j]);

                for &p_sub in p_el {
                    for &q_sub in q_el {
                        els.push(get_idx(m - 1, p_sub, n - 1, q_sub));
                    }
                }

                elements[m + n - 3].push(els);
            }
        }
    }

    Polytope::new(vertices, elements)
}

pub fn duotegum(p: &Polytope, q: &Polytope) -> Polytope {
    duopyramid_with_height(p, q, 0.0)
}

pub fn tegum_with_height(p: &Polytope, h: f64) -> Polytope {
    let mut dyad = dyad();
    dyad.scale(h);

    duotegum(p, &dyad)
}

pub fn tegum(p: &Polytope) -> Polytope {
    tegum_with_height(p, 1.0)
}

pub fn multitegum(polytopes: &[&Polytope]) -> Polytope {
    let mut r = point();

    for p in polytopes {
        r = duotegum(&p, &r);
    }

    r
}

pub fn duopyramid(p: &Polytope, q: &Polytope) -> Polytope {
    duopyramid_with_height(p, q, 1.0)
}

pub fn pyramid_with_height(p: &Polytope, h: f64) -> Polytope {
    let point = point();

    duopyramid_with_height(p, &point, h)
}

pub fn pyramid(p: &Polytope) -> Polytope {
    pyramid_with_height(p, 1.0)
}

pub fn multipyramid_with_height(polytopes: &[&Polytope], h: f64) -> Polytope {
    let mut polytopes = polytopes.iter();
    let mut r = (*polytopes.next().unwrap()).clone();

    for p in polytopes {
        r = duopyramid_with_height(&p, &r, h);
    }

    r
}

pub fn multipyramid(polytopes: &[&Polytope]) -> Polytope {
    multipyramid_with_height(polytopes, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Checks the element nums of a few polygons.
    fn polygon_nums() {
        assert_eq!(regular_polygon(5, 1).el_nums(), vec![5, 5, 1]);
        assert_eq!(regular_polygon(7, 2).el_nums(), vec![7, 7, 1]);
        assert_eq!(regular_polygon(6, 2).el_nums(), vec![6, 6, 2])
    }

    #[test]
    /// Checks the element num of a tetrahedron.
    fn tet_nums() {
        assert_eq!(tet().el_nums(), vec![4, 6, 4, 1])
    }

    #[test]
    /// Checks the element num of a cube.
    fn cube_nums() {
        assert_eq!(cube().el_nums(), vec![8, 12, 6, 1])
    }

    #[test]
    /// Checks the element num of an octahedron.
    fn oct_nums() {
        assert_eq!(oct().el_nums(), vec![6, 12, 8, 1])
    }

    #[test]
    /// Checks the element nums of a few antiprisms.
    fn antiprism_nums() {
        assert_eq!(antiprism(5, 1).el_nums(), vec![10, 20, 12, 1]);
        assert_eq!(antiprism(7, 2).el_nums(), vec![14, 28, 16, 1]);
        assert_eq!(antiprism(6, 2).el_nums(), vec![12, 24, 16, 2])
    }

    #[test]
    /// Checks the element num of a cube dual (octahedron).
    fn cube_dual_nums() {
        let cube_dual = dual(&cube());

        assert_eq!(cube_dual.el_nums(), vec![6, 12, 8, 1])
    }

    #[test]
    /// Checks the element num of a triangular-pentagonal duoprism.
    fn trapedip_nums() {
        let trig = regular_polygon(3, 1);
        let peg = regular_polygon(5, 1);
        let trapedip = duoprism(&trig, &peg);

        assert_eq!(trapedip.el_nums(), vec![15, 30, 23, 8, 1])
    }

    #[test]
    /// Checks the element num of a triangular trioprism.
    fn trittip_nums() {
        let trig = regular_polygon(3, 1);
        let trittip = multiprism(&vec![&trig; 3]);

        assert_eq!(trittip.el_nums(), vec![27, 81, 108, 81, 36, 9, 1])
    }

    #[test]
    /// Checks the element num of a triangular-pentagonal duotegum.
    fn trapedit_nums() {
        let trig = regular_polygon(3, 1);
        let peg = regular_polygon(5, 1);
        let trapedit = duotegum(&trig, &peg);

        assert_eq!(trapedit.el_nums(), vec![8, 23, 30, 15, 1])
    }

    #[test]
    /// Checks the element num of a triangular triotegum.
    fn trittit_nums() {
        let trig = regular_polygon(3, 1);
        let trittit = multitegum(&vec![&trig; 3]);

        assert_eq!(trittit.el_nums(), vec![9, 36, 81, 108, 81, 27, 1])
    }

    #[test]
    /// Checks the element num of a triangular-pentagonal duopyramid.
    fn trapdupy_nums() {
        let trig = regular_polygon(3, 1);
        let peg = regular_polygon(5, 1);
        let trapdupy = duopyramid(&trig, &peg);

        assert_eq!(trapdupy.el_nums(), vec![8, 23, 32, 23, 8, 1])
    }

    #[test]
    /// Checks the element num of a triangular triopyramid.
    fn tritippy_nums() {
        let trig = regular_polygon(3, 1);
        let tritippy = multipyramid(&vec![&trig; 3]);

        assert_eq!(tritippy.el_nums(), vec![9, 36, 84, 126, 126, 84, 36, 9, 1])
    }
}
