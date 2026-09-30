// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/
// IntPatch_InterferencePolyhedron.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_InterferencePolyhedron`: the interference of two polyhedra (section
//! points, section lines and tangent zones). The constructors, the couple
//! characteristics and the BVH selection are here; the intersection of two
//! triangles is in `intersect.rs` and `section.rs`, the tangent zones in
//! `tangent_zone.rs`.
mod intersect;
mod section;
mod tangent_zone;

use super::bvh_traversal::IntPatchBVHTraversal;
use super::polyhedron::IntPatchPolyhedron;
use super::polyhedron_bvh::{IntPatchPolyhedronBVH, IntPatchPolyhedronTool as Tool};
use crate::intf::{Interference, nonzero_tolerance};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Xyz;
use std::ops::{Deref, DerefMut};

/// `Pourcent3`: `i mod 3` for `i` in `0..9`.
pub(super) const POURCENT3: [usize; 9] = [0, 1, 2, 0, 1, 2, 0, 1, 2];

/// `IntPatch_InterferencePolyhedron`: the `Intf_Interference` result (reached through
/// `Deref`) and the characteristics of the current couple of triangles (`OI`, `TI`,
/// `dpOpT`, `dpOeT`, `deOpT`, `voo`, `vtt`, `Incidence`).
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchInterferencePolyhedron;
/// let interference = IntPatchInterferencePolyhedron::new();
/// assert_eq!(interference.nb_section_points(), 0);
/// ```
#[derive(Clone, Default)]
pub struct IntPatchInterferencePolyhedron {
    base: Interference,
    pub(super) oi: [i32; 3],
    pub(super) ti: [i32; 3],
    pub(super) dp_op_t: [[f64; 3]; 3],
    pub(super) dp_oe_t: [[f64; 3]; 3],
    pub(super) de_op_t: [[f64; 3]; 3],
    pub(super) voo: [Xyz; 3],
    pub(super) vtt: [Xyz; 3],
    pub(super) incidence: f64,
}

impl Deref for IntPatchInterferencePolyhedron {
    type Target = Interference;
    fn deref(&self) -> &Interference {
        &self.base
    }
}

impl DerefMut for IntPatchInterferencePolyhedron {
    fn deref_mut(&mut self) -> &mut Interference {
        &mut self.base
    }
}

impl IntPatchInterferencePolyhedron {
    /// `IntPatch_InterferencePolyhedron()`: an empty interference of two polyhedra.
    /// `IntPatchInterferencePolyhedron::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `IntPatch_InterferencePolyhedron(Obje1, Obje2)`: the interference of two
    /// polyhedra. `IntPatchInterferencePolyhedron::between(&p1, &p2)?`.
    pub fn between(
        first: &IntPatchPolyhedron,
        second: &IntPatchPolyhedron,
    ) -> Result<Self, Failure> {
        let mut interference = Self::new();
        interference.two_polyhedra(first, second)?;
        Ok(interference)
    }

    /// `IntPatch_InterferencePolyhedron(Obje)`: the self interference of a polyhedron,
    /// computed by comparing it with itself (lbr, 5 July 1996).
    /// `IntPatchInterferencePolyhedron::of_self(&p)?`.
    pub fn of_self(object: &IntPatchPolyhedron) -> Result<Self, Failure> {
        let mut interference = Self::new();
        interference.base = Interference::new(true);
        interference.base.tolerance =
            nonzero_tolerance(Tool::deflection_over_estimation(object) * 2.0);
        interference.interference(object, object)?;
        Ok(interference)
    }

    /// `Perform(Obje1, Obje2)`: clears the result and computes the interference of two
    /// polyhedra. `interference.perform(&p1, &p2)?`.
    pub fn perform(
        &mut self,
        first: &IntPatchPolyhedron,
        second: &IntPatchPolyhedron,
    ) -> Result<(), Failure> {
        self.base.self_interference(false);
        self.two_polyhedra(first, second)
    }

    /// `Perform(Obje)`: clears the result for a self interference and sets its
    /// tolerance; quirk kept, the one-polyhedron `Interference` computes nothing.
    /// `interference.perform_self(&p)`.
    pub fn perform_self(&mut self, object: &IntPatchPolyhedron) {
        self.base.self_interference(true);
        self.base.tolerance = nonzero_tolerance(Tool::deflection_over_estimation(object) * 2.0);
    }

