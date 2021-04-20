use crate::{
    polytope::{
        geometry::{Hyperplane, Hypersphere, Matrix, Point, Segment, Subspace, Vector},
        rank::RankVec,
        Abstract, Element, ElementList, Polytope, Subelements, Subsupelements,
    },
    EPS,
};
use approx::{abs_diff_eq, abs_diff_ne};
use gcd::Gcd;
use std::{
    collections::HashMap,
    f64::consts::{SQRT_2, TAU},
};

#[derive(Debug, Clone)]
/// Represents a [concrete polytope](https://polytope.miraheze.org/wiki/Polytope),
/// which is an [`Abstract`] together with its corresponding vertices.
pub struct Concrete {
    /// The list of vertices as points in Euclidean space.
    pub vertices: Vec<Point>,

    /// The underlying abstract polytope.
    pub abs: Abstract,
}

impl Concrete {
    /// Initializes a new concrete polytope from a set of vertices and an
    /// underlying abstract polytope. Does some debug assertions on the input.
    pub fn new(vertices: Vec<Point>, abs: Abstract) -> Self {
        // There must be as many abstract vertices as concrete ones.
        debug_assert_eq!(vertices.len(), abs.el_count(0));

        // All vertices must have the same dimension.
        if let Some(vertex0) = vertices.get(0) {
            for vertex1 in &vertices {
                debug_assert_eq!(vertex0.len(), vertex1.len());
            }
        }

        Self { vertices, abs }
    }

    /// Returns the rank of the polytope.
    pub fn rank(&self) -> isize {
        self.abs.rank()
    }

    /// Returns the number of dimensions of the space the polytope lives in,
    /// or `None` in the case of the nullitope.
    pub fn dim(&self) -> Option<usize> {
        Some(self.vertices.get(0)?.len())
    }

    /// Builds the Grünbaumian star polygon `{n / d}`, rotated by an angle.
    fn grunbaum_star_polygon_with_rot(n: usize, d: usize, rot: f64) -> Self {
        assert!(n >= 2);
        assert!(d >= 1);

        // Scaling factor for unit edge length.
        let angle = TAU * d as f64 / n as f64;
        let radius = (2.0 - 2.0 * angle.cos()).sqrt();

        Self::new(
            (0..n)
                .into_iter()
                .map(|k| {
                    let (sin, cos) = (k as f64 * angle + rot).sin_cos();
                    vec![sin / radius, cos / radius].into()
                })
                .collect(),
            Abstract::polygon(n),
        )
    }

    /// Builds the Grünbaumian star polygon `{n / d}`. If `n` and `d` have a
    /// common factor, the result is a multiply-wound polygon.
    pub fn grunbaum_star_polygon(n: usize, d: usize) -> Self {
        Self::grunbaum_star_polygon_with_rot(n, d, 0.0)
    }

    /// Builds the star polygon `{n / d}`. If `n` and `d` have a common factor,
    /// the result is a compound.
    pub fn star_polygon(n: usize, d: usize) -> Self {
        let gcd = n.gcd(d);
        let angle = TAU / n as f64;

        Self::compound_iter(
            (0..gcd)
                .into_iter()
                .map(|k| Self::grunbaum_star_polygon_with_rot(n / gcd, d / gcd, k as f64 * angle)),
        )
        .unwrap()
    }

    /// Scales a polytope by a given factor.
    pub fn scale(&mut self, k: f64) {
        for v in &mut self.vertices {
            *v *= k;
        }
    }

    /// Shifts all vertices by a given vector.
    pub fn shift(&mut self, o: Vector) {
        for v in &mut self.vertices {
            *v -= &o;
        }
    }

    /// Recenters a polytope so that the gravicenter is at the origin.
    pub fn recenter(&mut self) {
        if let Some(gravicenter) = self.gravicenter() {
            self.shift(gravicenter);
        }
    }

    /// Applies a matrix to all vertices of a polytope.
    pub fn apply(mut self, m: &Matrix) -> Self {
        for v in &mut self.vertices {
            *v = m * v.clone();
        }

        self
    }

