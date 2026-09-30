// Rust translation of the static helpers of ModelingAlgorithms/TKGeomAlgo/TopTrans/TopTrans_SurfaceTransition.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The file-static helpers of TopTrans_SurfaceTransition: the sector of an angle and
//! the choice of the boundary face nearest the reference surface in one sector.
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::gp::Direction3;

/// The value of an undefined sector angle (`100.` natively).
pub(crate) const UNDEFINED_ANGLE: f64 = 100.0;

/// What FUN_refnearest decides for one sector; the native codes are M_noupdate (0),
/// M_updateREF (1), M_Ointernal (10) and M_Unknown (-100).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NearestUpdate {
    /// The stored face stays the nearest.
    Keep,
    /// The new face becomes the nearest.
    Replace,
    /// The faces are tangent with opposite orientations: the sector becomes INTERNAL.
    Internal,
    /// The faces are tangent with the same orientation: the transition is undefined.
    Unknown,
}

/// A face stored in, or offered to, one sector: its angle, orientation and curvature.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SectorFace {
    pub angle: f64,
    pub orientation: Orientation,
    pub curvature: f64,
}

/// FUN_nCinsideS: the normal to the curve tangent `tangent` on the surface of normal
/// `normal`, oriented inside the surface; raises when they are parallel.
pub(crate) fn inside_normal(
    tangent: Direction3,
    normal: Direction3,
) -> Result<Direction3, Failure> {
    normal.crossed(tangent)
}

/// FUN_OO: the opposite sector index (1 and 2 swap, 0 stays).
pub(crate) fn opposite(index: usize) -> usize {
    match index {
        1 => 2,
        2 => 1,
        _ => 0,
    }
}

/// FUN_Ang: the oriented angle from `beafter` to the inside normal of the face, around
/// the tangent; the normal is reversed for a REVERSED face.
pub(crate) fn face_angle(
    beafter: Direction3,
    tangent: Direction3,
    normal: Direction3,
    orientation: Orientation,
) -> Result<f64, Failure> {
    let mut on_face = inside_normal(tangent, normal)?;
    if orientation == Orientation::Reversed {
        on_face.reverse();
    }
    Ok(beafter.angle_with_ref(on_face, tangent))
}

/// FUN_getSTA: the sector of an angle, `(i, j)` with i the sign of its cosine and j of
/// its sine: 0 when null at `tola`, 1 when positive, 2 when negative.
pub(crate) fn sector(angle: f64, tola: f64) -> (usize, usize) {
    (sign_index(angle.cos(), tola), sign_index(angle.sin(), tola))
}

fn sign_index(value: f64, tola: f64) -> usize {
    if value.abs() < tola {
        0
    } else if value > 0.0 {
        1
    } else {
        2
    }
}

/// The first FUN_refnearest: the nearest face of a sector for planar faces, compared by
/// the absolute cosine of their angles.
pub(crate) fn nearest_planar(stored: SectorFace, offered: SectorFace, tola: f64) -> NearestUpdate {
    if stored.angle == UNDEFINED_ANGLE {
        return NearestUpdate::Replace;
    }
    let dcos: f64 = stored.angle.cos().abs() - offered.angle.cos().abs();
    if dcos.abs() < tola {
        // Tangent faces on the same side: opposite orientations give INTERNAL
        // (EXTERNAL refers to no logical case).
        return tangent_update(stored.orientation, offered.orientation);
    }
    if dcos > 0.0 {
        NearestUpdate::Keep
    } else {
        NearestUpdate::Replace
    }
}

fn tangent_update(stored: Orientation, offered: Orientation) -> NearestUpdate {
    if top_abs::complement(offered) == stored {
        NearestUpdate::Internal
    } else {
        NearestUpdate::Unknown
    }
}

/// The sector `(i, j)` and the reference surface curvature that the curved
/// FUN_refnearest reads beside the two faces.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CurvedSector {
    pub i: usize,
    pub j: usize,
    pub reference_curvature: f64,
    pub tola: f64,
}

