// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat.hxx/.cxx,
// BRepFeat_Status.hxx, BRepFeat_StatusError.hxx and BRepFeat_PerfSelection.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `BRepFeat` package functions (sample points and barycentres of shapes, their
//! extent along a curve, faces inside faces, extended limiting faces, the solid of a
//! tool) and the enumerations of the features.
use crate::errors::null_object;
use crate::loc_ope::{BuildShape, CSIntersector};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, Solid};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point3, Vector2, Xyz};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;
use std::io;

/// `NECHANTBARYC`: the number of sample intervals per edge of the barycentres.
const NECHANTBARYC: i32 = 11;

/// `BRepFeat_Status`: the status of the cylindrical hole.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Status {
    /// `BRepFeat_NoError`.
    #[default]
    NoError,
    /// `BRepFeat_InvalidPlacement`.
    InvalidPlacement,
    /// `BRepFeat_HoleTooLong`.
    HoleTooLong,
}

/// `BRepFeat_StatusError`: the error of a form feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum StatusError {
    /// `BRepFeat_OK`.
    #[default]
    Ok,
    /// `BRepFeat_BadDirect`.
    BadDirect,
    /// `BRepFeat_BadIntersect`.
    BadIntersect,
    /// `BRepFeat_EmptyBaryCurve`.
    EmptyBaryCurve,
    /// `BRepFeat_EmptyCutResult`.
    EmptyCutResult,
    /// `BRepFeat_FalseSide`.
    FalseSide,
    /// `BRepFeat_IncDirection`.
    IncDirection,
    /// `BRepFeat_IncSlidFace`.
    IncSlidFace,
    /// `BRepFeat_IncParameter`.
    IncParameter,
    /// `BRepFeat_IncTypes`.
    IncTypes,
    /// `BRepFeat_IntervalOverlap`.
    IntervalOverlap,
    /// `BRepFeat_InvFirstShape`.
    InvFirstShape,
    /// `BRepFeat_InvOption`.
    InvOption,
    /// `BRepFeat_InvShape`.
    InvShape,
    /// `BRepFeat_LocOpeNotDone`.
    LocOpeNotDone,
    /// `BRepFeat_LocOpeInvNotDone`.
    LocOpeInvNotDone,
    /// `BRepFeat_NoExtFace`.
    NoExtFace,
    /// `BRepFeat_NoFaceProf`.
    NoFaceProf,
    /// `BRepFeat_NoGluer`.
    NoGluer,
    /// `BRepFeat_NoIntersectF`.
    NoIntersectF,
    /// `BRepFeat_NoIntersectU`.
    NoIntersectU,
    /// `BRepFeat_NoParts`.
    NoParts,
    /// `BRepFeat_NoProjPt`.
    NoProjPt,
    /// `BRepFeat_NotInitialized`.
    NotInitialized,
    /// `BRepFeat_NotYetImplemented`.
    NotYetImplemented,
    /// `BRepFeat_NullRealTool`.
    NullRealTool,
    /// `BRepFeat_NullToolF`.
    NullToolF,
    /// `BRepFeat_NullToolU`.
    NullToolU,
}

/// `BRepFeat_PerfSelection`: the selection semantics of the Perform methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum PerfSelection {
    /// `BRepFeat_NoSelection`.
    #[default]
    NoSelection,
    /// `BRepFeat_SelectionFU`: from a face up to another.
    SelectionFU,
    /// `BRepFeat_SelectionU`: up to a face.
    SelectionU,
    /// `BRepFeat_SelectionSh`: through the shape.
    SelectionSh,
    /// `BRepFeat_SelectionShU`: up to the shape.
    SelectionShU,
}

/// `BRepFeat::SampleEdges(S, Pt)`: as `LocOpe::SampleEdges`;
/// `brep_feat::sample_edges(&shape, &mut points)?`.
pub fn sample_edges(s: &Shape, pt: &mut Sequence<Point3>) -> Result<(), Failure> {
    crate::loc_ope::sample_edges(s, pt)
}

/// The located 3D curve of an edge (always transformed, as natively) and its range.
fn transformed_curve(edg: &topods::Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let located = BRepTool::curve_with_location(edg)?;
    let curve = located
        .curve
        .get()
        .ok_or_else(|| null_object("BRepFeat", "edge without 3D curve"))?;
    let c: Handle<dyn GeomCurve> = curve.transformed(located.location.transformation())?.cast();
    Ok((c, located.first, located.last))
}

