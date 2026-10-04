//! BRepOffset_Inter3d::FaceInter for unshared planar face pairs.
use super::super::BRepOffsetInter3d;
use rustycad_tkbo::int_tools::{Curve, FaceFace};
use rustycad_tkbool::brep_algo::BRepAlgoImage;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::Explorer,
    topods::{Edge, Face, Shape, Vertex},
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::{
    geom::GeomPlane,
    geom_adaptor::GeomAdaptorCurve,
    top_abs::{Orientation, ShapeEnum, State},
};
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::gp::{self, Point3, Vector3};
use rustycad_tktopalgo::brep_lib::same_parameter_edge;

impl BRepOffsetInter3d {
    /// Intersects unshared planar images and stores their section edge; common-topology cases
    /// return `Standard_NotImplemented` pending native branches.
    /// Example: `intersections.face_inter(&f1, &f2, &images)?`.
    pub fn face_inter(
        &mut self,
        first: &Face,
        second: &Face,
        images: &BRepAlgoImage,
    ) -> Result<(), Failure> {
        if self.should_skip(first, second, images)? {
            return Ok(());
        }
        self.require_supported_pair(first, second, images)?;
        let curves = intersect_planar_faces(first, second)?;
        self.store_curves(first, second, &curves)
    }

    fn should_skip(
        &self,
        first: &Face,
        second: &Face,
        images: &BRepAlgoImage,
    ) -> Result<bool, Failure> {
        if first.is_same(second) || self.is_done(first, second) {
            return Ok(true);
        }
        let first_source = images.image_from(first)?;
        let second_source = images.image_from(second)?;
        Ok(first_source.is_same(&second_source))
    }

    fn require_supported_pair(
        &self,
        first: &Face,
        second: &Face,
        images: &BRepAlgoImage,
    ) -> Result<(), Failure> {
        let first_source = images.image_from(first)?;
        let second_source = images.image_from(second)?;
        rustycad_tkbrep::topods::face(&first_source)?;
        rustycad_tkbrep::topods::face(&second_source)?;
        require_plane_face(first)?;
        require_plane_face(second)?;
        self.reject_shared_topology(first, second)
    }

    fn reject_shared_topology(&self, first: &Face, second: &Face) -> Result<(), Failure> {
        let mut common = rustycad_tkernel::ncollection::List::default();
        if self
            .as_des()
            .get()
            .is_some_and(|tool| tool.has_common_descendant(first, second, &mut common))
            || has_common_edge_or_vertex(first, second)?
        {
            return Err(not_implemented("shared edge or vertex face branch"));
        }
        Ok(())
    }

    fn store_curves(
        &mut self,
        first: &Face,
        second: &Face,
        curves: &[Curve],
    ) -> Result<(), Failure> {
        if curves.len() > 1 {
            return Err(not_implemented("multiple planar section curves"));
        }
        let Some(curve) = curves.first() else {
            return self.store(first, second, &[], &[]);
        };
        let section = make_section_edge(curve, first, second, self.side())?;
        let first_edges = [section.0];
        let second_edges = [section.1];
        self.store(first, second, &first_edges, &second_edges)
    }
}

fn intersect_planar_faces(first: &Face, second: &Face) -> Result<Vec<Curve>, Failure> {
    let mut intersection = FaceFace::default();
    intersection.set_parameters(true, true, true, precision::confusion());
    intersection.perform(first, second, false)?;
    if !intersection.is_done() {
        return Ok(Vec::new());
    }
    Ok(intersection.lines()?.to_vec())
}

fn require_plane_face(face: &Face) -> Result<(), Failure> {
    let surface = BRepTool::surface(face)?;
    if surface.cast::<GeomPlane>().is_null() {
        let actual = surface
            .dynamic_type()
            .map_or("null Geom_Surface", |runtime_type| runtime_type.name());
        return Err(Failure::with_message(
            "Standard_NotImplemented",
            &format!("unsupported face surface {actual}; expected Geom_Plane"),
        ));
    }
    Ok(())
}