/// The second FUN_refnearest (eap, 25 Mar 2002): the nearest face of a sector for
/// curved faces. Returns the decision and whether the faces touch (the native
/// TouchFlag is only ever set, never cleared, here).
pub(crate) fn nearest_curved(
    at: CurvedSector,
    stored: SectorFace,
    offered: SectorFace,
) -> (NearestUpdate, bool) {
    if stored.angle == UNDEFINED_ANGLE {
        return (first_in_sector(at, offered), false);
    }
    let dcos: f64 = stored.angle.cos().abs() - offered.angle.cos().abs();
    if dcos.abs() < at.tola {
        return (same_cosine(at, stored, offered), false);
    }
    let update = if dcos > 0.0 {
        NearestUpdate::Keep
    } else {
        NearestUpdate::Replace
    };
    (update, stored.orientation != offered.orientation)
}

/// The shape of an offered face in its sector, shared by both curved cases.
struct OfferedShape {
    iisj: bool,
    j0: bool,
    null_curv: bool,
    curv_pos: bool,
    curv_neg: bool,
    null_reference: bool,
}

fn offered_shape(at: CurvedSector, offered: SectorFace) -> OfferedShape {
    let abscos: f64 = offered.angle.cos().abs();
    OfferedShape {
        iisj: at.i == at.j,
        j0: abscos < at.tola,
        null_curv: offered.curvature == 0.0,
        curv_pos: offered.curvature > at.tola,
        curv_neg: offered.curvature < -at.tola,
        null_reference: at.reference_curvature == 0.0,
    }
}

/// The undefined-sector case of the curved FUN_refnearest.
fn first_in_sector(at: CurvedSector, offered: SectorFace) -> NearestUpdate {
    let shape = offered_shape(at, offered);
    let i0: bool = (1.0 - offered.angle.cos().abs()).abs() < at.tola;
    if i0 && ((shape.iisj && shape.curv_neg) || (!shape.iisj && shape.curv_pos)) {
        return NearestUpdate::Keep;
    }
    if !shape.j0 {
        return NearestUpdate::Replace;
    }
    let above = !shape.null_reference && at.j == 1;
    if above && shape.iisj && (shape.curv_pos || shape.null_curv) {
        return NearestUpdate::Replace;
    }
    if above && !shape.iisj && (shape.curv_neg || shape.null_curv) {
        return NearestUpdate::Replace;
    }
    if (shape.iisj && shape.curv_pos) || (!shape.iisj && shape.curv_neg) {
        return NearestUpdate::Keep;
    }
    NearestUpdate::Replace
}

/// The same-cosine case of the curved FUN_refnearest: the curvatures decide.
fn same_cosine(at: CurvedSector, stored: SectorFace, offered: SectorFace) -> NearestUpdate {
    // The fixed 1.e-4 curvature gap of the native file, independent of `tola`.
    if (stored.curvature - offered.curvature).abs() < 1.0e-4 {
        return tangent_update(stored.orientation, offered.orientation);
    }
    let shape = offered_shape(at, offered);
    let keep = (shape.iisj && stored.curvature > offered.curvature)
        || (!shape.iisj && stored.curvature < offered.curvature);
    if keep || !shape.j0 || shape.null_reference {
        return if keep {
            NearestUpdate::Keep
        } else {
            NearestUpdate::Replace
        };
    }
    // With a null sine, j == 1 puts the face ABOVE the reference and j == 2 BELOW it.
    let curvature: f64 = offered.curvature.abs();
    let below_keeps = at.j == 2 && curvature < at.reference_curvature;
    let above_keeps = at.j == 1 && curvature > at.reference_curvature;
    if below_keeps || above_keeps {
        NearestUpdate::Keep
    } else {
        NearestUpdate::Replace
    }
}
