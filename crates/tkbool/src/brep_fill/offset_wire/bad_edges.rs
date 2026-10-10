// Rust translation of the static CheckBadEdges, PerformCurve and QuasiFleche
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_OffsetWire.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges of the spine whose curvature at an end is too strong for the offset
//! (replaced by polylines before the offset), and the deflection sampling of the
//! polylines.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::geom_lprop::CLProps2d;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction2, Point3, Vector2, Vector3};
use rustycad_tktopalgo::brep_mat2d::{BRepMAT2dBisectingLocus, BRepMAT2dLinkTopoBilo};

/// Whether the end of the p-curve at `u` is bad: no tangent, or a curvature reaching
/// 1 / Offset on the side of the offset.
fn bad_end(props: &mut CLProps2d, u: f64, reverse: bool, lim_curv: f64) -> Result<bool, Failure> {
    let eps: f64 = precision::confusion();
    props.set_parameter(u)?;
    if !props.is_tangent_defined()? {
        return Ok(true);
    }
    let p = props.value();
    let crv: f64 = props.curvature()?;
    if crv < eps {
        return Ok(false);
    }
    let t: Direction2 = props.tangent()?;
    let mut n = Direction2::new(t.y(), -t.x())?;
    if reverse {
        n.reverse();
    }
    let pc = props.centre_of_curvature()?;
    let dir = Vector2::between(p, pc);
    // gp_Dir2d::Dot of the vector converts it to a direction (never null: the centre
    // of curvature is at the radius).
    Ok(n.dot(Direction2::from_vector(dir)?) > 0. && lim_curv <= crv + eps)
}

/// `CheckBadEdges(Spine, Offset, Locus, Link, BadEdges)`: the edges of the spine (each
/// once) that are neither lines nor circles and are bad at their first or last end.
pub(super) fn check_bad_edges(
    spine: &Face,
    offset: f64,
    locus: &BRepMAT2dBisectingLocus,
    link: &BRepMAT2dLinkTopoBilo,
) -> Result<Vec<Shape>, Failure> {
    let f_face = topods::face(&spine.oriented(Orientation::Forward))?;
    let lim_curv: f64 = 1. / offset;
    let mut a_map = MapOfShape::default();
    let mut bad_edges: Vec<Shape> = Vec::new();
    for ic in 1..=locus.number_of_contours() {
        for ie in 1..=locus.number_of_elts(ic)? {
            let se: Shape = link.generating_shape(&locus.basic_elt(ic, ie)?)?;
            if se.shape_type()? != ShapeEnum::Edge || a_map.contains(&se) {
                continue;
            }
            if is_bad_edge(&se, &f_face, lim_curv)? {
                bad_edges.push(se.clone());
                a_map.add(se);
            }
        }
    }
    Ok(bad_edges)
}

/// Whether an edge of the spine is bad.
fn is_bad_edge(se: &Shape, f_face: &Face, lim_curv: f64) -> Result<bool, Failure> {
    let e = topods::edge(se)?;
    let pc = BRepTool::curve_on_face(&e, f_face)?;
    let ac = Geom2dAdaptorCurve::with_range(&pc.pcurve, pc.first, pc.last)?;
    let a_c_type = rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d::get_type(&ac)?;
    if a_c_type == CurveType::Line || a_c_type == CurveType::Circle {
        return Ok(false);
    }
    let reverse: bool = e.orientation() == Orientation::Forward;
    let mut props = CLProps2d::new(pc.pcurve.clone(), 2, precision::confusion())?;
    Ok(bad_end(&mut props, pc.first, reverse, lim_curv)?
        || bad_end(&mut props, pc.last, reverse, lim_curv)?)
}

/// The sampling of a curve: its parameters and points.
#[derive(Default)]
pub(super) struct CurveSamples {
    /// The parameters.
    pub parameters: Vec<f64>,
    /// The points.
    pub points: Vec<Point3>,
}

/// A point of the curve with its derivative.
#[derive(Clone, Copy)]
struct SamplePoint {
    u: f64,
    p: Point3,
    v: Vector3,
}

