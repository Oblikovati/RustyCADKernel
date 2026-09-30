//! GeomAPI_PointsToBSplineSurface probe operation, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geomapi_surface.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, real, show};
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface};
use rustycad_tkgeomalgo::geom_api::PointsToBSplineSurface;
use rustycad_tkgeombase::approx::ParametrizationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;

/// `ga_gapi_bsurface_out`: degrees, periodicity, rationality, poles, knots, mults.
pub fn bsurface_out(s: &Handle<GeomBSplineSurface>) -> Result<(), Failure> {
    flag(s.is_null());
    let Some(s) = s.get() else { return Ok(()) };
    int(s.u_degree());
    int(s.v_degree());
    flag(s.is_u_periodic());
    flag(s.is_v_periodic());
    let rat = s.is_u_rational() || s.is_v_rational();
    flag(rat);
    int(s.nb_u_poles());
    int(s.nb_v_poles());
    for i in 1..=s.nb_u_poles() {
        for j in 1..=s.nb_v_poles() {
            pnt(s.pole(i, j)?);
            if rat {
                real(s.weight(i, j)?);
            }
        }
    }
    int(s.nb_u_knots());
    for i in 1..=s.nb_u_knots() {
        real(s.u_knot(i)?);
        int(s.u_multiplicity(i)?);
    }
    int(s.nb_v_knots());
    for i in 1..=s.nb_v_knots() {
        real(s.v_knot(i)?);
        int(s.v_multiplicity(i)?);
    }
    Ok(())
}

fn shape(s: i32) -> Shape {
    [
        Shape::C0,
        Shape::G1,
        Shape::C1,
        Shape::G2,
        Shape::C2,
        Shape::C3,
        Shape::CN,
    ][s as usize]
}

fn par_type(p: i32) -> ParametrizationType {
    [
        ParametrizationType::ChordLength,
        ParametrizationType::Centripetal,
        ParametrizationType::IsoParametric,
    ][p as usize]
}

type Grids = (Array2<'static, Point3>, Array2<'static, f64>);

// The Init, Interpolate or constructor of `mode`; a failed constructor keeps the
// previous object, as the native `reset` of a throwing `new`.
fn run_mode(
    r: &mut Reader<'_>,
    f: &mut PointsToBSplineSurface,
    mode: i32,
    (pts, zs): &Grids,
) -> Result<(), Failure> {
    match mode {
        0 | 7 => {
            let settings = (r.integer(), r.integer(), shape(r.integer()), r.real());
            if mode == 0 {
                return f.init(pts, settings);
            }
            *f = PointsToBSplineSurface::with_points(pts, settings)?;
        }
        1 | 8 => {
            let pt = par_type(r.integer());
            let settings = (r.integer(), r.integer(), shape(r.integer()), r.real());
            let per = mode == 1 && r.flag();
            if mode == 1 {
                return f.init_parametrization(pts, pt, settings, per);
            }
            *f = PointsToBSplineSurface::with_parametrization(pts, pt, settings)?;
        }
        2 | 9 => {
            let w = (r.real(), r.real(), r.real());
            let (b, s, t) = (r.integer(), shape(r.integer()), r.real());
            if mode == 2 {
                return f.init_weights(pts, w, b, s, t);
            }
            *f = PointsToBSplineSurface::with_weights(pts, w, b, s, t)?;
        }
        3 | 6 | 10 => heights_mode(r, f, mode, zs)?,
        4 => return f.interpolate(pts, r.flag()),
        _ => {
            let pt = par_type(r.integer());
            return f.interpolate_parametrization(pts, pt, r.flag());
        }
    }
    Ok(())
}

fn heights_mode(
    r: &mut Reader<'_>,
    f: &mut PointsToBSplineSurface,
    mode: i32,
    zs: &Array2<'static, f64>,
) -> Result<(), Failure> {
    let grid = (r.real(), r.real(), r.real(), r.real());
    if mode == 6 {
        return f.interpolate_heights(zs, grid);
    }
    let settings = (r.integer(), r.integer(), shape(r.integer()), r.real());
    if mode == 3 {
        return f.init_heights(zs, grid, settings);
    }
    *f = PointsToBSplineSurface::with_heights(zs, grid, settings)?;
    Ok(())
}

/// `mode rl ru cl cu values args`: GeomAPI_PointsToBSplineSurface.
pub fn p2bss_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let (rl, ru, cl, cu) = (r.integer(), r.integer(), r.integer(), r.integer());
    let heights = matches!(mode, 3 | 6 | 10);
    let mut pts: Array2<'static, Point3> = Array2::with_bounds(rl, ru, cl, cu).unwrap();
    let mut zs: Array2<'static, f64> = Array2::with_bounds(rl, ru, cl, cu).unwrap();
    for i in rl..=ru {
        for j in cl..=cu {
            if heights {
                zs[(i, j)] = r.real();
            } else {
                pts[(i, j)] = r.pnt();
            }
        }
    }
    let grids: Grids = (pts, zs);
    let mut f = PointsToBSplineSurface::new();
    attempt(|| run_mode(&mut r, &mut f, mode, &grids));
    flag(f.is_done());
    attempt(|| bsurface_out(&f.surface()?));
    show(f.as_surface(), |s| flag(s.is_null()));
    end();
}