/// The 10 interior samples of each non-degenerated edge of `s` (each edge once), then
/// each vertex point; `visit` gets every point.
fn visit_samples(
    s: &Shape,
    mut visit: impl FnMut(Point3) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let mut the_map = MapOfShape::default();
    for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edg = topods::edge(&e)?;
        if !the_map.add(e) || BRepTool::degenerated(&edg)? {
            continue;
        }
        let (c, f, l) = transformed_curve(&edg)?;
        let curve = c
            .get()
            .ok_or_else(|| null_object("BRepFeat", "transformed geometry is not a curve"))?;
        for i in 1..NECHANTBARYC {
            let prm =
                (f64::from(NECHANTBARYC - i) * f + f64::from(i) * l) / f64::from(NECHANTBARYC);
            visit(curve.value(prm)?)?;
        }
    }
    // Adds every vertex
    for v in Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        if the_map.add(v.clone()) {
            visit(BRepTool::pnt(&topods::vertex(&v)?)?)?;
        }
    }
    Ok(())
}

/// `BRepFeat::Barycenter(S, B)`: the mean of the sample points of `s`;
/// `brep_feat::barycenter(&shape)?`.
pub fn barycenter(s: &Shape) -> Result<Point3, Failure> {
    let mut bar = Xyz::new(0., 0., 0.);
    let mut nbp = 0;
    visit_samples(s, |p| {
        bar += p.xyz();
        nbp += 1;
        Ok(())
    })?;
    bar /= f64::from(nbp);
    Ok(Point3::new(bar.x(), bar.y(), bar.z()))
}

/// The parameter of the nearest extremum of `extpc` (the first of the smallest squared
/// distances), when there is one.
fn nearest_parameter(extpc: &mut ExtPC<'_>, p: Point3) -> Result<Option<f64>, Failure> {
    extpc.perform(p)?;
    if !extpc.is_done() || extpc.nb_ext()? < 1 {
        return Ok(None);
    }
    let mut dist2_min = extpc.square_distance(1)?;
    let mut kmin = 1;
    for k in 2..=extpc.nb_ext()? {
        let dist2 = extpc.square_distance(k)?;
        if dist2 < dist2_min {
            dist2_min = dist2;
            kmin = k;
        }
    }
    Ok(Some(extpc.point(kmin)?.parameter()))
}

/// The adaptor of `cc` on its own range.
fn curve_adaptor(cc: &Handle<dyn GeomCurve>) -> Result<(GeomAdaptorCurve, f64, f64), Failure> {
    let curve = cc
        .get()
        .ok_or_else(|| null_object("BRepFeat", "null curve"))?;
    let (f, l) = (curve.first_parameter(), curve.last_parameter());
    Ok((GeomAdaptorCurve::with_curve(cc)?, f, l))
}

/// `BRepFeat::ParametricBarycenter(S, C)`: the mean parameter on `cc` of the
/// projections of the sample edge points of `s`. Native quirk kept: the vertices
/// projected count in the mean but add no parameter;
/// `brep_feat::parametric_barycenter(&shape, &curve)?`.
pub fn parametric_barycenter(s: &Shape, cc: &Handle<dyn GeomCurve>) -> Result<f64, Failure> {
    let (the_curve, f, l) = curve_adaptor(cc)?;
    let mut extpc = ExtPC::new();
    extpc.initialize(&the_curve, f, l, 1.0e-10)?;
    let mut parbar = 0.;
    let mut nbp = 0;
    let mut the_map = MapOfShape::default();
    for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edg = topods::edge(&e)?;
        if !the_map.add(e) || BRepTool::degenerated(&edg)? {
            continue;
        }
        let (c, f, l) = transformed_curve(&edg)?;
        let curve = c
            .get()
            .ok_or_else(|| null_object("BRepFeat", "transformed geometry is not a curve"))?;
        for i in 1..NECHANTBARYC {
            let prm =
                (f64::from(NECHANTBARYC - i) * f + f64::from(i) * l) / f64::from(NECHANTBARYC);
            // On projette sur CC
            if let Some(prmp) = nearest_parameter(&mut extpc, curve.value(prm)?)? {
                nbp += 1;
                parbar += prmp;
            }
        }
    }
    // Adds every vertex
    for v in Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        if the_map.add(v.clone())
            && nearest_parameter(&mut extpc, BRepTool::pnt(&topods::vertex(&v)?)?)?.is_some()
        {
            nbp += 1;
        }
    }
    Ok(parbar / f64::from(nbp))
}

