// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_tol.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `FTOL_*` functions: the intersection tolerances, deflection and UV step of a
//! pair of faces.
use super::hbox_tool::fbox_get_hbox_tool;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;

/// The tolerances of a pair of faces: `(myTol1, myTol2, Deflection, MaxUV)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FaceTolerances {
    /// `myTol1`.
    pub tol1: f64,
    /// `myTol2`.
    pub tol2: f64,
    /// `Deflection`.
    pub deflection: f64,
    /// `MaxUV`.
    pub max_uv: f64,
}

/// Whether a box is closed and not void.
fn box_ok(b: &BoundingBox3) -> bool {
    !b.is_open_xmin()
        && !b.is_open_xmax()
        && !b.is_open_ymin()
        && !b.is_open_ymax()
        && !b.is_open_zmin()
        && !b.is_open_zmax()
        && !b.is_void()
}

/// The extents of a box.
fn extents(b: &BoundingBox3) -> Result<[f64; 3], Failure> {
    let [x0, y0, z0, x1, y1, z1] = b.get()?;
    Ok([x1 - x0, y1 - y0, z1 - z0])
}

/// The greatest extent of the closed boxes (1 without any), at most 1e6.
fn greatest_extent(b1: &BoundingBox3, b2: &BoundingBox3) -> Result<f64, Failure> {
    let [mut dx, mut dy, mut dz] = match (box_ok(b1), box_ok(b2)) {
        (true, _) => extents(b1)?,
        (false, true) => extents(b2)?,
        (false, false) => [1.0; 3],
    };
    if box_ok(b1) && box_ok(b2) {
        let [ex, ey, ez] = extents(b2)?;
        (dx, dy, dz) = (
            if ex > dx { ex } else { dx },
            if ey > dy { ey } else { dy },
            if ez > dz { ez } else { dz },
        );
    }
    if dx < dy {
        dx = dy;
    }
    if dx < dz {
        dx = dz;
    }
    Ok(if dx > 1_000_000.0 { 1_000_000.0 } else { dx })
}

/// The greatest edge tolerance of a face (at least Precision::Confusion()).
fn edge_tolerance(f: &Face) -> Result<f64, Failure> {
    let mut tolef: f64 = precision::confusion();
    for e in Explorer::new(f.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)? {
        let tole = BRepTool::edge_tolerance(&topods::edge(&e)?)?;
        if tole > tolef {
            tolef = tole;
        }
    }
    Ok(tolef)
}

/// The smaller UV range of the first surface and the larger of the second, the
/// smallest of both, times 0.01 (unused natively).
fn uv_step(s1: &BRepAdaptorSurface, s2: &BRepAdaptorSurface) -> Result<f64, Failure> {
    let (dp1, dp2) = (
        s1.last_u_parameter()? - s1.first_u_parameter()?,
        s1.last_v_parameter()? - s1.first_v_parameter()?,
    );
    let mut dp: f64 = if dp2 < dp1 { dp2 } else { dp1 };
    let (mut dq1, dq2) = (
        s2.last_u_parameter()? - s2.first_u_parameter()?,
        s2.last_v_parameter()? - s2.first_v_parameter()?,
    );
    if dq2 > dq1 {
        dq1 = dq2;
    }
    if dq1 < dp {
        dp = dq1;
    }
    Ok(dp * 0.01)
}

