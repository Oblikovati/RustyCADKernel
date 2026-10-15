// Rust translation of the file statics CalculDroite and CalculBatten
// (ChFi3d_Builder_CnCrn.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pcurve of a connection on a face: a straight segment, or a batten tangent to the
//! curves it joins.
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::fair_curve::{FairCurveAnalysisCode, FairCurveBatten};
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkgeombase::geom_lprop::CLProps2d;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction2, Point2};
use std::f64::consts::PI;

/// `CalculDroite(p2d1, xdir, ydir, pcurve)`: the segment from `p2d1` along (xdir, ydir),
/// of its length.
pub(super) fn calcul_droite(
    p2d1: Point2,
    xdir: f64,
    ydir: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let dir1 = Direction2::new(xdir, ydir)?;
    let l: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(p2d1, dir1)).cast();
    let l0 = (xdir * xdir + ydir * ydir).sqrt();
    Ok(Handle::new(Geom2dTrimmedCurve::new(&l, 0.0, l0, true, true)?).cast())
}

/// The ends of a batten: the two points, the curves it leaves and their parameters, and
/// whether its tangents are imposed and reversed at them.
pub(super) struct BattenEnds<'a> {
    pub p2d: [Point2; 2],
    pub contraint: [bool; 2],
    pub curv2d: [&'a Handle<dyn Geom2dCurve>; 2],
    pub par: [f64; 2],
    pub inverse: [bool; 2],
}

/// The tangent of the curve at `u`, reversed when asked.
fn tangent(curve: &Handle<dyn Geom2dCurve>, u: f64, inverse: bool) -> Result<Direction2, Failure> {
    let mut props = CLProps2d::with_parameter(curve.clone(), u, 1, 1.0e-4)?;
    let d = props.tangent()?;
    Ok(if inverse { d.reversed() } else { d })
}

/// The angles of the batten at its ends from the chord.
fn end_angles(dir1: Direction2, ends: &BattenEnds<'_>) -> Result<(f64, f64), Failure> {
    let dir3 = tangent(ends.curv2d[0], ends.par[0], ends.inverse[0])?;
    let dir4 = tangent(ends.curv2d[1], ends.par[1], ends.inverse[1])?;
    let ang1 = dir1.angle(dir3);
    let a4 = dir1.angle(dir4);
    let ang2 = if a4 > 0.0 { PI - a4 } else { -PI - a4 };
    Ok((ang1, ang2))
}

/// `CalculBatten(ASurf, Face, xdir, ydir, p2d1, p2d2, contraint1, contraint2, curv2d1,
/// curv2d2, picicplus, picplusic, inverseic, inverseicplus, pcurve)`: the batten between
/// the two points with the imposed angles, else the segment (big angles on a surface that
/// is not a plane, no batten, or one leaving the bounds of the face).
pub(super) fn calcul_batten(
    asurf: &Handle<GeomAdaptorSurface>,
    face: &Face,
    (xdir, ydir): (f64, f64),
    ends: &BattenEnds<'_>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let mut isplane = asurf.get_type()? == SurfaceType::Plane;
    let dir1 = Direction2::new(xdir, ydir)?;
    let (ang1, ang2) = end_angles(dir1, ends)?;
    let h = ends.p2d[1].distance(ends.p2d[0]) / 20.0;
    let mut bat = FairCurveBatten::new(ends.p2d[0], ends.p2d[1], h, 0.0)?;
    bat.set_free_sliding(true);
    let anglebig = match ends.contraint {
        [true, true] => ang1.abs() > 1.2 || ang2.abs() > 1.2,
        [true, false] => ang1.abs() > 1.2,
        [false, true] => ang2.abs() > 1.2,
        [false, false] => false,
    };
    if isplane && (ang1.abs() > PI / 2.0 || ang2.abs() > PI / 2.0) {
        isplane = false;
    }
    if anglebig && !isplane {
        return calcul_droite(ends.p2d[0], xdir, ydir);
    }
    if ends.contraint[0] {
        bat.set_angle1(ang1)
    } else {
        bat.set_constraint_order1(0)
    }
    if ends.contraint[1] {
        bat.set_angle2(ang2)
    } else {
        bat.set_constraint_order2(0)
    }
    let mut iana = FairCurveAnalysisCode::default();
    if bat.compute(&mut iana, 25, 1.0e-2)? {
        let pcurve: Handle<dyn Geom2dCurve> = bat.curve()?.cast();
        if inside_face(&pcurve, face)? {
            return Ok(pcurve);
        }
    }
    calcul_droite(ends.p2d[0], xdir, ydir)
}

/// The box of the curve is within the parametric bounds of the face (to 1e-7).
fn inside_face(pcurve: &Handle<dyn Geom2dCurve>, face: &Face) -> Result<bool, Failure> {
    let (umin, umax, vmin, vmax) = brep_tools::uv_bounds(face)?;
    let mut bc = BoundingBox2::default();
    Add2dCurve::add_adaptor(&Geom2dAdaptorCurve::with_curve(pcurve)?, 0.0, &mut bc)?;
    let [uminc, vminc, umaxc, vmaxc] = bc.get()?;
    Ok(uminc >= umin - 1.0e-7
        && umaxc <= umax + 1.0e-7
        && vminc >= vmin - 1.0e-7
        && vmaxc <= vmax + 1.0e-7)
}