/// The result of `BRepFeat::ParametricMinMax`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParametricMinMax {
    /// `prmin`: the first intersection parameter of the shape with the curve.
    pub prmin: f64,
    /// `prmax`: the last intersection parameter.
    pub prmax: f64,
    /// `prbmin`: the smallest parameter of the projected sample points.
    pub prbmin: f64,
    /// `prbmax`: the largest parameter of the projected sample points.
    pub prbmax: f64,
    /// `flag`: the curve meets the shape.
    pub flag: bool,
}

/// `BRepFeat::ParametricMinMax(S, C, prmin, prmax, prbmin, prbmax, flag, Ori)`: the
/// extreme intersection parameters of `cc` with `s` (ordered by the first transition
/// when `ori`) and the extreme parameters of the projections of the sample points of
/// `s`; `brep_feat::parametric_min_max(&shape, &curve, false)?`.
pub fn parametric_min_max(
    s: &Shape,
    cc: &Handle<dyn GeomCurve>,
    ori: bool,
) -> Result<ParametricMinMax, Failure> {
    let mut asi = CSIntersector::with_shape(s);
    let mut scur = Sequence::default();
    scur.append(cc.clone());
    asi.perform_curves(&scur)?;
    let mut r = ParametricMinMax {
        prmin: f64::MAX,
        prmax: -f64::MAX,
        prbmin: f64::MAX,
        prbmax: -f64::MAX,
        flag: false,
    };
    if asi.is_done() && asi.nb_points(1)? >= 1 {
        let n = asi.nb_points(1)?;
        let (first, last) = (asi.point(1, 1)?.parameter(), asi.point(1, n)?.parameter());
        if !ori {
            (r.prmin, r.prmax) = (first.min(last), first.max(last));
        } else if asi.point(1, 1)?.orientation() == Orientation::Forward {
            (r.prmin, r.prmax) = (first, last);
        } else {
            (r.prmax, r.prmin) = (first, last);
        }
        r.flag = true;
    }
    let (the_curve, f, l) = curve_adaptor(cc)?;
    let mut extpc = ExtPC::new();
    extpc.initialize(&the_curve, f, l, 1.0e-10)?;
    visit_samples(s, |p| {
        // On projette sur CC
        if let Some(prmp) = nearest_parameter(&mut extpc, p)? {
            if prmp <= r.prbmin {
                r.prbmin = prmp;
            }
            if prmp >= r.prbmax {
                r.prbmax = prmp;
            }
        }
        Ok(())
    })?;
    Ok(r)
}

