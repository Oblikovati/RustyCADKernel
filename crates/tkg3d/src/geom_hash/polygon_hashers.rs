// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_Polygon2DHasher.hxx/.cxx, GeomHash_Polygon3DHasher.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Hashers of the mesh polygons (`Poly_Polygon2D`, `Poly_Polygon3D`): the hash reads
//! the node count, the deflection and the end nodes (and end parameters), each value
//! in its own zero-initialized slot; the comparison walks every node.
//!
//! The default tolerances are both `Precision::Computational()`, unlike the geometry
//! hashers.
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d_hash::hash_key::{combined_hash, int_hash, rounded_hash};
use rustycad_tkg2d::hasher_tolerances;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::poly::{Polygon2D, Polygon3D};

hasher_tolerances!(
    /// `GeomHash_Polygon2DHasher`: hashes and compares `Poly_Polygon2D` handles; a null
    /// handle hashes to 0; `GeomHashPolygon2DHasher::default().hash(&polygon)`.
    GeomHashPolygon2DHasher,
    precision::computational(),
    precision::computational()
);
hasher_tolerances!(
    /// `GeomHash_Polygon3DHasher`: hashes and compares `Poly_Polygon3D` handles; a null
    /// handle hashes to 0; `GeomHashPolygon3DHasher::default().hash(&polygon)`.
    GeomHashPolygon3DHasher,
    precision::computational(),
    precision::computational()
);

/// The rounded hashes of `coordinates` written into `slots` in order
/// (`quantizedHash(value, aFactor)`); `write_rounded(&mut slots[2..4], &[x, y], factor)`.
pub(super) fn write_rounded(slots: &mut [usize], coordinates: &[f64], factor: f64) {
    for (slot, &value) in slots.iter_mut().zip(coordinates) {
        *slot = rounded_hash(value, factor);
    }
}

/// `x, y` of a 2D point; `xy_of(p)`.
pub(super) fn xy_of(point: Point2) -> [f64; 2] {
    [point.xy().x(), point.xy().y()]
}
/// `x, y, z` of a 3D point; `xyz_of(p)`.
pub(super) fn xyz_of(point: Point3) -> [f64; 3] {
    let xyz = point.xyz();
    [xyz.x(), xyz.y(), xyz.z()]
}

/// No coordinate pair farther apart than `comp_tolerance`: the mesh hashers test
/// `std::abs(a - b) > CompTolerance` to reject, so a NaN difference does not reject
/// (unlike the `<=` test of the point hasher); `none_beyond(1e-7, &[0.0], &[0.0])`.
pub(super) fn none_beyond(comp_tolerance: f64, first: &[f64], second: &[f64]) -> bool {
    !first
        .iter()
        .zip(second)
        .any(|(a, b)| (a - b).abs() > comp_tolerance)
}

/// Both handles null, or neither: `Some(result)` when a null handle decides the
/// comparison (`IsNull() || IsNull()`); `null_decides(&a, &b)`.
pub(super) fn null_decides<T: rustycad_tkernel::standard::RuntimeTyped>(
    first: &Handle<T>,
    second: &Handle<T>,
) -> Option<bool> {
    (first.is_null() || second.is_null()).then(|| first.is_null() && second.is_null())
}

impl GeomHashPolygon2DHasher {
    /// Node count, deflection and the rounded first and last nodes
    /// (`operator()(const occ::handle<Poly_Polygon2D>&)`); `hasher.hash(&polygon)`.
    pub fn hash(&self, polygon: &Handle<Polygon2D>) -> usize {
        let Some(poly) = polygon.get() else {
            return 0;
        };
        let factor: f64 = 1.0 / self.hash_tolerance;
        let mut slots = [0usize; 6];
        slots[0] = int_hash(poly.nb_nodes());
        slots[1] = rounded_hash(poly.deflection(), factor);
        if poly.nb_nodes() > 0 {
            let nodes = poly.nodes();
            let ends = [nodes.as_slice()[0], nodes.as_slice()[nodes.len() - 1]];
            write_rounded(&mut slots[2..4], &xy_of(ends[0]), factor);
            write_rounded(&mut slots[4..6], &xy_of(ends[1]), factor);
        }
        combined_hash(&slots)
    }
    /// Same node count, deflection and every node within `comp_tolerance`
    /// (`operator()(handle, handle)`); `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Handle<Polygon2D>, second: &Handle<Polygon2D>) -> bool {
        if let Some(decided) = null_decides(first, second) {
            return decided;
        }
        let (Some(a), Some(b)) = (first.get(), second.get()) else {
            return false;
        };
        if a.nb_nodes() != b.nb_nodes() {
            return false;
        }
        let comp: f64 = self.comp_tolerance;
        if !none_beyond(comp, &[a.deflection()], &[b.deflection()]) {
            return false;
        }
        let (nodes1, nodes2) = (a.nodes(), b.nodes());
        let mut pairs = nodes1.as_slice().iter().zip(nodes2.as_slice());
        pairs.all(|(p1, p2)| none_beyond(comp, &xy_of(*p1), &xy_of(*p2)))
    }
}

