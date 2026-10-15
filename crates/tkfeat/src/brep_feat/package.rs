// Rust translation of ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat.hxx/.cxx,
// BRepFeat_Status.hxx, BRepFeat_StatusError.hxx and BRepFeat_PerfSelection.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `BRepFeat` package functions (sample points and barycentres of shapes, their
//! extent along a curve, faces inside faces, extended limiting faces, the solid of a
//! tool) and the enumerations of the features.
use crate::errors::null_object;
use crate::loc_ope::CSIntersector;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::gp::{Point3, Xyz};
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