/// The static `IsIn(FC, AC)`: no point of a quasi-uniform deflection sampling of the
/// curve (100 Confusion) is OUT of the face.
fn is_in(fc: &BRepTopAdaptorFClass2d, ac: &Geom2dAdaptorCurve) -> Result<bool, Failure> {
    let def = 100. * precision::confusion();
    let qu = QuasiUniformDeflection::with_curve(ac as &dyn Adaptor2dCurve2d, def, Continuity::C1)?;
    for i in 1..=qu.nb_points()? {
        let p = ac.value(qu.parameter(i)?)?;
        if fc.perform(p, false)? == State::Out {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `PutInBoundsU` / `PutInBoundsV`: shifts the p-curve by periods along the direction
/// `along_u` so that it starts in `[min, max]`, then by one period back when it then
/// overflows more than the underflow it would make.
fn put_in_bounds(
    bounds: (f64, f64),
    eps: f64,
    period: f64,
    (f, l): (f64, f64),
    c2d: &Handle<dyn Geom2dCurve>,
    along_u: bool,
) -> Result<(), Failure> {
    let curve = c2d
        .get()
        .ok_or_else(|| null_object("BRepFeat::IsInside", "null p-curve"))?;
    let coord = |u: f64| -> Result<f64, Failure> {
        let p = curve.value(u)?;
        Ok(if along_u { p.x() } else { p.y() })
    };
    let shift = |d: f64| {
        if along_u {
            Vector2::new(d, 0.)
        } else {
            Vector2::new(0., d)
        }
    };
    let (cf, cl, cm) = (coord(f)?, coord(l)?, coord(0.34 * f + 0.66 * l)?);
    let (min, max) = bounds;
    let mut min_c = cf.min(cl).min(cm);
    let mut max_c = cf.max(cl).max(cm);
    let mut du = 0.;
    if min_c < min - eps {
        du = f64::from(((min - min_c) / period) as i32 + 1) * period;
    }
    if min_c > max + eps {
        du = -f64::from(((min_c - max) / period) as i32 + 1) * period;
    }
    if du != 0. {
        curve.translate(shift(du))?;
        min_c += du;
        max_c += du;
    }
    // Ajuste au mieux la courbe dans le domaine.
    if max_c > max + 100. * eps {
        let d1 = max_c - max;
        let d2 = min - min_c + period;
        if d2 < d1 {
            du = -period;
        }
        if du != 0. {
            curve.translate(shift(du))?;
        }
    }
    Ok(())
}

/// `BRepFeat::IsInside(F1, F2)`: every edge of `f1`, projected on the surface of `f2`
/// (recalibrated into its periods), stays inside `f2`;
/// `brep_feat::is_inside(&f1, &f2)?`.
pub fn is_inside(f1: &Face, f2: &Face) -> Result<bool, Failure> {
    let s = BRepTool::surface(f2)?;
    let (umin, umax, vmin, vmax) = brep_tools::uv_bounds(f2)?;
    let surface = s
        .get()
        .ok_or_else(|| null_object("BRepFeat::IsInside", "face without surface"))?;
    let uperiod = if surface.is_u_periodic() {
        Some(surface.u_period()?)
    } else {
        None
    };
    let vperiod = if surface.is_v_periodic() {
        Some(surface.v_period()?)
    } else {
        None
    };
    let fc = BRepTopAdaptorFClass2d::new(
        &topods::face(&f2.oriented(Orientation::Forward))?,
        precision::confusion(),
    )?;
    for e in Explorer::new(f1, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edg = topods::edge(&e)?;
        let (c0, mut f1p, mut l1p) = BRepTool::curve(&edg)?;
        let c = geom_proj_lib::curve2d_range(&c0, (f1p, l1p), &s)?;
        if uperiod.is_some() || vperiod.is_some() {
            let eps = BRepTool::edge_tolerance(&edg)?;
            (f1p, l1p) = BRepTool::range(&edg)?;
            if let Some(period) = uperiod {
                put_in_bounds((umin, umax), eps, period, (f1p, l1p), &c, true)?;
            }
            if let Some(period) = vperiod {
                put_in_bounds((vmin, vmax), eps, period, (f1p, l1p), &c, false)?;
            }
        }
        let ac = Geom2dAdaptorCurve::with_range(&c, f1p, l1p)?;
        if !is_in(&fc, &ac)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The extreme `(u, v)` parameters of the corners of `bounds` on a surface.
fn corner_parameters(
    bounds: &BoundingBox3,
    parameters: impl Fn(Point3) -> (f64, f64),
) -> Result<(f64, f64, f64, f64), Failure> {
    let [xmin, ymin, zmin, xmax, ymax, zmax] = bounds.get()?;
    let (x, y, z) = ([xmin, xmax], [ymin, ymax], [zmin, zmax]);
    let (mut umin, mut umax, mut vmin, mut vmax) = (f64::MAX, -f64::MAX, f64::MAX, -f64::MAX);
    for xi in x {
        for yj in y {
            for zk in z {
                let (u, v) = parameters(Point3::new(xi, yj, zk));
                if u < umin {
                    umin = u;
                }
                if u > umax {
                    umax = u;
                }
                if v < vmin {
                    vmin = v;
                }
                if v > vmax {
                    vmax = v;
                }
            }
        }
    }
    Ok((umin, umax, vmin, vmax))
}

/// `BRepFeat::FaceUntil(Sbase, FUntil)`: the face `f_until` extended on its plane,
/// cylinder or cone beyond the box of `sbase` (by ten box diagonals); null for other
/// surfaces; `brep_feat::face_until(&base, &mut face)?`.
pub fn face_until(sbase: &Shape, f_until: &mut Face) -> Result<(), Failure> {
    let mut b = BoundingBox3::default();
    BRepBndLib::add(sbase, &mut b, true)?;
    let diam = 10. * b.square_extent().sqrt();
    let mut s = BRepTool::surface(f_until)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    if let Some(t) = trimmed.get() {
        s = t.basis_surface();
    }
    let str = if let Some(plane) = s.cast::<GeomPlane>().get() {
        let pln = plane.pln();
        let (umin, umax, vmin, vmax) = corner_parameters(&b, |p| elslib::parameters(&pln, p))?;
        GeomRectangularTrimmedSurface::new(
            &s,
            (umin - diam, umax + diam),
            (vmin - diam, vmax + diam),
            (true, true),
        )?
    } else if let Some(cyl) = s.cast::<GeomCylindricalSurface>().get() {
        let cylinder = cyl.cylinder()?;
        let (_, _, vmin, vmax) = corner_parameters(&b, |p| elslib::parameters(&cylinder, p))?;
        GeomRectangularTrimmedSurface::with_one_direction(
            &s,
            vmin - diam,
            vmax + diam,
            false,
            true,
        )?
    } else if let Some(con) = s.cast::<GeomConicalSurface>().get() {
        let cone = con.cone()?;
        let (_, _, vmin, vmax) = corner_parameters(&b, |p| elslib::parameters(&cone, p))?;
        GeomRectangularTrimmedSurface::with_one_direction(
            &s,
            vmin - diam,
            vmax + diam,
            false,
            true,
        )?
    } else {
        f_until.nullify();
        return Ok(());
    };
    let str: Handle<dyn GeomSurface> = Handle::new(str).cast();
    *f_until = BRepLibMakeFace::from_surface(&str, precision::confusion())?.face()?;
    Ok(())
}

/// `BRepFeat::Tool(SRef, Fac, Orf)`: the solid bounded by the single shell of the
/// faces of `sref`, oriented so that `fac` has the orientation `orf` against it (null
/// when the faces do not make a single shell);
/// `brep_feat::tool(&shape, &face, Orientation::Forward)?`.
pub fn tool(sref: &Shape, fac: &Face, orf: Orientation) -> Result<Solid, Failure> {
    let lfaces = Explorer::new(sref, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
    let res = BuildShape::with_faces(&lfaces)?.shape().clone();
    let mut sh = match res.shape_type()? {
        // faire un solide
        ShapeEnum::Shell => res.clone(),
        ShapeEnum::Solid => {
            let mut shells = Explorer::new(&res, ShapeEnum::Shell, ShapeEnum::Shape)?;
            let first = Iterator::next(&mut shells).unwrap_or_default();
            if Iterator::next(&mut shells).is_some() {
                Shape::default()
            } else {
                first
            }
        }
        _ => Shape::default(),
    };
    if sh.is_null() {
        return Ok(Solid::default());
    }
    sh.set_orientation(Orientation::Forward);
    let orient = Explorer::new(&sh, ShapeEnum::Face, ShapeEnum::Shape)?
        .find(|x| x.is_same(fac))
        .map_or(Orientation::Forward, |x| x.orientation());
    let reverse = (orient == fac.orientation() && orf == Orientation::Reversed)
        || (orient != fac.orientation() && orf == Orientation::Forward);
    if reverse {
        sh.reverse();
    }
    let b = BRepBuilder::default();
    let mut soc = Solid::default();
    b.make_solid(&mut soc);
    b.add(&mut soc, &sh)?;
    Ok(soc)
}

/// The message of a status error, as `BRepFeat::Print` writes it.
pub fn status_error_message(se: StatusError) -> &'static str {
    match se {
        StatusError::Ok => "No error",
        StatusError::BadDirect => "Directions must be opposite",
        StatusError::BadIntersect => "Intersection failure",
        StatusError::EmptyBaryCurve => "Empty BaryCurve",
        StatusError::EmptyCutResult => "Failure in Cut : Empty resulting shape",
        StatusError::FalseSide => "Verify plane and wire orientation",
        StatusError::IncDirection => "Incoherent Direction for shapes From and Until",
        StatusError::IncSlidFace => "Sliding face not in Base shape",
        StatusError::IncParameter => "Incoherent Parameter : shape Until before shape From",
        StatusError::IncTypes => "Invalid option for faces From and Until : 1 Support and 1 not",
        StatusError::IntervalOverlap => "Shapes From and Until overlap",
        StatusError::InvFirstShape => "Invalid First shape : more than 1 face",
        StatusError::InvOption => "Invalid option",
        StatusError::InvShape => "Invalid shape",
        StatusError::LocOpeNotDone => "Local Operation not done",
        StatusError::LocOpeInvNotDone => "Local Operation : intersection line conflict",
        StatusError::NoExtFace => "No Extreme faces",
        StatusError::NoFaceProf => "No Face Profile",
        StatusError::NoGluer => "Gluer Failure",
        StatusError::NoIntersectF => "No intersection between Feature and shape From",
        StatusError::NoIntersectU => "No intersection between Feature and shape Until",
        StatusError::NoParts => "No parts of tool kept",
        StatusError::NoProjPt => "No projection points",
        StatusError::NotInitialized => "Fields not initialized",
        StatusError::NotYetImplemented => "Not yet implemented",
        StatusError::NullRealTool => "Real Tool : Null DPrism",
        StatusError::NullToolF => "Null Tool : Invalid type for shape Form",
        StatusError::NullToolU => "Null Tool : Invalid type for shape Until",
    }
}

/// `BRepFeat::Print(SE, S)`: writes the message of the error;
/// `brep_feat::print(StatusError::Ok, &mut out)?`.
pub fn print(se: StatusError, s: &mut impl io::Write) -> io::Result<()> {
    s.write_all(status_error_message(se).as_bytes())
}
