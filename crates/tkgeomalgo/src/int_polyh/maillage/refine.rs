// Rust translation of CommonPartRefinement, LocalSurfaceRefinement, ComputeDeflections,
// TrianglesDeflectionsRefinementBSB and the static TrianglesDeflectionsRefinement and
// LargeTrianglesDeflectionsRefinement of
// ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The refinement of the triangulations: every marked triangle, or by deflection among
//! the interfering triangles, or the large triangles of a much larger surface.
use super::interfering::{get_interfering_triangles, seek};
use super::{IntPolyhMaillageAffinage, Side};
use crate::int_polyh::triangle_refine::{multiple_refine_in_array, refine_in_array};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use std::collections::HashSet;

impl IntPolyhMaillageAffinage {
    /// `CommonPartRefinement()`: refines every triangle that may intersect, on both
    /// surfaces. `m.common_part_refinement()?`.
    pub fn common_part_refinement(&mut self) -> Result<(), Failure> {
        refine_marked(&mut self.sides[0])?;
        refine_marked(&mut self.sides[1])
    }

    /// `LocalSurfaceRefinement(SurfId)`: refines every triangle that may intersect on the
    /// surface 1 or 2 (nothing for another id). `m.local_surface_refinement(1)?`.
    pub fn local_surface_refinement(&mut self, surf_id: i32) -> Result<(), Failure> {
        if surf_id == 1 {
            refine_marked(&mut self.sides[0])?;
        }
        if surf_id == 2 {
            refine_marked(&mut self.sides[1])?;
        }
        Ok(())
    }

    /// `ComputeDeflections(SurfID)`: the deflection of every triangle, with their minimum
    /// and maximum (`FlecheMin`, `FlecheMax`). `m.compute_deflections(1)?`.
    pub fn compute_deflections(&mut self, surf_id: i32) -> Result<(), Failure> {
        let side: &mut Side = &mut self.sides[Self::rank(surf_id)];
        side.fleche_max = -f64::MAX;
        side.fleche_min = f64::MAX;
        for i in 0..side.triangles.nb_items() {
            let fleche: f64 = side.triangles[i].compute_deflection(&*side.surface, &side.points)?;
            if fleche > side.fleche_max {
                side.fleche_max = fleche;
            }
            if fleche < side.fleche_min {
                side.fleche_min = fleche;
            }
        }
        Ok(())
    }

    /// `TrianglesDeflectionsRefinementBSB()`: refines both surfaces over the triangles
    /// whose boxes interfere, by the deflection criterion `min + 0.8 (max - min)` of each
    /// surface, or, when one surface is much larger than the other (its critical
    /// deflection is above the other's squared box diagonal per sample), the large
    /// triangles of the larger one against the box of the smaller one.
    /// `m.triangles_deflections_refinement_bsb()?`.
    pub fn triangles_deflections_refinement_bsb(&mut self) -> Result<(), Failure> {
        self.compute_deflections(1)?;
        let Some(critique1) = critical_deflection(&self.sides[0]) else {
            return Ok(());
        };
        self.compute_deflections(2)?;
        let Some(critique2) = critical_deflection(&self.sides[1]) else {
            return Ok(());
        };
        let diag1: f64 = sample_diagonal(&self.sides[0])?;
        let diag2: f64 = sample_diagonal(&self.sides[1])?;
        let [side1, side2] = &mut self.sides;
        // The surface with the greater box is refined.
        if diag1 < diag2 {
            if critique2 < diag1 {
                return deflections_refinement((side1, critique1), (side2, critique2));
            }
            let the_box: BoundingBox3 = side1.bbox.clone();
            return large_triangles_refinement(side2, &the_box);
        }
        if critique1 < diag2 {
            return deflections_refinement((side2, critique2), (side1, critique1));
        }
        let the_box: BoundingBox3 = side2.bbox.clone();
        large_triangles_refinement(side1, &the_box)
    }
}

/// `min + (max - min) * 80%` written `FlecheMin * 0.2 + FlecheMax * 0.8`; `None` when the
/// minimum is above the maximum (no triangle).
fn critical_deflection(side: &Side) -> Option<f64> {
    if side.fleche_min > side.fleche_max {
        return None;
    }
    Some(side.fleche_min * 0.2 + side.fleche_max * 0.8)
}

/// The squared diagonal of the box of the points divided by the number of samples.
fn sample_diagonal(side: &Side) -> Result<f64, Failure> {
    let [mut x0, mut y0, mut z0, x1, y1, z1] = side.bbox.get()?;
    x0 -= x1;
    y0 -= y1;
    z0 -= z1;
    let diag: f64 = x0 * x0 + y0 * y0 + z0 * z0;
    let nb_samples_uv: f64 = f64::from(side.nb_samples_u) * f64::from(side.nb_samples_v);
    Ok(diag / nb_samples_uv)
}