impl GeomHashPolygon3DHasher {
    /// Node count, parameter presence, deflection, the rounded first and last nodes and,
    /// with parameters, the rounded first and last parameters
    /// (`operator()(const occ::handle<Poly_Polygon3D>&)`); `hasher.hash(&polygon)`.
    pub fn hash(&self, polygon: &Handle<Polygon3D>) -> usize {
        let Some(poly) = polygon.get() else {
            return 0;
        };
        let factor: f64 = 1.0 / self.hash_tolerance;
        let mut slots = [0usize; 11];
        slots[0] = int_hash(poly.nb_nodes());
        slots[1] = int_hash(i32::from(poly.has_parameters()));
        slots[2] = rounded_hash(poly.deflection(), factor);
        if poly.nb_nodes() > 0 {
            let nodes = poly.nodes();
            let ends = [nodes.as_slice()[0], nodes.as_slice()[nodes.len() - 1]];
            write_rounded(&mut slots[3..6], &xyz_of(ends[0]), factor);
            write_rounded(&mut slots[6..9], &xyz_of(ends[1]), factor);
            if let Ok(parameters) = poly.parameters() {
                let values = parameters.as_slice();
                write_rounded(
                    &mut slots[9..11],
                    &[values[0], values[values.len() - 1]],
                    factor,
                );
            }
        }
        combined_hash(&slots)
    }
    /// Same node count, parameter presence and deflection, then every node and every
    /// parameter within `comp_tolerance` (`operator()(handle, handle)`);
    /// `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &Handle<Polygon3D>, second: &Handle<Polygon3D>) -> bool {
        if let Some(decided) = null_decides(first, second) {
            return decided;
        }
        let (Some(a), Some(b)) = (first.get(), second.get()) else {
            return false;
        };
        if a.nb_nodes() != b.nb_nodes() || a.has_parameters() != b.has_parameters() {
            return false;
        }
        let comp: f64 = self.comp_tolerance;
        if !none_beyond(comp, &[a.deflection()], &[b.deflection()]) {
            return false;
        }
        same_points3(comp, a.nodes().as_slice(), b.nodes().as_slice())
            && same_parameters(comp, a, b)
    }
}

/// No point pair farther apart than `comp_tolerance` on any coordinate, in order;
/// `same_points3(1e-7, &nodes1, &nodes2)`.
pub(super) fn same_points3(comp: f64, first: &[Point3], second: &[Point3]) -> bool {
    let mut pairs = first.iter().zip(second);
    pairs.all(|(p1, p2)| none_beyond(comp, &xyz_of(*p1), &xyz_of(*p2)))
}

// The parameters pairwise within tolerance; true without parameters.
fn same_parameters(comp: f64, first: &Polygon3D, second: &Polygon3D) -> bool {
    let (Ok(params1), Ok(params2)) = (first.parameters(), second.parameters()) else {
        return true;
    };
    none_beyond(comp, params1.as_slice(), params2.as_slice())
}

impl HashPolicy<Handle<Polygon2D>> for GeomHashPolygon2DHasher {
    fn hash(&self, key: &Handle<Polygon2D>) -> usize {
        GeomHashPolygon2DHasher::hash(self, key)
    }
    fn equal(&self, first: &Handle<Polygon2D>, second: &Handle<Polygon2D>) -> bool {
        GeomHashPolygon2DHasher::equal(self, first, second)
    }
}
impl HashPolicy<Handle<Polygon3D>> for GeomHashPolygon3DHasher {
    fn hash(&self, key: &Handle<Polygon3D>) -> usize {
        GeomHashPolygon3DHasher::hash(self, key)
    }
    fn equal(&self, first: &Handle<Polygon3D>, second: &Handle<Polygon3D>) -> bool {
        GeomHashPolygon3DHasher::equal(self, first, second)
    }
}