    /// Calculates the circumsphere of a polytope. Returns it if the polytope
    /// has one, and returns `None` otherwise.
    pub fn circumsphere(&self) -> Option<Hypersphere> {
        let mut vertices = self.vertices.iter();

        let v0 = vertices.next().expect("Polytope has no vertices!").clone();
        let mut o: Point = v0.clone();
        let mut h = Subspace::new(v0.clone());

        for v in vertices {
            // If the new vertex does not lie on the hyperplane of the others:
            if let Some(b) = h.add(&v) {
                // Calculates the new circumcenter.
                let k = ((&o - v).norm_squared() - (&o - &v0).norm_squared())
                    / (2.0 * (v - &v0).dot(&b));

                o += k * b;
            }
            // If the new vertex lies on the others' hyperplane, but is not at
            // the correct distance from the first vertex:
            else if abs_diff_ne!((&o - &v0).norm(), (&o - v).norm(), epsilon = EPS) {
                return None;
            }
        }

        Some(Hypersphere {
            radius: (&o - v0).norm(),
            center: o,
        })
    }

    /// Gets the gravicenter of a polytope, or `None` in the case of the
    /// nullitope.
    pub fn gravicenter(&self) -> Option<Point> {
        let mut g: Point = vec![0.0; self.dim()? as usize].into();

        for v in &self.vertices {
            g += v;
        }

        Some(g / (self.vertices.len() as f64))
    }

    /// Gets the edge lengths of all edges in the polytope, in order.
    pub fn edge_lengths(&self) -> Vec<f64> {
        let mut edge_lengths = Vec::new();

        // If there are no edges, we just return the empty vector.
        if let Some(edges) = self.abs.get(1) {
            edge_lengths.reserve_exact(edges.len());

            for edge in edges.iter() {
                let sub0 = edge.subs[0];
                let sub1 = edge.subs[1];

                edge_lengths.push((&self.vertices[sub0] - &self.vertices[sub1]).norm());
            }
        }

        edge_lengths
    }

    /// Checks whether a polytope is equilateral to a fixed precision, and with
    /// a specified edge length.
    pub fn is_equilateral_with_len(&self, len: f64) -> bool {
        let edge_lengths = self.edge_lengths().into_iter();

        // Checks that every other edge length is equal to the first.
        for edge_len in edge_lengths {
            if abs_diff_eq!(edge_len, len, epsilon = EPS) {
                return false;
            }
        }

        true
    }

    /// Checks whether a polytope is equilateral to a fixed precision.
    pub fn is_equilateral(&self) -> bool {
        if let Some(vertices) = self.element_vertices_ref(1, 0) {
            let (v0, v1) = (vertices[0], vertices[1]);

            return self.is_equilateral_with_len((v0 - v1).norm());
        }

        true
    }

    /// I haven't actually implemented this in the general case.
    ///
    /// # Todo
    /// Maybe make this work in the general case?
    pub fn midradius(&self) -> f64 {
        let vertices = &self.vertices;
        let edges = &self[0];
        let edge = &edges[0];

        let sub0 = edge.subs[0];
        let sub1 = edge.subs[1];

        (&vertices[sub0] + &vertices[sub1]).norm() / 2.0
    }

    /// Returns the dual of a polytope with a given reciprocation sphere, or
    /// `None` if any facets pass through the reciprocation center.
    pub fn dual_with_sphere(&self, sphere: &Hypersphere) -> Option<Self> {
        let mut clone = self.clone();

        if clone.dual_mut_with_sphere(sphere).is_ok() {
            Some(clone)
        } else {
            None
        }
    }

    /// Builds the dual of a polytope with a given reciprocation sphere in
    /// place, or does nothing in case any facets go through the reciprocation
    /// center. Returns the dual if successful, and `None` otherwise.
    pub fn dual_mut_with_sphere(&mut self, sphere: &Hypersphere) -> Result<(), ()> {
        // If we're dealing with a nullitope, the dual is itself.
        let rank = self.rank();
        if rank == -1 {
            return Ok(());
        }
        // In the case of points, we reciprocate them.
        else if rank == 0 {
            for v in self.vertices.iter_mut() {
                if sphere.reciprocate(v).is_err() {
                    return Err(());
                }
            }
        }

        // We project the sphere's center onto the polytope's hyperplane to
        // avoid skew weirdness.
        let h = Subspace::from_points(&self.vertices);
        let o = h.project(&sphere.center);

        let mut projections;

        // We project our inversion center onto each of the facets.
        if rank >= 2 {
            let facet_count = self.el_count(rank - 1);
            projections = Vec::with_capacity(facet_count);

            for idx in 0..facet_count {
                projections.push(
                    Subspace::from_point_refs(&self.element_vertices_ref(rank - 1, idx).unwrap())
                        .project(&o),
                );
            }
        }
        // If our polytope is 1D, the vertices themselves are the facets.
        else {
            projections = self.vertices.clone();
        }

        // Reciprocates the projected points.
        for v in projections.iter_mut() {
            if sphere.reciprocate(v).is_err() {
                return Err(());
            }
        }

        self.vertices = projections;

        // Takes the abstract dual.
        self.abs.dual_mut();

        Ok(())
    }

