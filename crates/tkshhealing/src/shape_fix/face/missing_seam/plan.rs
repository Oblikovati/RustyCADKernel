use super::check_wire::{OpenWireDirection, check_wire};
use crate::shape_analysis::ShapeAnalysis;
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomSurface, SurfaceBounds};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use std::cmp::Ordering;

#[derive(Clone)]
pub(super) struct MissingSeamPlan {
    pub(super) face: Face,
    pub(super) surface: Handle<dyn GeomSurface>,
    pub(super) surface_bounds: SurfaceBounds,
    pub(super) u_closed: bool,
    pub(super) v_closed: bool,
    pub(super) u_period: f64,
    pub(super) v_period: f64,
    pub(super) wires: Vec<Wire>,
    pub(super) nonmanifold: Vec<Shape>,
    pub(super) first_wire: Wire,
    pub(super) second_wire: Wire,
    pub(super) direction: OpenWireDirection,
}

struct WireBuckets {
    wires: Vec<Wire>,
    nonmanifold: Vec<Shape>,
}

#[derive(Clone)]
struct OpenWire {
    wire: Wire,
    direction: OpenWireDirection,
}

#[derive(Default)]
struct OpenWirePair {
    first: Option<OpenWire>,
    second: Option<OpenWire>,
    first_degenerate_index: usize,
    second_degenerate_index: usize,
}

pub(super) fn analyze(
    face: &Face,
    surface: &Handle<dyn GeomSurface>,
) -> Result<Option<MissingSeamPlan>, Failure> {
    if surface.is_null() {
        return Ok(None);
    }
    let (u_closed, v_closed) = closed_directions(surface)?;
    if !u_closed && !v_closed || !periodic_spline_supported(surface) {
        return Ok(None);
    }
    let bounds = finite_bounds(surface.bounds(), face)?;
    let u_period = bounded_period(bounds.0, bounds.1);
    let v_period = bounded_period(bounds.2, bounds.3);
    let mut buckets = face_wires(face)?;
    let pair = select_open_wire_pair(&mut buckets.wires, face, u_period, v_period)?;
    assemble_plan(
        face,
        surface,
        bounds,
        (u_closed, v_closed),
        (u_period, v_period),
        buckets,
        pair,
    )
}

fn closed_directions(surface: &Handle<dyn GeomSurface>) -> Result<(bool, bool), Failure> {
    let live = surface.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ShapeFix_Face::FixMissingSeam: null surface",
        )
    })?;
    Ok((live.is_u_closed()?, live.is_v_closed()?))
}

fn periodic_spline_supported(surface: &Handle<dyn GeomSurface>) -> bool {
    let spline = surface.cast::<GeomBSplineSurface>();
    let Some(spline) = spline.get() else {
        return true;
    };
    spline.is_u_periodic() || spline.is_v_periodic()
}

fn finite_bounds(bounds: SurfaceBounds, face: &Face) -> Result<SurfaceBounds, Failure> {
    let face_bounds = ShapeAnalysis::get_face_uv_bounds(face)?;
    let u = finite_direction(bounds.0, bounds.1, face_bounds.0, face_bounds.1);
    let v = finite_direction(bounds.2, bounds.3, face_bounds.2, face_bounds.3);
    Ok((u.0, u.1, v.0, v.1))
}

fn finite_direction(first: f64, last: f64, face_first: f64, face_last: f64) -> (f64, f64) {
    let start = if precision::is_infinite(first) {
        face_first
    } else {
        first
    };
    let end = if precision::is_infinite(last) {
        face_last
    } else {
        last
    };
    if (end - start).abs().partial_cmp(&precision::p_confusion()) != Some(Ordering::Less) {
        return (start, end);
    }
    if precision::is_infinite(first) {
        (start - 1000.0, end)
    } else {
        (start, end + 1000.0)
    }
}

fn bounded_period(first: f64, last: f64) -> f64 {
    (last - first).abs().min(precision::infinite())
}