/// Refines every triangle of the side that may intersect (the triangles present at the
/// start).
fn refine_marked(side: &mut Side) -> Result<(), Failure> {
    let fin_init: i32 = side.triangles.nb_items();
    for i in 0..fin_init {
        if side.triangles[i].is_intersection_possible() {
            refine_in_array(
                i,
                &*side.surface,
                &mut side.points,
                &mut side.triangles,
                &mut side.edges,
            )?;
        }
    }
    Ok(())
}

/// `TrianglesDeflectionsRefinement(theS1, ..., theFlecheCritique1, theS2, ...,
/// theFlecheCritique2)`: among the interfering triangles, refines those above their
/// surface's critical deflection; the triangles of either surface that interfere with
/// none become impossible.
fn deflections_refinement(
    (s1, critique1): (&mut Side, f64),
    (s2, critique2): (&mut Side, f64),
) -> Result<(), Failure> {
    let couples = get_interfering_triangles(
        (&mut s1.triangles, &s1.points),
        (&mut s2.triangles, &s2.points),
    )?;
    // The number of triangles changes during the refinement.
    let (fin_tt1, fin_tt2) = (s1.triangles.nb_items(), s2.triangles.nb_items());
    let mut int_s2: HashSet<i32> = HashSet::new();
    for i_s1 in 0..fin_tt1 {
        if !s1.triangles[i_s1].is_intersection_possible() {
            continue;
        }
        let Some(list) = seek(&couples, i_s1).filter(|l| !l.is_empty()) else {
            s1.triangles[i_s1].set_intersection_possible(false);
            continue;
        };
        if s1.triangles[i_s1].deflection() > critique1 {
            refine_in_array(
                i_s1,
                &*s1.surface,
                &mut s1.points,
                &mut s1.triangles,
                &mut s1.edges,
            )?;
        }
        for &i_s2 in list {
            if int_s2.insert(i_s2) && s2.triangles[i_s2].deflection() > critique2 {
                // The larger triangles are refined.
                refine_in_array(
                    i_s2,
                    &*s2.surface,
                    &mut s2.points,
                    &mut s2.triangles,
                    &mut s2.edges,
                )?;
            }
        }
    }
    if (int_s2.len() as i32) < fin_tt2 {
        for i_s2 in 0..fin_tt2 {
            if !int_s2.contains(&i_s2) {
                s2.triangles[i_s2].set_intersection_possible(false);
            }
        }
    }
    Ok(())
}

/// `LargeTrianglesDeflectionsRefinement(theS, theTriangles, theEdges, thePoints,
/// theOppositeBox)`: the triangles whose box meets the opposite box are refined, several
/// times while their deflection is above half the smallest side of that box; the others
/// become impossible.
fn large_triangles_refinement(side: &mut Side, opposite: &BoundingBox3) -> Result<(), Failure> {
    let mut lit: Vec<i32> = Vec::new();
    for i in 0..side.triangles.nb_items() {
        let triangle = &mut side.triangles[i];
        if !triangle.is_intersection_possible() || triangle.is_degenerated() {
            continue;
        }
        let a_box: &BoundingBox3 = triangle.bounding_box(&side.points);
        if a_box.is_void() {
            continue;
        }
        if a_box.is_out_box(opposite) {
            triangle.set_intersection_possible(false);
            continue;
        }
        lit.push(i);
    }
    if lit.is_empty() {
        return Ok(());
    }
    let critere_affinage: f64 = smallest_side(opposite)? * 0.5;
    for i in lit {
        if side.triangles[i].deflection() > critere_affinage {
            let surface = side.surface.clone();
            multiple_refine_in_array((critere_affinage, opposite), i, &*surface, &mut side.mesh())?;
        } else {
            refine_in_array(
                i,
                &*side.surface,
                &mut side.points,
                &mut side.triangles,
                &mut side.edges,
            )?;
        }
    }
    Ok(())
}

/// The smallest side of the box (the criterion of refining is minimized).
fn smallest_side(the_box: &BoundingBox3) -> Result<f64, Failure> {
    let [x0, y0, z0, x1, y1, z1] = the_box.get()?;
    let dx: f64 = (x1 - x0).abs();
    let dy: f64 = (y1 - y0).abs();
    let mut diag: f64 = (z1 - z0).abs();
    let dd: f64 = if dx > dy { dy } else { dx };
    if diag > dd {
        diag = dd;
    }
    Ok(diag)
}