fn has_common_edge_or_vertex(first: &Face, second: &Face) -> Result<bool, Failure> {
    for kind in [ShapeEnum::Edge, ShapeEnum::Vertex] {
        let first_shapes = shapes_of_type(first, kind)?;
        let second_shapes = shapes_of_type(second, kind)?;
        if first_shapes
            .iter()
            .any(|left| second_shapes.iter().any(|right| right.is_same(left)))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn shapes_of_type(face: &Face, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let face_shape: Shape = face.clone().into();
    let mut explorer = Explorer::new(&face_shape, kind, ShapeEnum::Shape)?;
    let mut shapes = Vec::new();
    while explorer.more() {
        shapes.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(shapes)
}

fn make_section_edge(
    curve: &Curve,
    first: &Face,
    second: &Face,
    side: State,
) -> Result<(Shape, Shape), Failure> {
    let (range, first_point, last_point) = curve_bounds(curve)?;
    let edge = make_edge_geometry(curve, range, first_point, last_point)?;
    add_face_pcurves(&edge, curve, first, second)?;
    let (first_orientation, second_orientation) = orient_section(curve, first, second, range)?;
    let first_orientation = apply_side(first_orientation, side);
    let second_orientation = apply_side(second_orientation, side);
    same_parameter_edge(&edge, precision::confusion())?;
    Ok((
        edge.oriented(first_orientation),
        edge.oriented(second_orientation),
    ))
}

fn curve_bounds(curve: &Curve) -> Result<((f64, f64), Point3, Point3), Failure> {
    let mut first = 0.0;
    let mut last = 0.0;
    let mut start = Point3::default();
    let mut finish = Point3::default();
    if !curve.bounds(&mut first, &mut last, &mut start, &mut finish)? {
        return Err(not_implemented("unbounded plane section curve"));
    }
    Ok(((first, last), start, finish))
}

fn make_edge_geometry(
    curve: &Curve,
    range: (f64, f64),
    first_point: Point3,
    last_point: Point3,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge_with_curve(&mut edge, curve.curve(), curve.tolerance())?;
    add_boundary_vertex(&builder, &mut edge, first_point, false)?;
    add_boundary_vertex(&builder, &mut edge, last_point, true)?;
    builder.range(&edge, range.0, range.1, false)?;
    Ok(edge)
}

fn add_boundary_vertex(
    builder: &BRepBuilder,
    edge: &mut Edge,
    point: Point3,
    reversed: bool,
) -> Result<(), Failure> {
    let mut vertex = Vertex::default();
    builder.make_vertex_at(&mut vertex, point, precision::confusion())?;
    let occurrence = if reversed {
        vertex.reversed()
    } else {
        vertex.into_shape()
    };
    builder.add(edge, &occurrence)
}

fn add_face_pcurves(
    edge: &Edge,
    curve: &Curve,
    first: &Face,
    second: &Face,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let first_curve = required_pcurve(curve.first_curve2d(), "first")?;
    let second_curve = required_pcurve(curve.second_curve2d(), "second")?;
    builder.update_edge_pcurve_on_face(edge, first_curve, first, curve.tolerance())?;
    builder.update_edge_pcurve_on_face(edge, second_curve, second, curve.tolerance())
}

fn required_pcurve<'a>(
    pcurve: &'a Handle<dyn Geom2dCurve>,
    label: &str,
) -> Result<&'a Handle<dyn Geom2dCurve>, Failure> {
    if pcurve.get().is_none() {
        return Err(Failure::with_message(
            "Standard_NotImplemented",
            &format!("plane section has null {label} pcurve; expected both face pcurves"),
        ));
    }
    Ok(pcurve)
}

fn orient_section(
    curve: &Curve,
    first: &Face,
    second: &Face,
    range: (f64, f64),
) -> Result<(Orientation, Orientation), Failure> {
    let parameter = middle_length_parameter(curve, range)?;
    let tangent = section_tangent(curve, parameter)?;
    let first_normal = face_normal(first, curve.first_curve2d(), parameter)?;
    let second_normal = face_normal(second, curve.second_curve2d(), parameter)?;
    Ok(orient_from_normals(
        first_normal,
        second_normal,
        tangent,
        first,
        second,
    ))
}

fn middle_length_parameter(curve: &Curve, range: (f64, f64)) -> Result<f64, Failure> {
    let adaptor = GeomAdaptorCurve::with_curve(curve.curve())?;
    let half_length = AbscissaPoint::length(&adaptor)? / 2.0;
    Ok(AbscissaPoint::from_abscissa(&adaptor, half_length, range.0)
        .and_then(|point| point.parameter())
        .unwrap_or(range.0 + (range.1 - range.0) / 2.0))
}

fn section_tangent(curve: &Curve, parameter: f64) -> Result<Vector3, Failure> {
    let curve = curve
        .curve()
        .get()
        .ok_or_else(|| not_implemented("null spatial section curve; expected a 3d curve"))?;
    let tangent = curve.eval_dn(parameter, 1)?;
    if tangent.square_magnitude() <= gp::resolution() {
        return Ok(tangent);
    }
    tangent.normalized()
}

fn face_normal(
    face: &Face,
    pcurve: &Handle<dyn Geom2dCurve>,
    parameter: f64,
) -> Result<Vector3, Failure> {
    let pcurve = pcurve
        .get()
        .ok_or_else(|| not_implemented("null face pcurve; expected a pcurve on each face"))?;
    let uv = pcurve.eval_d0(parameter)?;
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let derivatives = surface
        .get()
        .ok_or_else(|| not_implemented("null face surface; expected Geom_Surface"))?
        .eval_d1(uv.x(), uv.y())?;
    let mut normal = derivatives
        .d1u
        .crossed(derivatives.d1v)
        .transformed(location.transformation());
    if face.orientation() == Orientation::Reversed {
        normal.reverse();
    }
    Ok(normal)
}

fn orient_from_normals(
    first_normal: Vector3,
    second_normal: Vector3,
    tangent: Vector3,
    first: &Face,
    second: &Face,
) -> (Orientation, Orientation) {
    let first = orient_from_product(first_normal.dot(second_normal.crossed(tangent)), first);
    let second = orient_from_product(second_normal.dot(first_normal.crossed(tangent)), second);
    (first, second)
}

fn orient_from_product(product: f64, face: &Face) -> Orientation {
    let forward = product < 0.0;
    let reverse = face.orientation() == Orientation::Reversed;
    if forward ^ reverse {
        Orientation::Forward
    } else {
        Orientation::Reversed
    }
}

fn apply_side(orientation: Orientation, side: State) -> Orientation {
    if side == State::Out {
        return reverse_orientation(orientation);
    }
    orientation
}

fn reverse_orientation(orientation: Orientation) -> Orientation {
    match orientation {
        Orientation::Forward => Orientation::Reversed,
        Orientation::Reversed => Orientation::Forward,
        other => other,
    }
}

fn not_implemented(branch: &str) -> Failure {
    Failure::with_message(
        "Standard_NotImplemented",
        &format!("BRepOffset_Inter3d::FaceInter does not support {branch}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkbool::brep_algo::BRepAlgoAsDes;
    use rustycad_tkbrep::topods::edge;
    use rustycad_tkg3d::geom::GeomSurface;
    use rustycad_tkg3d::top_abs::Orientation;
    use rustycad_tkmath::gp::{Direction3, Plane3};
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

    #[test]
    fn unbounded_plane_face_intersection_matches_native_store_state() {
        let first_source = plane_face(Point3::new(0.0, 0.0, 2.0), Direction3::Z);
        let second_source = plane_face(Point3::new(0.0, 2.0, 0.0), Direction3::Y);
        let first_offset = plane_face(Point3::default(), Direction3::Z);
        let second_offset = plane_face(Point3::default(), Direction3::Y);
        let mut images = BRepAlgoImage::new();
        images.bind(&first_source, &first_offset).unwrap();
        images.bind(&second_source, &second_offset).unwrap();
        let as_des = Handle::new(BRepAlgoAsDes::new());
        let mut intersections = BRepOffsetInter3d::new(&as_des, State::In, 1e-7);

        intersections
            .face_inter(&first_offset, &second_offset, &images)
            .unwrap();

        assert_eq!(intersections.touched_faces().len(), 2);
        assert_eq!(intersections.new_edges().len(), 1);
        assert!(intersections.is_done(&first_offset, &second_offset));
        assert!(intersections.is_done(&second_offset, &first_offset));
        assert_eq!(as_des.get().unwrap().descendant(&first_offset).count(), 1);
        assert_eq!(as_des.get().unwrap().descendant(&second_offset).count(), 1);
        assert_section_geometry(&intersections, &as_des, &first_offset, &second_offset);
    }

    #[test]
    fn bounded_plane_pair_intersection_matches_native_section_and_store() {
        let first_source = finite_plane_face(Point3::new(0.0, 0.0, 2.0), Direction3::Z);
        let second_source = finite_plane_face(Point3::new(0.0, 2.0, 0.0), Direction3::Y);
        let first_offset = finite_plane_face(Point3::default(), Direction3::Z);
        let second_offset = finite_plane_face(Point3::default(), Direction3::Y);
        let mut images = BRepAlgoImage::new();
        images.bind(&first_source, &first_offset).unwrap();
        images.bind(&second_source, &second_offset).unwrap();
        let as_des = Handle::new(BRepAlgoAsDes::new());
        let mut intersections = BRepOffsetInter3d::new(&as_des, State::In, 1e-7);

        intersections
            .face_inter(&first_offset, &second_offset, &images)
            .unwrap();

        assert_eq!(intersections.touched_faces().len(), 2);
        assert_eq!(intersections.new_edges().len(), 1);
        assert!(intersections.is_done(&first_offset, &second_offset));
        assert!(intersections.is_done(&second_offset, &first_offset));
        assert_eq!(as_des.get().unwrap().descendant(&first_offset).count(), 1);
        assert_eq!(as_des.get().unwrap().descendant(&second_offset).count(), 1);
        assert_bounded_section_geometry(&intersections, &as_des, &second_offset);
    }

    fn plane_face(origin: Point3, normal: Direction3) -> Face {
        let frame = Plane3::new(origin, normal).unwrap().position();
        let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
        let mut face = Face::default();
        BRepBuilder::default()
            .make_face_on_surface(&mut face, &surface, precision::confusion())
            .unwrap();
        face.set_orientation(Orientation::Forward);
        face
    }

    fn finite_plane_face(origin: Point3, normal: Direction3) -> Face {
        BRepBuilderAPIMakeFace::from_plane_bounds(
            Plane3::new(origin, normal).unwrap(),
            (-1.0, 1.0, -1.0, 1.0),
        )
        .unwrap()
        .face()
        .unwrap()
    }

    fn assert_section_geometry(
        intersections: &BRepOffsetInter3d,
        as_des: &Handle<BRepAlgoAsDes>,
        first: &Face,
        second: &Face,
    ) {
        let first_edge = edge(&intersections.new_edges()[0]).unwrap();
        let second_edges = as_des.get().unwrap().descendant(second).collect::<Vec<_>>();
        let second_edge = edge(&second_edges[0]).unwrap();
        assert_eq!(first_edge.orientation(), Orientation::Reversed);
        assert_eq!(second_edge.orientation(), Orientation::Forward);
        let (first_parameter, last_parameter) = BRepTool::range(&first_edge).unwrap();
        assert_eq!((first_parameter, last_parameter), (-2.0e100, 2.0e100));
        let (curve, _, _) = BRepTool::curve(&first_edge).unwrap();
        let first_point = curve.eval_d0(first_parameter).unwrap();
        let last_point = curve.eval_d0(last_parameter).unwrap();
        assert_eq!(first_point, Point3::new(2.0e100, 0.0, 0.0));
        assert_eq!(last_point, Point3::new(-2.0e100, 0.0, 0.0));
        assert!(intersections.is_done(first, second));
    }

    fn assert_bounded_section_geometry(
        intersections: &BRepOffsetInter3d,
        as_des: &Handle<BRepAlgoAsDes>,
        second: &Face,
    ) {
        let section = edge(&intersections.new_edges()[0]).unwrap();
        let second_edges = as_des.get().unwrap().descendant(second).collect::<Vec<_>>();
        assert_eq!(section.orientation(), Orientation::Reversed);
        assert_eq!(
            edge(&second_edges[0]).unwrap().orientation(),
            Orientation::Forward
        );
        assert_eq!(BRepTool::range(&section).unwrap(), (-1.0, 1.0));
        let (curve, _, _) = BRepTool::curve(&section).unwrap();
        assert_eq!(curve.eval_d0(-1.0).unwrap(), Point3::new(1.0, 0.0, 0.0));
        assert_eq!(curve.eval_d0(1.0).unwrap(), Point3::new(-1.0, 0.0, 0.0));
    }
}
