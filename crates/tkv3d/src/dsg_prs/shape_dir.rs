//! `DsgPrs_ShapeDirPresentation`, `DsgPrs_SymbPresentation` and `DsgPrs_TangentPresentation`.
use super::support::*;
use crate::prs3d::{Prs3dArrow, Prs3dDrawer, Prs3dText};
use crate::std_prs::StdPrsPoint;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg2d::geom2d::Geom2dLine;
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomCurve, GeomPoint};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::geom_lprop::{CLProps, SLProps};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{self, Direction3, Point2, Point3, Vector3};
use rustycad_tkservice::aspect::{AspectTypeOfLine, AspectTypeOfMarker};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_class::BRepClassFaceClassifier;
use std::f64::consts::PI;

/// `DsgPrs_ShapeDirPresentation`.
pub struct DsgPrsShapeDirPresentation;

/// `FindPointOnFace`: the centroid of the first wire's p-curve polygon when it lies on the
/// face, else the first sample; false (with the first sample) for a null area.
fn find_point_on_face(face: &Face) -> Result<(bool, Point2), Failure> {
    let wires = Explorer::new(face, ShapeEnum::Wire, ShapeEnum::Shape)?;
    if !wires.more() {
        return Ok((false, Point2::default()));
    }
    let nptt = 21;
    let mut points = vec![Point2::default(); nptt];
    let (mut area, mut xcent, mut ycent) = (0.0, 0.0, 0.0);
    let mut edges = Explorer::new(wires.current(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while edges.more() {
        let edge = topods::edge(edges.current())?;
        let npoints = sample_pcurve(&edge, face, &mut points)?;
        for i in 0..npoints - 1 {
            let h = 0.5 * (points[i].y() + points[i + 1].y());
            let c = points[i + 1].x() - points[i].x();
            let d = points[i + 1].x() + points[i].x();
            area += h * c;
            xcent += 0.5 * h * c * d;
            ycent += 0.5 * h * h * c;
        }
        edges.next()?;
    }
    if area.abs() < gp::resolution() {
        return Ok((false, points[0]));
    }
    let mut point = Point2::new(xcent / area, ycent / area);
    let classifier =
        BRepClassFaceClassifier::with_face_point2d(face, &point, gp::resolution(), false, 0.1)?;
    if matches!(classifier.state(), State::Out | State::Unknown) {
        point = points[0];
    }
    Ok((true, point))
}

// The p-curve samples of one edge in wire order: two for a line, else 21.
fn sample_pcurve(edge: &Edge, face: &Face, points: &mut [Point2]) -> Result<usize, Failure> {
    let pcurve = BRepTool::curve_on_face(edge, face)?;
    let (mut first, mut last) = (pcurve.first, pcurve.last);
    if edge.orientation() == Orientation::Reversed {
        std::mem::swap(&mut first, &mut last);
    }
    let curve = pcurve
        .pcurve
        .get()
        .ok_or_else(|| null_object("DsgPrs_ShapeDirPresentation: no p-curve"))?;
    if pcurve
        .pcurve
        .is_instance(<Geom2dLine as rustycad_tkernel::standard::RuntimeTyped>::static_type())
    {
        points[0] = curve.value(first)?;
        points[1] = curve.value(last)?;
        return Ok(2);
    }
    let nptt = points.len();
    let delta = (last - first) / (nptt - 1) as f64;
    for (i, slot) in points.iter_mut().enumerate() {
        let t = if i == 0 {
            first
        } else if i == nptt - 1 {
            last
        } else {
            first + i as f64 * delta
        };
        *slot = curve.value(t)?;
    }
    Ok(nptt)
}

/// `ComputeDir`: the point and direction at an edge end or a face's inner point, reversed
/// for a forward shape in mode 1 or a reversed one in mode 0; None when undefined.
fn compute_dir(shape: &Shape, mode: i32) -> Result<Option<(Point3, Direction3)>, Failure> {
    let found = match shape.shape_type()? {
        ShapeEnum::Edge => edge_direction(&topods::edge(shape)?, mode)?,
        ShapeEnum::Face => face_direction(&topods::face(shape)?)?,
        _ => Some((Point3::default(), Direction3::default())),
    };
    let Some((point, mut dir)) = found else {
        return Ok(None);
    };
    let orientation = shape.orientation();
    if (orientation == Orientation::Forward && mode == 1)
        || (orientation == Orientation::Reversed && mode == 0)
    {
        dir.reverse();
    }
    Ok(Some((point, dir)))
}

fn edge_direction(edge: &Edge, mode: i32) -> Result<Option<(Point3, Direction3)>, Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let source = found
        .curve
        .get()
        .ok_or_else(|| null_object("DsgPrs_ShapeDirPresentation: no 3d curve"))?;
    let curve: Handle<dyn GeomCurve> = source.copy()?.cast();
    if let Some(copy) = curve.get() {
        copy.transform(found.location.transformation())?;
    }
    let mut props = CLProps::new(curve, 1, gp::resolution())?;
    props.set_parameter(if mode == 0 { found.last } else { found.first })?;
    if !props.is_tangent_defined()? {
        return Ok(None);
    }
    let point = props.value();
    Ok(Some((point, props.tangent()?)))
}

fn face_direction(face: &Face) -> Result<Option<(Point3, Direction3)>, Failure> {
    let surface = BRepTool::surface(face)?;
    let point = if BRepTool::natural_restriction(face)? {
        let (u1, u2, v1, v2) = surface
            .get()
            .ok_or_else(|| null_object("DsgPrs_ShapeDirPresentation: no surface"))?
            .bounds();
        Point2::new((u1 + u2) * 0.5, (v1 + v2) * 0.5)
    } else {
        let (found, point) = find_point_on_face(face)?;
        if !found {
            return Ok(None);
        }
        point
    };
    let mut props = SLProps::with_parameters(surface, point.x(), point.y(), 1, gp::resolution())?;
    if !props.is_normal_defined()? {
        return Ok(None);
    }
    Ok(Some((props.value(), props.normal()?)))
}

impl DsgPrsShapeDirPresentation {
    /// `Add(prs, drawer, shape, mode)`: an arrow along the shape direction (mode 0 at the end,
    /// mode 1 reversed at the start), sized from the shape bounds; other modes draw nothing.
    /// An undefined direction keeps the default point and direction, as natively.
    /// Example: `DsgPrsShapeDirPresentation::add(&prs, &drawer, &face, 0)?`.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        shape: &Shape,
        mode: i32,
    ) -> Result<(), Failure> {
        if mode != 0 && mode != 1 {
            return Ok(());
        }
        let mut bounds = BoundingBox3::default();
        let target = direction_shape(shape, mode, &mut bounds)?;
        let (point, dir) = compute_dir(&target, mode)?.unwrap_or_default();
        let [x0, y0, z0, x1, y1, z1] = bounds.get()?;
        let mut length = Point3::new(x0, y0, z0).distance(&Point3::new(x1, y1, z1)) / 3.0;
        if length >= 20000.0 {
            length = 50.0;
        }
        let point2 = Point3::from(point.xyz() + dir.xyz() * length);
        let drawer = drawer
            .get()
            .ok_or_else(|| null_object("DsgPrs_ShapeDirPresentation: the drawer is null"))?;
        let group = current_group(prs)?;
        if let (Some(value), Some(line)) = (group.get(), drawer.line_aspect().get()) {
            value.set_primitives_aspect(&line.aspect().cast());
        }
        add_array(&group, &segments(2, &[point, point2])?);
        Prs3dArrow::draw(
            &current_group(prs)?,
            &point2,
            &dir,
            PI / 180.0 * 10.0,
            length * 0.3,
        )
    }
}