    /// The body shared by the constructor and `Perform` of two polyhedra: nothing when
    /// the boxes do not overlap.
    fn two_polyhedra(
        &mut self,
        first: &IntPatchPolyhedron,
        second: &IntPatchPolyhedron,
    ) -> Result<(), Failure> {
        if Tool::bounding(first).is_out_box(Tool::bounding(second)) {
            return Ok(());
        }
        let tolerance: f64 =
            Tool::deflection_over_estimation(first) + Tool::deflection_over_estimation(second);
        self.base.tolerance = nonzero_tolerance(tolerance);
        self.interference(first, second)
    }

    /// `Interference(Obje1, Obje2)`: intersects the couples of triangles whose boxes
    /// overlap, in the order of the BVH descent.
    fn interference(
        &mut self,
        first: &IntPatchPolyhedron,
        second: &IntPatchPolyhedron,
    ) -> Result<(), Failure> {
        let mut set1 = IntPatchPolyhedronBVH::with_polyhedron(first)?;
        let mut set2 = IntPatchPolyhedronBVH::with_polyhedron(second)?;
        let mut traversal = IntPatchBVHTraversal::new();
        traversal.perform(&mut set1, &mut set2, self.base.self_interference)?;
        for pair in traversal.pairs() {
            self.intersect(pair.first, first, pair.second, second)?;
        }
        Ok(())
    }

    /// `CoupleCharacteristics(FirstPol, SeconPol)`: the edges of both triangles and the
    /// distances between their vertices and edges, signed by the common normal.
    pub(super) fn couple_characteristics(
        &mut self,
        first: &IntPatchPolyhedron,
        second: &IntPatchPolyhedron,
    ) -> Result<(), Failure> {
        let (po, pt) = (self.object_points(first)?, self.tool_points(second)?);
        for n1 in 0..3 {
            let n2: usize = POURCENT3[n1 + 1];
            self.voo[n1] = po[n2] - po[n1];
            self.vtt[n1] = pt[n2] - pt[n1];
        }
        let (voo, vtt) = (self.voo, self.vtt);
        let mut vvec: Xyz = (voo[0] ^ voo[1]) + (voo[1] ^ voo[2]) + (voo[2] ^ voo[0]);
        let vnor_t: Xyz = (vtt[0] ^ vtt[1]) + (vtt[1] ^ vtt[2]) + (vtt[2] ^ vtt[0]);
        if vnor_t.modulus() > vvec.modulus() {
            vvec = vnor_t;
        }
        for (n1, o) in po.iter().enumerate() {
            for (n2, t) in pt.iter().enumerate() {
                self.vertex_edge_distances(n1, n2, *o - *t, vvec);
            }
        }
        Ok(())
    }

    /// `dpOpT`, `dpOeT` and `deOpT` of the object vertex `n1` and the tool vertex `n2`.
    fn vertex_edge_distances(&mut self, n1: usize, n2: usize, vto: Xyz, vvec: Xyz) {
        self.dp_op_t[n1][n2] = vto.modulus();
        let mut lg: f64 = self.vtt[n2].modulus();
        if lg > 1e-16 {
            // tol:length (RealEpsilon)
            let vv: Xyz = vto ^ self.vtt[n2];
            lg = if vvec * vv > 0.0 { lg } else { -lg };
            self.dp_oe_t[n1][n2] = vv.modulus() / lg;
        } else {
            self.dp_oe_t[n1][n2] = self.dp_op_t[n1][n2];
        }
        lg = self.voo[n1].modulus();
        if lg > 1e-16 {
            // tol:length (RealEpsilon)
            let vv: Xyz = vto ^ self.voo[n1];
            lg = if vvec * vv > 0.0 { -lg } else { lg };
            self.de_op_t[n1][n2] = vv.modulus() / lg;
        } else {
            self.de_op_t[n1][n2] = self.dp_op_t[n1][n2];
        }
    }

    /// The points of the object triangle `OI`.
    pub(super) fn object_points(&self, first: &IntPatchPolyhedron) -> Result<[Xyz; 3], Failure> {
        points_of(first, self.oi)
    }

    /// The points of the tool triangle `TI`.
    pub(super) fn tool_points(&self, second: &IntPatchPolyhedron) -> Result<[Xyz; 3], Failure> {
        points_of(second, self.ti)
    }
}

fn points_of(polyhedron: &IntPatchPolyhedron, indices: [i32; 3]) -> Result<[Xyz; 3], Failure> {
    Ok([
        Tool::point(polyhedron, indices[0])?.xyz(),
        Tool::point(polyhedron, indices[1])?.xyz(),
        Tool::point(polyhedron, indices[2])?.xyz(),
    ])
}
