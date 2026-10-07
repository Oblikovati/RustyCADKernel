// Rust translation of ChFi3d_PerformElSpine with CurveCleaner and GoodExt
// (ChFi3d_Builder_0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The guide curve of an elementary spine: the edges of the spine on its range
//! concatenated into one B-spline, extended at the open ends, then smoothed to C2 or
//! C3. The extensions and the final smoothing are in elspine_ends.rs.
use super::elspine_ends::{curve_cleaner, extend_and_smooth};
use super::smooth::{approx_by_c2, is_smooth};
use crate::chfi_ds::{ElSpine, Spine, SpineHandle, lock};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBoundedCurve, GeomCurve, GeomLine, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_convert::{self, CompCurveToBSplineCurve};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3};
use rustycad_tktopalgo::brep_lib;

/// The range of the elementary spine on the spine: its parameters, their references on
/// the edges, the edges `IF..IL` it covers, and the points and unit tangents at its ends.
pub(super) struct ElSpineRange {
    pub(super) wf: f64,
    pub(super) wl: f64,
    pub(super) wrefdeb: f64,
    pub(super) wreffin: f64,
    nwf: f64,
    nwl: f64,
    nbed: i32,
    periodic: bool,
    first_index: i32,
    last_index: i32,
    pub(super) pdeb: Point3,
    pub(super) vrefdeb: Vector3,
    pub(super) pfin: Point3,
    pub(super) vreffin: Vector3,
}

/// `std::max` and `std::min` of the native code.
fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}
fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

impl ElSpineRange {
    fn new(es: &ElSpine, spine: &mut Spine, tol: f64) -> Result<Self, Failure> {
        let (wf, wl) = (es.first_parameter()?, es.last_parameter()?);
        let (nbed, periodic) = (spine.nb_edges(), spine.is_periodic());
        let mut range = Self {
            wf,
            wl,
            wrefdeb: wf,
            wreffin: wl,
            nwf: wf,
            nwl: wl,
            nbed,
            periodic,
            first_index: 0,
            last_index: 0,
            pdeb: Point3::default(),
            vrefdeb: Vector3::default(),
            pfin: Point3::default(),
            vreffin: Vector3::default(),
        };
        if periodic {
            let period = spine.period()?;
            range.nwf = elclib::in_period(wf, -tol, period - tol);
            range.first_index = spine.index_at(range.nwf, true)?;
            range.nwl = elclib::in_period(wl, tol, period + tol);
            range.last_index = spine.index_at(range.nwl, false)?;
            if range.nwl < range.nwf + tol {
                range.last_index += nbed;
            }
        } else {
            range.first_index = spine.index_at(wf, true)?;
            range.last_index = spine.index_at(wl, false)?;
            range.wrefdeb = std_max(spine.first_parameter_of(range.first_index)?, wf);
            range.wreffin = std_min(spine.last_parameter_of(range.last_index)?, wl);
        }
        (range.pdeb, range.vrefdeb) = spine.d1(wf)?;
        (range.pfin, range.vreffin) = spine.d1(wl)?;
        range.vrefdeb.normalize()?;
        range.vreffin.normalize()?;
        Ok(range)
    }
}

/// The edge `index` of the spine (its offset edge when asked).
fn spine_edge(spine: &Spine, index: i32, is_offset: bool) -> Result<Edge, Failure> {
    Ok(if is_offset {
        spine.offset_edges(index)?.clone()
    } else {
        spine.edges(index)?.clone()
    })
}

/// The point and tangent at the start of the edge (as oriented) added to the
/// elementary spine.
fn add_vertex_tangent(
    es: &ElSpine,
    e: &Edge,
    cv: &Handle<dyn GeomCurve>,
    (first, last): (f64, f64),
) -> Result<(), Failure> {
    let par = if e.orientation() == Orientation::Forward {
        first
    } else {
        last
    };
    let (p, d) = cv.d1(par)?;
    es.add_vertex_with_tangent(Axis1::new(p, Direction3::from_vector(d)?));
    Ok(())
}

/// The parameter on the edge `index` of the end abscissa `w` when it falls inside the
/// edge (`inside`), else `None`.
fn pared_at(spine: &mut Spine, index: i32, w: f64, inside: bool) -> Result<Option<f64>, Failure> {
    if !inside {
        return Ok(None);
    }
    Ok(Some(spine.parameter_on_edge(index, w, false)?))
}