    /// Gets the references to the (geometric) vertices of an element on the
    /// polytope.
    pub fn element_vertices_ref(&self, rank: isize, idx: usize) -> Option<Vec<&Point>> {
        Some(
            self.abs
                .element_vertices(rank, idx)?
                .iter()
                .map(|&v| &self.vertices[v])
                .collect(),
        )
    }

    /// Gets the (geometric) vertices of an element on the polytope.
    pub fn element_vertices(&self, rank: isize, idx: usize) -> Option<Vec<Point>> {
        Some(
            self.element_vertices_ref(rank, idx)?
                .into_iter()
                .cloned()
                .collect(),
        )
    }

    /// Generates the vertices for either a tegum or a pyramid product with two
    /// given vertex sets and a given height.
    fn duopyramid_vertices(p: &[Point], q: &[Point], height: f64, tegum: bool) -> Vec<Point> {
        let p_dim = p[0].len();
        let q_dim = q[0].len();

        let dim = p_dim + q_dim + tegum as usize;

        let mut vertices = Vec::with_capacity(p.len() + q.len());

        // The vertices corresponding to products of p's nullitope with q's
        // vertices.
        for q_vertex in q {
            let mut prod_vertex = Vec::with_capacity(dim);
            let pad = p_dim;

            // Pads prod_vertex to the left.
            prod_vertex.resize(pad, 0.0);

            // Copies q_vertex into prod_vertex.
            for &c in q_vertex.iter() {
                prod_vertex.push(c);
            }

            // Adds the height, in case of a pyramid product.
            if !tegum {
                prod_vertex.push(height / 2.0);
            }

            vertices.push(prod_vertex.into());
        }

        // The vertices corresponding to products of q's nullitope with p's
        // vertices.
        for p_vertex in p {
            let mut prod_vertex = Vec::with_capacity(dim);

            // Copies p_vertex into prod_vertex.
            for &c in p_vertex.iter() {
                prod_vertex.push(c);
            }

            // Pads prod_vertex to the right.
            prod_vertex.resize(p_dim + q_dim, 0.0);

            // Adds the height, in case of a pyramid product.
            if !tegum {
                prod_vertex.push(-height / 2.0);
            }

            vertices.push(prod_vertex.into());
        }

        vertices
    }

    /// Generates the vertices for a duoprism with two given vertex sets.
    fn duoprism_vertices(p: &[Point], q: &[Point]) -> Vec<Point> {
        let mut vertices = Vec::with_capacity(p.len() * q.len());

        // Concatenates all pairs of vertices in order.
        for p_vertex in p {
            for q_vertex in q {
                let p_vertex = p_vertex.into_iter();
                let q_vertex = q_vertex.into_iter();

                vertices.push(p_vertex.chain(q_vertex).cloned().collect::<Vec<_>>().into());
            }
        }

        vertices
    }

    /// Generates a duopyramid from two given polytopes with a given height.
    pub fn duopyramid_with_height(p: &Self, q: &Self, height: f64) -> Self {
        Self::new(
            Self::duopyramid_vertices(&p.vertices, &q.vertices, height, false),
            Abstract::duopyramid(&p.abs, &q.abs),
        )
    }

    /// Projects the vertices of the polytope into the lowest dimension possible.
    /// If the polytope's subspace is already of full rank, this is a no-op.
    pub fn flatten(&mut self) {
        let subspace = Subspace::from_points(&self.vertices);

        if !subspace.is_full_rank() {
            for v in self.vertices.iter_mut() {
                *v = subspace.flatten(v);
            }
        }
    }