// The edge or face whose direction is shown, filling the bounds as natively.
fn direction_shape(shape: &Shape, mode: i32, bounds: &mut BoundingBox3) -> Result<Shape, Failure> {
    match shape.shape_type()? {
        ShapeEnum::Edge | ShapeEnum::Face => {
            BRepBndLib::add(shape, bounds, true)?;
            Ok(shape.clone())
        }
        ShapeEnum::Wire => {
            let mut edges: Vec<Shape> = Vec::new();
            let mut explorer = WireExplorer::with_wire(&topods::wire(shape)?)?;
            while explorer.more() {
                let edge: Shape = explorer.current().clone().into_shape();
                if edges.len() < 3 {
                    BRepBndLib::add(&edge, bounds, true)?;
                }
                edges.push(edge);
                explorer.next()?;
            }
            let chosen = if mode == 0 {
                edges.last()
            } else {
                edges.first()
            };
            chosen
                .cloned()
                .ok_or_else(|| null_object("DsgPrs_ShapeDirPresentation: the wire has no edge"))
        }
        _ => {
            let mut faces: Vec<Shape> = Vec::new();
            let mut explorer = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)?;
            while explorer.more() {
                faces.push(explorer.current().clone());
                BRepBndLib::add(explorer.current(), bounds, true)?;
                if faces.len() > 3 {
                    break;
                }
                explorer.next()?;
            }
            faces
                .last()
                .cloned()
                .ok_or_else(|| null_object("DsgPrs_ShapeDirPresentation: the shape has no face"))
        }
    }
}