/// The curve of the first edge restricted to the range of the elementary spine (both
/// ends when it is also the last edge), oriented as the edge.
fn first_edge_range(
    range: &ElSpineRange,
    spine: &mut Spine,
    e: &Edge,
    (cv, first, last): (Handle<dyn GeomCurve>, f64, f64),
) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let (mut cv, mut first, mut last) = (cv, first, last);
    let urefdeb = spine.first_parameter_of(range.first_index)?;
    let pared_deb = pared_at(spine, range.first_index, range.nwf, range.nwf > urefdeb)?;
    let same_edge = range.last_index == range.first_index;
    let ureffin = if same_edge {
        spine.last_parameter_of(range.last_index)?
    } else {
        0.0
    };
    let reversed = e.orientation() == Orientation::Reversed;
    if reversed {
        let sov = first;
        first = cv.reversed_parameter(last)?;
        last = cv.reversed_parameter(sov)?;
    }
    let pared = match pared_deb {
        Some(p) if reversed => cv.reversed_parameter(p)?,
        Some(p) => p,
        None => first,
    };
    if first < pared {
        first = pared;
    }
    if same_edge {
        let pared_fin = pared_at(spine, range.last_index, range.nwl, range.nwl < ureffin)?;
        let pared = match pared_fin {
            Some(p) if reversed => cv.reversed_parameter(p)?,
            Some(p) => p,
            None => last,
        };
        if pared < last {
            last = pared;
        }
    }
    if reversed {
        cv = cv.reversed()?;
    }
    Ok((cv, first, last))
}

/// The straight elementary spine of an edge eaten by a particular case: along the
/// tangent at the end of the spine it lies beyond.
fn straight_elspine(
    es: &ElSpine,
    spine: &Spine,
    range: &ElSpineRange,
    tol: f64,
) -> Result<(), Failure> {
    let (w, p, tangent) = if range.wl < spine.first_parameter_of(1)? + tol {
        let (_, vtemp) = es.last_point_and_tgt();
        (range.wl, range.pfin, vtemp)
    } else if range.wf > spine.last_parameter_of(range.nbed)? - tol {
        let (_, vtemp) = es.first_point_and_tgt();
        (range.wf, range.pdeb, vtemp)
    } else {
        return Ok(());
    };
    let d = Direction3::from_vector(tangent)?;
    let olin = Point3::from(d.xyz() * -w + p.xyz());
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(olin, d)).cast();
    es.set_curve(&line)
}

/// The trimmed piece of an edge as a curve to concatenate: kept, or replaced by its C2
/// approximation when it is neither C2 nor smooth (counted).
fn smoothed_piece(
    cv: &Handle<dyn GeomCurve>,
    (first, last): (f64, f64),
    wspan: f64,
    approx_count: &mut i32,
) -> Result<Handle<dyn GeomBoundedCurve>, Failure> {
    let tc: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(cv, first, last, true, true)?).cast();
    let bs = geom_convert::curve_to_bspline_curve(&tc, ParameterisationType::TgtThetaOver2)?;
    curve_cleaner(&bs, wspan * 1.0e-4, 0)?;
    let continuity = tc.continuity();
    // IsSmooth is evaluated whatever the continuity, as natively.
    let smooth = is_smooth(&tc)?;
    if (continuity as i32) < (Continuity::C2 as i32) && !smooth {
        *approx_count += 1;
        return Ok(approx_by_c2(&tc)?.cast());
    }
    Ok(tc.cast())
}