    /// Takes the cross-section of a polytope through a given hyperplane.
    ///
    /// # Todo
    /// We should make this function take a general [`Subspace`] instead.
    pub fn slice(&self, slice: Hyperplane) -> Self {
        let mut vertices = Vec::new();

        let mut abs = Abstract::new();

        // We map all indices of k-elements in the original polytope to the
        // indices of the new (k-1)-elements resulting from taking their
        // intersections with the slicing hyperplane.
        let mut hash_element = HashMap::new();

        // Determines the vertices of the cross-section.
        for (idx, edge) in self[1].iter().enumerate() {
            let segment = Segment(
                self.vertices[edge.subs[0]].clone(),
                self.vertices[edge.subs[1]].clone(),
            );

            // If we got ourselves a new vertex:
            if let Some(p) = slice.intersect(segment) {
                hash_element.insert(idx, vertices.len());
                vertices.push(slice.flatten(&p));
            }
        }

        let vertex_count = vertices.len();

        // The slice does not intersect the polytope.
        if vertex_count == 0 {
            return Self::nullitope();
        }

        abs.push(ElementList::min(vertex_count));
        abs.push(ElementList::vertices(vertex_count));

        // Takes care of building everything else.
        for r in 2..self.rank() {
            let mut new_hash_element = HashMap::new();
            let mut new_els = ElementList::new();

            for (idx, el) in self[r].iter().enumerate() {
                let mut new_subs = Subelements::new();
                for sub in el.subs.iter() {
                    if let Some(&v) = hash_element.get(sub) {
                        new_subs.push(v);
                    }
                }

                // If we got ourselves a new edge:
                if !new_subs.is_empty() {
                    new_hash_element.insert(idx, new_els.len());
                    new_els.push(Element::from_subs(new_subs));
                }
            }

            abs.push_subs(new_els);
            hash_element = new_hash_element;
        }

        // Adds a maximal element manually.
        let facet_count = abs.last().unwrap().len();
        abs.push_subs(ElementList::max(facet_count));

        Self::new(vertices, abs)
    }
}

impl Polytope for Concrete {
    type Dual = Option<Self>;
    type DualMut = Result<(), ()>;

    /// Returns the rank of the polytope.
    fn rank(&self) -> isize {
        self.abs.rank()
    }

    /// Gets the number of elements of a given rank.
    fn el_count(&self, rank: isize) -> usize {
        self.abs.el_count(rank)
    }

    /// Gets the number of elements of all ranks.
    fn el_counts(&self) -> RankVec<usize> {
        self.abs.el_counts()
    }

    /// Builds the unique polytope of rank −1.
    fn nullitope() -> Self {
        Self::new(Vec::new(), Abstract::nullitope())
    }

    /// Builds the unique polytope of rank 0.
    fn point() -> Self {
        Self::new(vec![vec![].into()], Abstract::point())
    }

    /// Builds a dyad with unit edge length.
    fn dyad() -> Self {
        Self::new(vec![vec![-0.5].into(), vec![0.5].into()], Abstract::dyad())
    }

    /// Builds a convex regular polygon with `n` sides and unit edge length.
    fn polygon(n: usize) -> Self {
        Self::grunbaum_star_polygon(n, 1)
    }

    /// Returns the dual of a polytope, or `None` if any facets pass through the
    /// origin.
    fn dual(&self) -> Self::Dual {
        let mut clone = self.clone();

        if clone.dual_mut().is_ok() {
            Some(clone)
        } else {
            None
        }
    }

    /// Builds the dual of a polytope in place, or does nothing in case any
    /// facets go through the origin. Returns the dual if successful, and `None`
    /// otherwise.
    fn dual_mut(&mut self) -> Self::DualMut {
        self.dual_mut_with_sphere(&Hypersphere::unit(self.dim().unwrap_or(1)))
    }

    /// "Appends" a polytope into another, creating a compound polytope. Fails
    /// if the polytopes have different ranks.
    fn append(&mut self, mut p: Self) -> Result<(), ()> {
        if self.abs.append(p.abs).is_err() {
            return Err(());
        }

        self.vertices.append(&mut p.vertices);
        Ok(())
    }

    fn element(&self, rank: isize, idx: usize) -> Option<Self> {
        let (vertices, abs) = self.abs.element_and_vertices(rank, idx)?;

        Some(Self::new(
            vertices
                .into_iter()
                .map(|idx| self.vertices[idx].clone())
                .collect(),
            abs,
        ))
    }

    fn element_fig(&self, _rank: isize, _idx: usize) -> Option<Self> {
        todo!()
    }

    fn section(
        &self,
        rank_lo: isize,
        idx_lo: usize,
        rank_hi: isize,
        idx_hi: usize,
    ) -> Option<Self> {
        self.element(rank_hi, idx_hi)?.element_fig(rank_lo, idx_lo)
    }