/// `DsgPrs_SymbPresentation`.
pub struct DsgPrsSymbPresentation;

impl DsgPrsSymbPresentation {
    /// `Add`: a green text and a RING2 point at the offset point; the dimension text aspect
    /// and the drawer point aspect are changed, as natively.
    /// Example: `DsgPrsSymbPresentation::add(&prs, &drawer, &t, &o)?`.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        text: &ExtendedString,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect
            .get()
            .ok_or_else(|| null_object("DsgPrs: the dimension aspect is null"))?;
        let text_aspect = la.text_aspect();
        if let Some(value) = text_aspect.get() {
            value.set_color(Color::named(NamedColor::GREEN));
        }
        Prs3dText::draw(&current_group(prs)?, &text_aspect, text, offset);
        let point: Handle<dyn GeomPoint> =
            Handle::new(GeomCartesianPoint::from_pnt(*offset)).cast();
        let point_aspect = drawer.get().map(|drawer| drawer.point_aspect());
        if let Some(value) = point_aspect.as_ref().and_then(|aspect| aspect.get()) {
            value.set_type_of_marker(AspectTypeOfMarker::Ring2);
        }
        StdPrsPoint::add(prs, &point, drawer)
    }
}

/// `DsgPrs_TangentPresentation`.
pub struct DsgPrsTangentPresentation;

impl DsgPrsTangentPresentation {
    /// `Add`: a tangent segment with arrows at both ends; the dimension line is set solid and
    /// the arrow length to a fifth of the length, as natively.
    /// Example: `DsgPrsTangentPresentation::add(&prs, &drawer, &o, &d, 10.0)?`.
    pub fn add(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        offset: &Point3,
        direction: &Direction3,
        length: f64,
    ) -> Result<(), Failure> {
        let vec = Vector3::from(*direction);
        let p1 = offset.translated(vec * length);
        let p2 = offset.translated(vec * -length);
        let aspect = dimension_aspect(drawer)?;
        let la = aspect
            .get()
            .ok_or_else(|| null_object("DsgPrs: the dimension aspect is null"))?;
        if let Some(line) = la.line_aspect().get() {
            line.set_type_of_line(AspectTypeOfLine::Solid);
        }
        if let Some(arrow) = la.arrow_aspect().get() {
            arrow.set_length(length / 5.0);
            arrow.set_length(length / 5.0);
        }
        let group = current_line_group(prs, la)?;
        add_array(&group, &segments(2, &[p1, p2])?);
        draw_arrow(&new_line_group(prs, la)?, la, &p1, direction)?;
        draw_arrow(&new_line_group(prs, la)?, la, &p2, &direction.reversed())
    }
}
