//! `PrsDim_LengthDimension` geometry initialization: the measured points of two edges, an
//! edge and a vertex, an edge and a face, two faces, or one straight edge.
use super::length_dimension::PrsDimLengthDimension;
use super::*;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeomalgo::geom_api::{
    ExtremaCurveCurve, ExtremaSurfaceSurface, ProjectPointOnCurve,
};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{self, Axis2, Direction3, Plane3, Point2, Point3};
use rustycad_tktopalgo::brep_extrema::BRepExtremaDistShapeShape;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

fn make_dir(from: &Point3, to: &Point3) -> Result<Direction3, Failure> {
    Direction3::from_xyz(to.xyz() - from.xyz())
        .map_err(|_| Failure::with_message("StdFail_NotDone", "gce_MakeDir::Value() - no result"))
}

/// The edge tangent at a parameter, when not degenerate.
fn edge_tangent(edge: &Edge, parameter: f64) -> Result<Option<Direction3>, Failure> {
    let adaptor = BRepAdaptorCurve::with_edge(edge)?;
    let (_, tangent) = adaptor.d1(parameter)?;
    if tangent.square_magnitude() <= gp::resolution() {
        return Ok(None);
    }
    Ok(Some(Direction3::from_vector(tangent)?))
}

impl PrsDimLengthDimension {
    fn set_points(&self, first: Point3, second: Point3) {
        let mut f = self.fields.write();
        f.first_point = first;
        f.second_point = second;
    }

    /// `InitTwoEdgesLength`: the points of two parallel lines (one end and its projection),
    /// else the nearest points of the curves, with the attach direction.
    /// Example: `dimension.init_two_edges_length(&e1, &e2)?`.
    pub fn init_two_edges_length(
        &self,
        first: &Edge,
        second: &Edge,
    ) -> Result<Option<Direction3>, Failure> {
        let Some(two) = PrsDim::compute_two_edges(first, second)? else {
            return Ok(None);
        };
        let line1 = two.first_curve.cast::<GeomLine>();
        let line2 = two.second_curve.cast::<GeomLine>();
        if let (Some(l1), Some(l2)) = (line1.get(), line2.get()) {
            let direction = l1.lin().direction();
            if !direction.is_parallel(l2.lin().direction(), precision::angular()) {
                return Ok(None);
            }
            let (p11, p12) = two.first_ends;
            let (p21, p22) = two.second_ends;
            if !two.first_infinite {
                for p in [p11, p12] {
                    let (inside, nearest) =
                        PrsDim::nearest_on_curve(&two.second_curve, &p, &p21, &p22)?;
                    if inside {
                        self.set_points(p, nearest);
                        return Ok(self.is_valid_points(&p, &nearest).then_some(direction));
                    }
                }
            }
            if !two.second_infinite {
                for p in [p21, p22] {
                    let (inside, nearest) =
                        PrsDim::nearest_on_curve(&two.first_curve, &p, &p11, &p12)?;
                    if inside {
                        self.set_points(nearest, p);
                        return Ok(self.is_valid_points(&nearest, &p).then_some(direction));
                    }
                }
            }
            return self.extrema_length(
                first,
                &two.first_curve,
                &two.second_curve,
                Some(direction),
            );
        }
        self.extrema_length(first, &two.first_curve, &two.second_curve, None)
    }

    fn extrema_length(
        &self,
        first: &Edge,
        curve1: &Handle<dyn GeomCurve>,
        curve2: &Handle<dyn GeomCurve>,
        line_direction: Option<Direction3>,
    ) -> Result<Option<Direction3>, Failure> {
        let extrema = ExtremaCurveCurve::with_curves(curve1, curve2)?;
        if extrema.nb_extrema() == 0 {
            return Ok(None);
        }
        let (p1, p2) = extrema.nearest_points()?;
        self.set_points(p1, p2);
        if !self.is_valid_points(&p1, &p2) {
            return Ok(None);
        }
        if let Some(direction) = line_direction {
            return Ok(Some(direction));
        }
        let (parameter, _) = extrema.lower_distance_parameters()?;
        edge_tangent(first, parameter)
    }