fn face_wires(face: &Face) -> Result<WireBuckets, Failure> {
    let mut buckets = WireBuckets {
        wires: Vec::new(),
        nonmanifold: Vec::new(),
    };
    for child in ShapeIterator::new(&face.clone().into_shape(), false, false) {
        classify_face_child(child, &mut buckets)?;
    }
    Ok(buckets)
}

fn classify_face_child(child: Shape, buckets: &mut WireBuckets) -> Result<(), Failure> {
    if child.shape_type()? != ShapeEnum::Wire || !manifold_orientation(child.orientation()) {
        buckets.nonmanifold.push(child);
        return Ok(());
    }
    buckets.wires.push(topods::wire(&child)?);
    Ok(())
}

fn manifold_orientation(orientation: Orientation) -> bool {
    orientation == Orientation::Forward || orientation == Orientation::Reversed
}

fn select_open_wire_pair(
    wires: &mut Vec<Wire>,
    face: &Face,
    u_period: f64,
    v_period: f64,
) -> Result<OpenWirePair, Failure> {
    let mut pair = OpenWirePair::default();
    let mut index = 0;
    while index < wires.len() {
        let Some(candidate) = open_wire_at(&wires[index], face, u_period, v_period)? else {
            index += 1;
            continue;
        };
        if let Some(remove_index) = pair.assign_open_wire(candidate, index + 1) {
            wires.remove(remove_index - 1);
            pair = OpenWirePair::default();
            index = 0;
            continue;
        }
        index += 1;
    }
    Ok(pair)
}

impl OpenWirePair {
    fn assign_open_wire(&mut self, wire: OpenWire, source_index: usize) -> Option<usize> {
        if self.first.is_none() {
            self.first_degenerate_index = degenerate_index(&wire, source_index);
            self.first = Some(wire);
            return None;
        }
        if self.second.is_none() {
            self.assign_second(wire, source_index);
            return None;
        }
        self.remove_degenerate_wire(&wire, source_index)
    }

    fn assign_second(&mut self, mut wire: OpenWire, source_index: usize) {
        let first = self.first.as_ref().expect("first wire set");
        if opposite_directions(first.direction, wire.direction) {
            self.second_degenerate_index = degenerate_index(&wire, source_index);
            self.second = Some(wire);
            return;
        }
        if !same_directions(first.direction, wire.direction) {
            return;
        }
        self.second_degenerate_index = usize::from(wire.direction.all_degenerated);
        if first.direction.all_degenerated {
            let first = self.first.as_mut().expect("first wire set");
            first.wire.reverse();
            first.direction = reverse_direction(first.direction);
        } else {
            wire.wire.reverse();
        }
        self.second = Some(wire);
    }

    fn remove_degenerate_wire(&self, wire: &OpenWire, source_index: usize) -> Option<usize> {
        if wire.direction.all_degenerated {
            return Some(source_index);
        }
        if self.first_degenerate_index != 0 {
            return Some(self.first_degenerate_index);
        }
        if self.second_degenerate_index != 0 {
            return Some(self.second_degenerate_index);
        }
        None
    }
}

fn reverse_direction(direction: OpenWireDirection) -> OpenWireDirection {
    OpenWireDirection {
        u: -direction.u,
        v: -direction.v,
        all_degenerated: direction.all_degenerated,
    }
}

fn open_wire_at(
    wire: &Wire,
    face: &Face,
    u_period: f64,
    v_period: f64,
) -> Result<Option<OpenWire>, Failure> {
    let Some(direction) = check_wire(wire, face, u_period, v_period)? else {
        return Ok(None);
    };
    Ok(Some(OpenWire {
        wire: wire.clone(),
        direction,
    }))
}

fn degenerate_index(wire: &OpenWire, source_index: usize) -> usize {
    if wire.direction.all_degenerated {
        source_index
    } else {
        0
    }
}

