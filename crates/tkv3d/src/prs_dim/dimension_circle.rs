//! `PrsDim_Dimension` circular geometry: the circle of a planar face, an edge or a revolved
//! face, with the middle point of its arc.
use super::{PrsDim, PrsDimCurveEnds, PrsDimKindOfSurface};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gc::MakeCircle;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle3, Point3, Vector3};
use std::f64::consts::PI;

/// A circle with the ends of its arc.
pub type PrsDimCircleArc = (Circle3, Point3, Point3);

fn is_type(
    curve: &Handle<dyn GeomCurve>,
    kind: &'static rustycad_tkernel::standard::RuntimeType,
) -> bool {
    curve
        .get()
        .is_some_and(|c| std::ptr::eq(c.dynamic_type(), kind))
}

/// `CircleFromPlanarFace`: the first circular edge of a face, with its ends.
/// Example: `circle_from_planar_face(&face)?`.
pub fn circle_from_planar_face(face: &Face) -> Result<Option<PrsDimCurveEnds>, Failure> {
    for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = topods::edge(&edge)?;
        if let Some((curve, (first, last))) = PrsDim::compute_edge_geometry(&edge)?
            && curve.is_instance(GeomCircle::static_type())
        {
            return Ok(Some((curve, (first, last))));
        }
    }
    Ok(None)
}

/// `CircleFromEdge`: the circle of a circular edge (or of a round ellipse) with its ends.
/// Example: `circle_from_edge(&edge)?`.
pub fn circle_from_edge(edge: &Edge) -> Result<Option<PrsDimCircleArc>, Failure> {
    let adaptor = BRepAdaptorCurve::with_edge(edge)?;
    let circle = match adaptor.get_type()? {
        CurveType::Circle => adaptor.circle()?,
        CurveType::Ellipse => {
            let ellipse = adaptor.ellipse()?;
            if ellipse.minor_radius() - ellipse.major_radius() >= precision::confusion() {
                return Ok(None);
            }
            Circle3::new(ellipse.position(), ellipse.minor_radius())?
        }
        _ => return Ok(None),
    };
    let first = adaptor.value(adaptor.first_parameter()?)?;
    let last = adaptor.value(adaptor.last_parameter()?)?;
    Ok(Some((circle, first, last)))
}

/// The circle of a curvilinear face: its middle V iso, or the circle through three points.
fn circle_of_revolved_face(
    face: &Face,
    surface: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    kind: PrsDimKindOfSurface,
    previous: Circle3,
) -> Result<Option<PrsDimCircleArc>, Failure> {
    let adaptor = BRepAdaptorSurface::with_face(face, true)?;
    let (mut first_u, mut last_u) = (adaptor.first_u_parameter()?, adaptor.last_u_parameter()?);
    let (first_v, last_v) = (adaptor.first_v_parameter()?, adaptor.last_v_parameter()?);
    let (mid_u, mid_v) = ((first_u + last_u) * 0.5, (first_v + last_v) * 0.5);
    let current = adaptor.d0(mid_u, mid_v)?;
    let expected = match kind {
        PrsDimKindOfSurface::Cylinder => true,
        PrsDimKindOfSurface::Revolution => {
            let basis = adaptor.basis_curve()?;
            basis
                .get()
                .is_some_and(|c| c.get_type().ok() == Some(CurveType::Line))
        }
        PrsDimKindOfSurface::Extrusion => {
            let basis = adaptor.basis_curve()?;
            basis
                .get()
                .is_some_and(|c| c.get_type().ok() == Some(CurveType::Circle))
        }
        _ => false,
    };
    if !expected {
        return Ok(None);
    }
    let curve = surface
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "PrsDim: null surface"))?
        .v_iso(mid_v)?;
    // A non-circular trimmed basis keeps the caller's circle, as natively.
    let mut circle = previous;
    if is_type(&curve, GeomCircle::static_type()) {
        circle = curve.cast::<GeomCircle>().get().expect("circle").circ()?;
    } else if is_type(&curve, GeomTrimmedCurve::static_type()) {
        let trimmed = curve.cast::<GeomTrimmedCurve>();
        let trimmed = trimmed.get().expect("trimmed");
        first_u = trimmed.first_parameter();
        last_u = trimmed.last_parameter();
        let basis = trimmed.basis_curve();
        if is_type(&basis, GeomCircle::static_type()) {
            circle = basis.cast::<GeomCircle>().get().expect("circle").circ()?;
        }
    } else {
        let p1 = adaptor.d0(first_u, mid_v)?;
        let p2 = adaptor.d0(last_u, mid_v)?;
        let made = MakeCircle::through_points(p1, current, p2)?.value()?;
        circle = made
            .get()
            .ok_or_else(|| Failure::with_message("StdFail_NotDone", "GC_MakeCircle"))?
            .circ()?;
    }
    Ok(Some((
        circle,
        elclib::value(first_u, &circle),
        elclib::value(last_u, &circle),
    )))
}

/// `InitCircularDimension`: the circle of a face, wire or edge, the middle point of its arc
/// and whether it is closed (`previous` is the in/out circle a non-circular trimmed iso keeps);
/// `None` for other shapes or non-circular geometry.
/// Example: `init_circular_dimension(&shape, circle)?`.
pub fn init_circular_dimension(
    shape: &Shape,
    previous: Circle3,
) -> Result<Option<(Circle3, Point3, bool)>, Failure> {
    let (circle, first, last) = match shape.shape_type()? {
        ShapeEnum::Face => {
            let face = topods::face(shape)?;
            let (_, surface) = PrsDim::get_plane_from_face(&face)?;
            if surface.kind == PrsDimKindOfSurface::Plane {
                let Some((curve, (first, last))) = circle_from_planar_face(&face)? else {
                    return Ok(None);
                };
                (
                    curve.cast::<GeomCircle>().get().expect("circle").circ()?,
                    first,
                    last,
                )
            } else {
                let Some(found) =
                    circle_of_revolved_face(&face, &surface.surface, surface.kind, previous)?
                else {
                    return Ok(None);
                };
                found
            }
        }
        ShapeEnum::Wire => {
            let explorer = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
            let edge = if explorer.more() {
                topods::edge(explorer.current())?
            } else {
                Edge::default()
            };
            let Some(found) = circle_from_edge(&edge)? else {
                return Ok(None);
            };
            found
        }
        ShapeEnum::Edge => {
            let Some(found) = circle_from_edge(&topods::edge(shape)?)? else {
                return Ok(None);
            };
            found
        }
        _ => return Ok(None),
    };
    let closed = first.is_equal(&last, precision::confusion());
    let center = circle.location();
    if closed {
        let x = circle.x_axis().direction();
        return Ok(Some((
            circle,
            center.translated(Vector3::from(x) * circle.radius()),
            true,
        )));
    }
    let mut first_param = elclib::parameter(&circle, first)?;
    let last_param = elclib::parameter(&circle, last)?;
    if first_param > last_param {
        first_param -= 2.0 * PI;
    }
    let middle = (first_param + last_param) * 0.5;
    let vector =
        Vector3::between(center, elclib::value(middle, &circle)).normalized()? * circle.radius();
    Ok(Some((circle, center.translated(vector), false)))
}