/// `FTOL_FaceTolerances(B1, B2, F1, F2, S1, S2, myTol1, myTol2, Deflection, MaxUV)`:
/// both tolerances are the sum of the face tolerances, the deflection 1% of the
/// greatest box extent clamped to [1e-3, 0.1] and MaxUV 0.01 (the edge tolerances and
/// the UV ranges are computed but unused natively);
/// `ftol_face_tolerances(&b1, &b2, &f1, &f2, &s1, &s2)?`.
pub fn ftol_face_tolerances(
    b1: &BoundingBox3,
    b2: &BoundingBox3,
    f1: &Face,
    f2: &Face,
    s1: &BRepAdaptorSurface,
    s2: &BRepAdaptorSurface,
) -> Result<FaceTolerances, Failure> {
    let tol1: f64 = BRepTool::face_tolerance(f1)? + BRepTool::face_tolerance(f2)?;
    let dx = greatest_extent(b1, b2)?;
    let _tolef: f64 = edge_tolerance(f1)?.max(edge_tolerance(f2)?);
    let mut mdeflection: f64 = 0.01 * dx;
    let mut mmaxuv: f64 = 0.01;
    let _dp = uv_step(s1, s2)?;
    if mmaxuv < 1e-3 {
        mmaxuv = 1e-3;
    }
    if mdeflection < 1e-3 {
        mdeflection = 1e-3;
    }
    if mmaxuv > 0.01 {
        mmaxuv = 0.01;
    }
    if mdeflection > 0.1 {
        mdeflection = 0.1;
    }
    Ok(FaceTolerances {
        tol1,
        tol2: tol1,
        deflection: mdeflection,
        max_uv: mmaxuv,
    })
}

/// The box of a face in the global box tool, or the unit box.
fn global_box(f: &Face) -> Result<BoundingBox3, Failure> {
    let hbt = fbox_get_hbox_tool();
    if let Some(tool) = hbt.get()
        && tool.has_box(f.as_ref())
    {
        return tool.box_of_shape(f.as_ref());
    }
    let mut b = BoundingBox3::default();
    b.update_bounds([0.0; 3], [1.0; 3]);
    Ok(b)
}

/// `FTOL_FaceTolerances3d(F1, F2, Tol)`: the greatest tolerance of
/// `FTOL_FaceTolerances` on the boxes of the global box tool (unit boxes for the faces
/// without one), at most 1e-4; `let tol = ftol_face_tolerances3d(&f1, &f2)?;`.
pub fn ftol_face_tolerances3d(f1: &Face, f2: &Face) -> Result<f64, Failure> {
    let (b1, b2) = (global_box(f1)?, global_box(f2)?);
    let s1 = BRepAdaptorSurface::with_face(f1, true)?;
    let s2 = BRepAdaptorSurface::with_face(f2, true)?;
    let t = ftol_face_tolerances(&b1, &b2, f1, f2, &s1, &s2)?;
    let tol1: f64 = if t.tol1 > 1.0e-4 { 1.0e-4 } else { t.tol1 };
    let tol2: f64 = if t.tol2 > 1.0e-4 { 1.0e-4 } else { t.tol2 };
    Ok(tol1.max(tol2))
}

/// `FTOL_FaceTolerances3d(B1, B2, F1, F2, S1, S2, ...)`: `FTOL_FaceTolerances`;
/// `ftol_face_tolerances3d_on_boxes(&b1, &b2, &f1, &f2, &s1, &s2)?`.
pub fn ftol_face_tolerances3d_on_boxes(
    b1: &BoundingBox3,
    b2: &BoundingBox3,
    f1: &Face,
    f2: &Face,
    s1: &BRepAdaptorSurface,
    s2: &BRepAdaptorSurface,
) -> Result<FaceTolerances, Failure> {
    ftol_face_tolerances(b1, b2, f1, f2, s1, s2)
}

/// `FTOL_FaceTolerances2d(B1, B2, F1, F2, S1, S2, myTol1, myTol2)`: the tolerances of
/// `FTOL_FaceTolerances`; `let (t1, t2) = ftol_face_tolerances2d(&b1, &b2, &f1, &f2, &s1, &s2)?;`.
pub fn ftol_face_tolerances2d(
    b1: &BoundingBox3,
    b2: &BoundingBox3,
    f1: &Face,
    f2: &Face,
    s1: &BRepAdaptorSurface,
    s2: &BRepAdaptorSurface,
) -> Result<(f64, f64), Failure> {
    let t = ftol_face_tolerances(b1, b2, f1, f2, s1, s2)?;
    Ok((t.tol1, t.tol2))
}