fn opposite_directions(first: OpenWireDirection, second: OpenWireDirection) -> bool {
    first.u == -second.u && first.v == -second.v
}

fn same_directions(first: OpenWireDirection, second: OpenWireDirection) -> bool {
    first.u == second.u && first.v == second.v
}

fn assemble_plan(
    face: &Face,
    surface: &Handle<dyn GeomSurface>,
    bounds: SurfaceBounds,
    closure: (bool, bool),
    periods: (f64, f64),
    buckets: WireBuckets,
    pair: OpenWirePair,
) -> Result<Option<MissingSeamPlan>, Failure> {
    let Some(first) = pair.first else {
        return Ok(None);
    };
    let direction = first.direction;
    let second = pair.second.map_or_else(Wire::default, |wire| wire.wire);
    Ok(Some(MissingSeamPlan {
        face: face.clone(),
        surface: surface.clone(),
        surface_bounds: bounds,
        u_closed: closure.0,
        v_closed: closure.1,
        u_period: periods.0,
        v_period: periods.1,
        wires: buckets.wires,
        nonmanifold: buckets.nonmanifold,
        first_wire: first.wire,
        second_wire: second,
        direction,
    }))
}

#[cfg(test)]
mod tests {
    use super::{
        analyze, finite_direction, manifold_orientation, opposite_directions, same_directions,
    };
    use crate::shape_analysis::ShapeAnalysisEdge;
    use crate::shape_fix::face::missing_seam::check_wire::OpenWireDirection;
    use crate::shape_fix::face::missing_seam::degenerate_seam::create_for_single_open_wire;
    use crate::shape_fix::face::missing_seam::layout::order_and_align;
    use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
    use rustycad_tkbrep::topods::{Builder, Edge, Face, ShapeIterator, Wire};
    use rustycad_tkernel::precision;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
    use rustycad_tkg3d::geom::{
        GeomCircle, GeomCylindricalSurface, GeomSphericalSurface, GeomSurface,
    };
    use rustycad_tkg3d::top_abs::Orientation;
    use rustycad_tkmath::gp::{Axis2, Axis3, Direction2, Point2};
    use std::f64::consts::TAU;

    #[test]
    fn native_wire_orientation_gate_accepts_only_forward_and_reversed() {
        assert!(manifold_orientation(Orientation::Forward));
        assert!(manifold_orientation(Orientation::Reversed));
        assert!(!manifold_orientation(Orientation::Internal));
    }

    #[test]
    fn same_and_opposite_open_wire_directions_keep_native_sign_rules() {
        let positive = OpenWireDirection {
            u: 1,
            v: 0,
            all_degenerated: false,
        };
        let negative = OpenWireDirection {
            u: -1,
            v: 0,
            all_degenerated: true,
        };
        assert!(same_directions(positive, positive));
        assert!(opposite_directions(positive, negative));
    }

    #[test]
    fn native_infinite_bounds_expand_collapsed_domains_only_on_the_infinite_side() {
        assert_eq!(finite_direction(-1.0e100, 4.0, 4.0, 4.0), (-996.0, 4.0));
        assert_eq!(finite_direction(0.0, 1.0e100, 0.0, 0.0), (0.0, 1000.0));
    }

    #[test]
    fn cylinder_circle_pcurve_is_detected_as_a_positive_u_open_wire() {
        let (face, surface) = periodic_cylinder_face().unwrap();
        let plan = analyze(&face, &surface).unwrap().unwrap();
        assert_eq!((plan.direction.u, plan.direction.v), (1, 0));
        assert!(!plan.direction.all_degenerated);
        assert!(plan.second_wire.is_null());
        assert_eq!(plan.wires.len(), 1);
        assert!(plan.u_closed);
        assert!(!plan.v_closed);
        assert!((plan.u_period - TAU).abs() < precision::p_confusion());
    }