    /// `InitEdgeVertexLength`: the vertex and its projection on the edge, with the edge
    /// direction there; `dimension.init_edge_vertex_length(&edge, &vertex)?`.
    pub fn init_edge_vertex_length(
        &self,
        edge: &Edge,
        vertex: &Vertex,
    ) -> Result<Option<Direction3>, Failure> {
        let Some((curve, _, _)) = PrsDim::compute_edge_geometry_infinite(edge)? else {
            return Ok(None);
        };
        let point = BRepTool::pnt(vertex)?;
        self.fields.write().first_point = point;
        if let Some(line) = curve.cast::<GeomLine>().get() {
            let lin = line.lin();
            let nearest = PrsDim::nearest_on_line(&lin, &point)?;
            self.fields.write().second_point = nearest;
            return Ok(self
                .is_valid_points(&point, &nearest)
                .then_some(lin.direction()));
        }
        let projector = ProjectPointOnCurve::with_curve(point, &curve)?;
        if projector.nb_points() == 0 {
            return Ok(None);
        }
        let nearest = projector.nearest_point()?;
        self.fields.write().second_point = nearest;
        if !self.is_valid_points(&point, &nearest) {
            return Ok(None);
        }
        edge_tangent(edge, projector.lower_distance_parameter()?)
    }

    /// `InitEdgeFaceLength`: the minimal distance points of an edge and a face, with the edge
    /// direction (reversed toward its first end); `dimension.init_edge_face_length(&e, &f)?`.
    pub fn init_edge_face_length(
        &self,
        edge: &Edge,
        face: &Face,
    ) -> Result<(bool, Direction3), Failure> {
        let mut direction = gp::dx();
        let distance = BRepExtremaDistShapeShape::with_shapes(
            &edge.clone().into(),
            &face.clone().into(),
            (ExtFlag::Min, ExtAlgo::Grad),
        )?;
        if !distance.is_done() || distance.nb_solution() < 1 {
            return Ok((false, direction));
        }
        let (p1, p2) = (distance.point_on_shape1(1)?, distance.point_on_shape2(1)?);
        self.set_points(p1, p2);
        let adaptor = BRepAdaptorCurve::with_edge(edge)?;
        let (first, last) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
        let parameter = if distance.support_on_shape1(1)?.shape_type()? == ShapeEnum::Edge {
            distance.par_on_edge_s1(1)?
        } else {
            let d1 = adaptor.value(first)?.square_distance(&p1);
            let d2 = adaptor.value(last)?.square_distance(&p1);
            if d1 < d2 { first } else { last }
        };
        let (_, tangent) = adaptor.d1(parameter)?;
        if tangent.square_magnitude() > gp::resolution() {
            direction = Direction3::from_vector(tangent)?;
        }
        if (parameter - first).abs() < (parameter - last).abs() {
            direction.reverse();
        }
        Ok((self.is_valid_points(&p1, &p2), direction))
    }

    fn faces_length(&self, first: &Face, second: &Face) -> Result<(bool, Option<Plane3>), Failure> {
        let surface1 = PrsDim::init_face_length(first)?;
        self.base.fields.write().geometry_type = PrsDimGeometryType::Faces;
        let surface2 = PrsDim::init_face_length(second)?;
        if surface1.kind == PrsDimKindOfSurface::Plane {
            let (n1, n2) = (
                surface1.plane.axis().direction(),
                surface2.plane.axis().direction(),
            );
            if !n1.is_parallel(n2, precision::angular()) {
                return Ok((false, None));
            }
            let explorer = Explorer::new(first, ShapeEnum::Vertex, ShapeEnum::Shape)?;
            let p1 = if explorer.more() {
                BRepTool::pnt(&topods::vertex(explorer.current())?)?
            } else {
                surface1.plane.location()
            };
            let mut p2 = PrsDim::project_point_on_plane(&p1, &surface2.plane);
            let (u, v) = elslib::parameters(&surface2.plane, p2);
            let classifier = BRepTopAdaptorFClass2d::new(second, precision::confusion())?;
            let state = classifier.perform(Point2::new(u, v), false)?;
            if state == State::Out || state == State::Unknown {
                p2 = PrsDim::nearest_vertex(&second.clone().into(), &p1)?;
            }
            self.set_points(p1, p2);
            if !self.is_valid_points(&p1, &p2) {
                return Ok((false, None));
            }
            let plane = self.compute_plane(&surface1.plane.position().x_direction())?;
            return Ok((true, Some(plane)));
        }
        let (u1min, u1max, v1min, v1max) = uv_bounds(first)?;
        let (u2min, u2max, v2min, v2max) = uv_bounds(second)?;
        let extrema = ExtremaSurfaceSurface::with_bounds(
            &surface1.surface,
            &surface2.surface,
            (u1min, u1max, v1min, v1max),
            (u2min, u2max, v2min, v2max),
        )?;
        let (u1, v1, u2, v2) = extrema.lower_distance_parameters()?;
        let s1 = BRepTool::surface(first)?;
        let s2 = BRepTool::surface(second)?;
        let null = || Failure::with_message("Standard_NullObject", "PrsDim: null surface");
        let p1 = s1.get().ok_or_else(null)?.value(u1, v1)?;
        let p2 = s2.get().ok_or_else(null)?.value(u2, v2)?;
        self.set_points(p1, p2);
        let axes = Axis2::new(p1, make_dir(&p1, &p2)?)?;
        let attach = axes.x_direction();
        if !self.is_valid_points(&p1, &p2) {
            return Ok((false, None));
        }
        let plane = self.compute_plane(&attach)?;
        Ok((self.is_valid_points(&p1, &p2), Some(plane)))
    }