/// `PerformCurve(Parameters, Points, C, Deflection, U1, U2, EPSILON, Nbmin)`: the
/// points of the curve between the parameters within the deflection, at least `Nbmin`.
pub(super) fn perform_curve(
    c: &dyn Adaptor3dCurve,
    deflection: f64,
    (u1, u2): (f64, f64),
    epsilon: f64,
    nbmin: i32,
) -> Result<CurveSamples, Failure> {
    let (uu1, uu2) = (u1.min(u2), u1.max(u2));
    let mut samples = CurveSamples::default();
    let (pdeb, ddeb) = c.d1(uu1)?;
    samples.parameters.push(uu1);
    samples.points.push(pdeb);
    let (pfin, dfin) = c.d1(uu2)?;
    let a_delta: f64 = uu2 - uu1;
    let a_dist: f64 = pdeb.distance(&pfin);
    if (a_delta / a_dist) > 5.0e-14 {
        let ends = (
            SamplePoint {
                u: uu1,
                p: pdeb,
                v: ddeb,
            },
            SamplePoint {
                u: uu2,
                p: pfin,
                v: dfin,
            },
        );
        quasi_fleche(
            c,
            (deflection * deflection, epsilon * epsilon),
            ends,
            nbmin,
            &mut samples,
        )?;
    }
    Ok(samples)
}

/// The sag of the arc between two samples: interpolated from the tangents, else the
/// distance of the chord middle to the curve.
fn fleche(
    c: &dyn Adaptor3dCurve,
    deb: &SamplePoint,
    delta: &SamplePoint,
    (udelta, eps): (f64, f64),
) -> Result<f64, Failure> {
    let norme: f64 = Vector3::between(deb.p, delta.p).square_magnitude();
    if norme > eps {
        // Evaluation of the arrow by interpolation. See IntWalk_IWalking::TestDeflection.
        let (n1, n2) = (deb.v.square_magnitude(), delta.v.square_magnitude());
        if n1 > eps && n2 > eps {
            let normediff: f64 =
                (*deb.v.normalized()?.xyz() - *delta.v.normalized()?.xyz()).square_modulus();
            if normediff > eps {
                return Ok(normediff * norme / 64.);
            }
        }
    }
    let pmid = Point3::from((deb.p.xyz() + delta.p.xyz()) / 2.);
    let pverif = c.value(deb.u + udelta / 2.)?;
    Ok(pmid.square_distance(&pverif))
}

/// `QuasiFleche(C, Deflection2, Udeb, Pdeb, Vdeb, Ufin, Pfin, Vfin, Nbmin, Eps,
/// Parameters, Points)`: the samples after `deb` up to `fin`, the first step of
/// `1 / (Nbmin - 1)` of the range split in two until its sag is below the deflection.
fn quasi_fleche(
    c: &dyn Adaptor3dCurve,
    (deflection2, eps): (f64, f64),
    (deb, fin): (SamplePoint, SamplePoint),
    nbmin: i32,
    samples: &mut CurveSamples,
) -> Result<(), Failure> {
    let ptslength: usize = samples.points.len();
    let mut udelta: f64 = fin.u - deb.u;
    let delta: SamplePoint = if nbmin > 2 {
        udelta /= f64::from(nbmin - 1);
        let (p, v) = c.d1(deb.u + udelta)?;
        SamplePoint {
            u: deb.u + udelta,
            p,
            v,
        }
    } else {
        SamplePoint {
            u: deb.u + udelta,
            ..fin
        }
    };
    let the_fleche: f64 = fleche(c, &deb, &delta, (udelta, eps))?;
    if the_fleche < deflection2 {
        samples.parameters.push(deb.u + udelta);
        samples.points.push(delta.p);
    } else {
        quasi_fleche(c, (deflection2, eps), (deb, delta), 3, samples)?;
    }
    if nbmin > 2 {
        let remaining: i32 = nbmin - (samples.points.len() - ptslength) as i32;
        quasi_fleche(c, (deflection2, eps), (delta, fin), remaining, samples)?;
    }
    Ok(())
}
