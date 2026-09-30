// Rust translation of IntCyCy of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Two cylinders: the analytic cases, else the numerical walking of the two lines of
//! the system of the cylinders.
mod analytic;
mod boundaries;
mod coeffs;
mod critical;
mod finish;
mod inscribe;
mod walk;
mod walk_step;
mod wline;

use super::super::IntPatchImpImpStatus;
use super::plane::{Lines, Points};
use crate::int_surf::IntSurfQuadric;
use analytic::cycy_analytical_intersect;
use boundaries::{WorkWithBoundaries, boundaries_computing};
use coeffs::CoeffsValue;
use inscribe::{inscribe_interval, split_all};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{QuadQuadGeo, ResultType};
use rustycad_tkmath::bnd::{BoundingBox2, Range1};
use rustycad_tkmath::gp::Cylinder3;
use std::f64::consts::PI;
use walk::cycy_no_geometric;

/// The flags of `IntCyCy`: empty, same surface, multiple point.
pub(super) struct CyCyFlags<'a> {
    pub(super) empty: &'a mut bool,
    pub(super) same: &'a mut bool,
    pub(super) multpoint: &'a mut bool,
}

/// `IntCyCy(theQuad1, theQuad2, theTol3D, theTol2D, theUVSurf1, theUVSurf2,
/// isTheEmpty, isTheSameSurface, isTheMultiplePoint, theSlin, theSPnt)`: the analytic
/// result when there is one, else the walking lines parametrized by the U of the
/// cylinder whose useful U range is the longer.
#[allow(clippy::too_many_arguments)]
pub(super) fn int_cycy(
    (quad1, quad2): (&IntSurfQuadric, &IntSurfQuadric),
    tol_3d: f64,
    tol_2d: f64,
    (uv_surf1, uv_surf2): (&BoundingBox2, &BoundingBox2),
    flags: CyCyFlags<'_>,
    slin: &mut Lines,
    spnt: &mut Points,
) -> Result<IntPatchImpImpStatus, Failure> {
    (*flags.empty, *flags.same, *flags.multpoint) = (true, false, false);
    slin.clear();
    spnt.clear();
    let (cyl1, cyl2): (Cylinder3, Cylinder3) = (quad1.cylinder()?, quad2.cylinder()?);
    let mut inter = QuadQuadGeo::new();
    inter.perform_cylinders(cyl1, cyl2, tol_3d)?;
    if !inter.is_done() {
        return Ok(IntPatchImpImpStatus::Fail);
    }
    if inter.type_inter()? != ResultType::NoGeometricSolution {
        let (e, s, m) = (&mut *flags.empty, &mut *flags.same, &mut *flags.multpoint);
        if cycy_analytical_intersect((quad1, quad2), &inter, tol_3d, e, s, m, slin, spnt)? {
            return Ok(IntPatchImpImpStatus::Ok);
        }
        // The analytical handler declined: the walking starts from a clean state.
        (*flags.empty, *flags.same, *flags.multpoint) = (true, false, false);
        slin.clear();
        spnt.clear();
    }
    // Here the intersection line is not an analytical curve.
    let usbou: [[f64; 2]; 2] = [u_bounds(uv_surf1)?, u_bounds(uv_surf2)?];
    let period: f64 = 2.0 * PI;
    let coeffs1 = CoeffsValue::new(&cyl1, &cyl2)?;
    let coeffs2 = CoeffsValue::new(&cyl2, &cyl1)?;
    // The intersection can have two non-connected regions.
    let mut urange: [[Range1; 2]; 2] = [[Range1::default(); 2]; 2];
    if !boundaries_computing(&coeffs1, period, &mut urange[0]) {
        return Ok(IntPatchImpImpStatus::Ok);
    }
    if !boundaries_computing(&coeffs2, period, &mut urange[1]) {
        return Ok(IntPatchImpImpStatus::Ok);
    }
    let sum1: f64 = sum_range(&urange[0], usbou[0], tol_2d, period)?;
    let sum2: f64 = sum_range(&urange[1], usbou[1], tol_2d, period)?;
    // The quadric of the longer range varies (more points, a more precise line); an
    // analytical result does not depend on the order.
    let is_to_reverse: bool = sum2 > sum1;
    let empty: &mut bool = flags.empty;
    if is_to_reverse {
        let bw = WorkWithBoundaries {
            quad1: quad2,
            quad2: quad1,
            coeffs: &coeffs2,
            uv_surf1: uv_surf2,
            uv_surf2: uv_surf1,
            period,
            tol_3d,
            tol_2d,
            is_reverse: true,
        };
        return cycy_no_geometric(&cyl2, &cyl1, &bw, &mut urange[1], empty, (slin, spnt));
    }
    let bw = WorkWithBoundaries {
        quad1,
        quad2,
        coeffs: &coeffs1,
        uv_surf1,
        uv_surf2,
        period,
        tol_3d,
        tol_2d,
        is_reverse: false,
    };
    cycy_no_geometric(&cyl1, &cyl2, &bw, &mut urange[0], empty, (slin, spnt))
}

/// The U bounds `[first, last]` of a UV box.
fn u_bounds(uv: &BoundingBox2) -> Result<[f64; 2], Failure> {
    let [u0, _, u1, _] = uv.get()?;
    Ok([u0, u1])
}

/// The summary length of the ranges inside `[Ufirst, Ulast]` of the cylinder: each
/// range is split by the bounds and the seam, shifted into the bounds and cut by them
/// (e.g. [5, 7] on a full cylinder gives [5, 2PI] and [0, 7-2PI], of length 2).
fn sum_range(
    ranges: &[Range1; 2],
    usbou: [f64; 2],
    tol_2d: f64,
    period: f64,
) -> Result<f64, Failure> {
    let mut list: List<Range1> = List::default();
    list.append(ranges[0]);
    list.append(ranges[1]);
    for split in [usbou[0], usbou[1], 0.0] {
        list = split_all(&list, split, period)?;
    }
    let mut sum: f64 = 0.0;
    for r in list.iter() {
        let mut curr: Range1 = *r;
        let mut bound_r = Range1::default();
        bound_r.add(usbou[0]);
        bound_r.add(usbou[1]);
        if !inscribe_interval(usbou[0], usbou[1], &mut curr, tol_2d, period) {
            // No common block with [Ufirst, Ulast]: inscribe it in the range.
            let (mut f, mut l) = (1.0, 0.0);
            if !curr.get_bounds(&mut f, &mut l) {
                continue;
            }
            if l < usbou[0] {
                curr.shift(period);
            } else if f > usbou[1] {
                curr.shift(-period);
            }
        }
        bound_r.common(curr);
        let delta: f64 = bound_r.delta();
        if delta > 0.0 {
            sum += delta;
        }
    }
    Ok(sum)
}