    fn computed(
        &self,
        success: bool,
        direction: Direction3,
    ) -> Result<(bool, Option<Plane3>), Failure> {
        if !success {
            return Ok((false, None));
        }
        Ok((true, Some(self.compute_plane(&direction)?)))
    }

    /// `InitTwoShapesPoints`: the points (and computed plane) of two shapes, with the
    /// geometry type; `dimension.init_two_shapes_points(&s1, &s2)?`.
    pub fn init_two_shapes_points(
        &self,
        first: &Shape,
        second: &Shape,
    ) -> Result<(bool, Option<Plane3>), Failure> {
        let set_type = |kind| self.base.fields.write().geometry_type = kind;
        match (first.shape_type()?, second.shape_type()?) {
            (ShapeEnum::Face, ShapeEnum::Face) => {
                self.faces_length(&topods::face(first)?, &topods::face(second)?)
            }
            (ShapeEnum::Face, ShapeEnum::Edge) => {
                // The first face is still initialized natively (its outputs unused).
                PrsDim::init_face_length(&topods::face(first)?)?;
                set_type(PrsDimGeometryType::EdgeFace);
                let (ok, direction) =
                    self.init_edge_face_length(&topods::edge(second)?, &topods::face(first)?)?;
                self.computed(ok, direction)
            }
            (ShapeEnum::Face, _) => {
                PrsDim::init_face_length(&topods::face(first)?)?;
                Ok((false, None))
            }
            (ShapeEnum::Edge, ShapeEnum::Vertex) => {
                set_type(PrsDimGeometryType::EdgeVertex);
                match self
                    .init_edge_vertex_length(&topods::edge(first)?, &topods::vertex(second)?)?
                {
                    Some(direction) => self.computed(true, direction),
                    None => Ok((false, None)),
                }
            }
            (ShapeEnum::Edge, ShapeEnum::Edge) => {
                set_type(PrsDimGeometryType::Edges);
                match self.init_two_edges_length(&topods::edge(first)?, &topods::edge(second)?)? {
                    Some(direction) => self.computed(true, direction),
                    None => Ok((false, None)),
                }
            }
            (ShapeEnum::Edge, ShapeEnum::Face) => {
                set_type(PrsDimGeometryType::EdgeFace);
                let (ok, direction) =
                    self.init_edge_face_length(&topods::edge(first)?, &topods::face(second)?)?;
                self.computed(ok, direction)
            }
            (ShapeEnum::Vertex, ShapeEnum::Vertex) => {
                set_type(PrsDimGeometryType::Points);
                let p1 = BRepTool::pnt(&topods::vertex(first)?)?;
                let p2 = BRepTool::pnt(&topods::vertex(second)?)?;
                self.set_points(p1, p2);
                Ok((self.is_valid_points(&p1, &p2), None))
            }
            (ShapeEnum::Vertex, ShapeEnum::Edge) => {
                set_type(PrsDimGeometryType::EdgeVertex);
                match self
                    .init_edge_vertex_length(&topods::edge(second)?, &topods::vertex(first)?)?
                {
                    Some(direction) => self.computed(true, direction),
                    None => Ok((false, None)),
                }
            }
            _ => Ok((false, None)),
        }
    }

    /// `InitOneShapePoints`: the ends of a finite straight edge.
    /// Example: `dimension.init_one_shape_points(&shape)?`.
    pub fn init_one_shape_points(&self, shape: &Shape) -> Result<bool, Failure> {
        if shape.shape_type()? != ShapeEnum::Edge {
            return Ok(false);
        }
        let adaptor = BRepAdaptorCurve::with_edge(&topods::edge(shape)?)?;
        let (first, last) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
        if adaptor.get_type()? != CurveType::Line {
            return Ok(false);
        }
        if precision::is_infinite(first) || precision::is_infinite(last) {
            return Ok(false);
        }
        let (p1, p2) = (adaptor.value(first)?, adaptor.value(last)?);
        self.set_points(p1, p2);
        Ok(self.is_valid_points(&p1, &p2))
    }
}