    #[test]
    fn sphere_open_u_wire_gets_native_degenerate_pole_seam() {
        let sphere = GeomSphericalSurface::new(Axis3::default(), 1.0).unwrap();
        let surface: Handle<dyn GeomSurface> = Handle::new(sphere).cast();
        let (face, _) = periodic_surface_face(surface.clone()).unwrap();
        let mut plan = analyze(&face, &surface).unwrap().unwrap();

        assert!(create_for_single_open_wire(&mut plan).unwrap());
        assert_eq!(plan.wires.len(), 2);
        let edges = ShapeIterator::new(&plan.second_wire.clone().into_shape(), false, false);
        let edge = rustycad_tkbrep::topods::edge(edges.value().unwrap()).unwrap();
        assert!(BRepTool::degenerated(&edge).unwrap());
        let analysis = ShapeAnalysisEdge::new();
        assert!(
            analysis
                .first_vertex(&edge)
                .is_same(&analysis.last_vertex(&edge))
        );
        let (curve, first, last) = analysis.pcurve_on_face(&edge, &face, true).unwrap();
        let curve = curve.get().unwrap();
        let first_point = curve.value(first).unwrap();
        let last_point = curve.value(last).unwrap();
        assert!((first_point.x() - TAU).abs() < precision::p_confusion());
        assert!(last_point.x().abs() < precision::p_confusion());
        assert!((first_point.y() - std::f64::consts::FRAC_PI_2).abs() < precision::p_confusion());
    }

    #[test]
    fn sphere_degenerate_wire_reorders_and_selects_native_periodic_split() {
        let sphere = GeomSphericalSurface::new(Axis3::default(), 1.0).unwrap();
        let surface: Handle<dyn GeomSurface> = Handle::new(sphere).cast();
        let (face, _) = periodic_surface_face(surface.clone()).unwrap();
        let mut plan = analyze(&face, &surface).unwrap().unwrap();
        assert!(create_for_single_open_wire(&mut plan).unwrap());

        let ordered = order_and_align(&mut plan, precision::confusion())
            .unwrap()
            .expect("native seam pair has pcurves");
        assert!(ordered.first_bounds.iter().all(|value| value.is_finite()));
        assert!(ordered.u_split.is_finite());
        assert!(ordered.v_split.is_finite());
        assert!(
            (ordered.u_split - 0.0).abs() < precision::p_confusion(),
            "u={}, v={}",
            ordered.u_split,
            ordered.v_split
        );
    }

    fn periodic_cylinder_face()
    -> Result<(Face, Handle<dyn GeomSurface>), rustycad_tkernel::standard::Failure> {
        let surface: Handle<dyn GeomSurface> =
            Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.0)?).cast();
        periodic_surface_face(surface)
    }

    fn periodic_surface_face(
        surface: Handle<dyn GeomSurface>,
    ) -> Result<(Face, Handle<dyn GeomSurface>), rustycad_tkernel::standard::Failure> {
        let mut face = Face::default();
        let builder = BRepBuilder::default();
        builder.make_face_on_surface(&mut face, &surface, precision::confusion())?;
        let circle: Handle<dyn rustycad_tkg3d::geom::GeomCurve> =
            Handle::new(GeomCircle::new(Axis2::default(), 1.0)?).cast();
        let mut edge = Edge::default();
        builder.make_edge_with_curve(&mut edge, &circle, precision::confusion())?;
        builder.range(&edge, 0.0, TAU, true)?;
        let pcurve: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::X)).cast();
        builder.update_edge_pcurve_on_face(&edge, &pcurve, &face, precision::confusion())?;
        let (support, location) = rustycad_tkbrep::brep::BRepTool::surface_with_location(&face)?;
        builder.range_on_surface(&edge, &support, &location, 0.0, TAU)?;
        let mut wire = Wire::default();
        Builder.make_wire(&mut wire);
        Builder.add(&mut wire, &edge.into_shape())?;
        Builder.add(&mut face, &wire.into_shape())?;
        Ok((face, surface))
    }
}