/// The next edge `iedge` of the path concatenated: its range ends at the elementary
/// spine on the last edge; the tolerance of the junction grows when the addition fails.
fn concatenate_edge(
    (es, spine, range): (&ElSpine, &mut Spine, &ElSpineRange),
    (iedge, eold): (i32, &Edge),
    (tol, is_offset): (f64, bool),
    concat: &mut CompCurveToBSplineCurve,
    approx_count: &mut i32,
) -> Result<Option<Edge>, Failure> {
    let iloc = if range.periodic {
        (iedge - 1) % range.nbed + 1
    } else {
        iedge
    };
    let e = spine_edge(spine, iloc, is_offset)?;
    if BRepTool::degenerated(&e)? {
        return Ok(None);
    }
    let eps_v = match topexp::common_vertex(eold, &e) {
        Some(v) => BRepTool::vertex_tolerance(&v)?,
        None => tol,
    };
    if !brep_lib::build_curve3d(&e, 1.0e-5, Continuity::C1, 14, 0)? {
        return Err(Failure::with_message(
            "Standard_ConstructionError",
            "PerformElSpine : BuildCurve3d error",
        ));
    }
    let (mut cv, mut first, mut last) = BRepTool::curve(&e)?;
    add_vertex_tangent(es, &e, &cv, (first, last))?;
    if iedge == range.last_index {
        let ureffin = spine.last_parameter_of(iloc)?;
        let pared_fin = pared_at(spine, iloc, range.nwl, range.nwl < ureffin)?;
        let mut pared = pared_fin.unwrap_or(last);
        if e.orientation() == Orientation::Reversed {
            let sov = first;
            first = cv.reversed_parameter(last)?;
            last = cv.reversed_parameter(sov)?;
            pared = match pared_fin {
                Some(p) => cv.reversed_parameter(p)?,
                None => last,
            };
            cv = cv.reversed()?;
        }
        if pared < last {
            last = pared;
        }
    }
    let tc = smoothed_piece(
        &cv,
        (first, last),
        (range.wl - range.wf).abs(),
        approx_count,
    )?;
    let tolrac = std_min(tol, eps_v);
    let added = concat.add(&tc, 2.0 * tolrac, true, true, 0)?
        || concat.add(&tc, 2.0 * eps_v, true, true, 0)?
        || concat.add(&tc, 200.0 * eps_v, true, true, 0)?;
    if !added {
        return Err(Failure::with_message(
            "Standard_ConstructionError",
            "PerformElSpine: spine merged error",
        ));
    }
    Ok(Some(e))
}

/// `ChFi3d_PerformElSpine(HES, Spine, continuity, tol, IsOffset)`: the curve of the
/// elementary spine: the edges of the spine (or their offsets) on its range concatenated
/// and reparametrized as the spine, extended beyond the spine to open ends and smoothed
/// to `continuity` (C3, else C2); a straight line along the end tangent when the first
/// edge reduces to nothing; `perform_el_spine(&es, &spine, Continuity::C2, 1e-4, false)?`.
pub fn perform_el_spine(
    hes: &Handle<ElSpine>,
    spine: &SpineHandle,
    continuity: Continuity,
    tol: f64,
    is_offset: bool,
) -> Result<(), Failure> {
    let es = hes.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFi3d_PerformElSpine: null elementary spine",
        )
    })?;
    let mut guard = lock(spine);
    let spine = guard.spine_mut();
    let range = ElSpineRange::new(es, spine, tol)?;
    let e = spine_edge(spine, range.first_index, is_offset)?;
    brep_lib::build_curve3d(&e, 1.0e-5, Continuity::C1, 14, 0)?;
    let tolpared = spine
        .current_elementary_spine(range.first_index)?
        .resolution(tol)?;
    let (cv, first, last) = BRepTool::curve(&e)?;
    if es.is_periodic()? {
        add_vertex_tangent(es, &e, &cv, (first, last))?;
    }
    let (cv, first, last) = first_edge_range(&range, spine, &e, (cv, first, last))?;
    if (last - first).abs() < tolpared {
        return straight_elspine(es, spine, &range, tol);
    }
    let mut approx_count = 0;
    let tc = smoothed_piece(
        &cv,
        (first, last),
        (range.wl - range.wf).abs(),
        &mut approx_count,
    )?;
    let mut concat = CompCurveToBSplineCurve::with_curve(&tc, ParameterisationType::QuasiAngular)?;
    let mut eold = e;
    for iedge in (range.first_index + 1)..=range.last_index {
        if let Some(e) = concatenate_edge(
            (es, spine, &range),
            (iedge, &eold),
            (tol, is_offset),
            &mut concat,
            &mut approx_count,
        )? {
            eold = e;
        }
    }
    extend_and_smooth(
        es,
        concat.bspline_curve(),
        range,
        (continuity, tol),
        approx_count,
    )
}