    /// Builds a [duopyramid](https://polytope.miraheze.org/wiki/Pyramid_product)
    /// from two polytopes.
    fn duopyramid(p: &Self, q: &Self) -> Self {
        Self::duopyramid_with_height(p, q, 1.0)
    }

    /// Builds a [duoprism](https://polytope.miraheze.org/wiki/Prism_product)
    /// from two polytopes.
    fn duoprism(p: &Self, q: &Self) -> Self {
        Self::new(
            Self::duoprism_vertices(&p.vertices, &q.vertices),
            Abstract::duoprism(&p.abs, &q.abs),
        )
    }

    /// Builds a [duotegum](https://polytope.miraheze.org/wiki/Tegum_product)
    /// from two polytopes.
    fn duotegum(p: &Self, q: &Self) -> Self {
        // Point-polytope duotegums are special cases.
        if p.rank() == 0 {
            q.clone()
        } else if q.rank() == 0 {
            p.clone()
        } else {
            Self::new(
                Self::duopyramid_vertices(&p.vertices, &q.vertices, 0.0, true),
                Abstract::duotegum(&p.abs, &q.abs),
            )
        }
    }

    /// Builds a [duocomb](https://polytope.miraheze.org/wiki/Honeycomb_product)
    /// from two polytopes.
    fn duocomb(p: &Self, q: &Self) -> Self {
        Self::new(
            Self::duoprism_vertices(&p.vertices, &q.vertices),
            Abstract::duocomb(&p.abs, &q.abs),
        )
    }

    /// Builds a [ditope](https://polytope.miraheze.org/wiki/Ditope) of a given
    /// polytope.
    fn ditope(&self) -> Self {
        Self::new(self.vertices.clone(), self.abs.ditope())
    }

    /// Builds a [ditope](https://polytope.miraheze.org/wiki/Ditope) of a given
    /// polytope in place.
    fn ditope_mut(&mut self) {
        self.abs.ditope_mut();
    }

    /// Builds a [hosotope](https://polytope.miraheze.org/wiki/hosotope) of a
    /// given polytope.
    fn hosotope(&self) -> Self {
        Self::new(
            vec![vec![-0.5].into(), vec![0.5].into()],
            self.abs.hosotope(),
        )
    }

    /// Builds a [hosotope](https://polytope.miraheze.org/wiki/hosotope) of a
    /// given polytope in place.
    fn hosotope_mut(&mut self) {
        self.vertices = vec![vec![-0.5].into(), vec![0.5].into()];
        self.abs.hosotope_mut();
    }

    /// Builds an [antiprism](https://polytope.miraheze.org/wiki/Antiprism)
    /// based on a given polytope.
    fn antiprism(&self) -> Self {
        todo!()
    }

    /// Determines whether a given polytope is
    /// [orientable](https://polytope.miraheze.org/wiki/Orientability).
    fn orientable(&self) -> bool {
        self.abs.orientable()
    }

    /// Builds a [simplex](https://polytope.miraheze.org/wiki/Simplex) with a
    /// given rank.
    fn simplex(rank: isize) -> Self {
        if rank == -1 {
            Self::nullitope()
        } else {
            let dim = rank as usize;
            let mut vertices = Vec::with_capacity(dim + 1);

            // Adds all points with a single entry equal to √2/2, and all others
            // equal to 0.
            for i in 0..dim {
                let mut v = vec![0.0; dim];
                v[i] = SQRT_2 / 2.0;
                vertices.push(v.into());
            }

            // Adds the remaining vertex, all of whose coordinates are equal.
            let a = (1.0 - ((dim + 1) as f64).sqrt()) * SQRT_2 / (2.0 * dim as f64);
            vertices.push(vec![a; dim].into());

            let mut simplex = Concrete::new(vertices, Abstract::simplex(rank));
            simplex.recenter();
            simplex
        }
    }
}

impl std::ops::Index<isize> for Concrete {
    type Output = ElementList;

    /// Gets the list of elements with a given rank.
    fn index(&self, rank: isize) -> &Self::Output {
        &self.abs[rank]
    }
}

impl std::ops::IndexMut<isize> for Concrete {
    /// Gets the list of elements with a given rank.
    fn index_mut(&mut self, rank: isize) -> &mut Self::Output {
        &mut self.abs[rank]
    }
}
