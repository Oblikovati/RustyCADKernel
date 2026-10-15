// Rust port of ApplicationFramework/TKVCAF/TPrsStd/TPrsStd_ConstraintTools.cxx (the angle
// and offset presentations) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ComputeAngle`, `ComputeAngleForOneFace` and `ComputeOffset`: the presentations that
//! derive their plane from the measured faces or edges.
use super::constraint_tools::{
    ConstraintPresentation, TPrsStdConstraintTools, constraint_geometry, external_shape,
    good_shape, live_constraint, no_presentation, one_shape, two_shapes,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape, Vertex, Wire};
use rustycad_tkcaf::tdata_xtd::TDataXtdConstraint;
use rustycad_tkcaf::tnaming::TNamingIterator;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCircle, GeomGeometry, GeomLine, GeomPlane};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::int_ana::{QuadQuadGeo, ResultType};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis1, Direction3, Plane3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
use rustycad_tkv3d::prs_dim::{PrsDimAngleDimension, PrsDimDimensionSource, PrsDimLengthDimension};

/// A face of an angle: a wire becomes its planar face (when one can be made), a face stays,
/// a compound gives its first face (`CheckIsShapeCompound`); `None` for another shape.
/// When no face can be made of a wire, natively the variable keeps its `previous` face
/// (null for the first shape, the first face for the second).
fn angle_face(shape: &mut Shape, previous: &Face) -> Result<Option<Face>, Failure> {
    match shape.shape_type()? {
        ShapeEnum::Wire => {
            let mut maker = BRepBuilderAPIMakeFace::from_wire(&topods::wire(shape)?, true)?;
            if !maker.is_done() {
                return Ok(Some(previous.clone()));
            }
            let face = topods::face(maker.shape()?)?;
            *shape = face.clone().into_shape();
            Ok(Some(face))
        }
        ShapeEnum::Face => Ok(Some(topods::face(shape)?)),
        _ => compound_face(shape),
    }
}

/// `CheckIsShapeCompound`: the first face of a compound, which replaces the shape.
fn compound_face(shape: &mut Shape) -> Result<Option<Face>, Failure> {
    if shape.shape_type()? != ShapeEnum::Compound {
        return Ok(None);
    }
    let mut faces = rustycad_tkbrep::toptools::IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, ShapeEnum::Face, &mut faces)?;
    for index in 1..=faces.extent() {
        let face = topods::face(faces.find_key(index)?)?;
        if !face.is_null() {
            *shape = face.clone().into_shape();
            return Ok(Some(face));
        }
    }
    Ok(None)
}

/// The axis of a face of an angle and its plane (planes only): the plane normal, or the
/// axis of a cylinder, cone or torus; `None` for another surface.
fn face_axis(face: &Face) -> Result<Option<(Axis1, SurfaceType, Plane3)>, Failure> {
    let surface = BRepAdaptorSurface::with_face(face, true)?;
    let kind = surface.get_type()?;
    let (axis, plane) = match kind {
        SurfaceType::Plane => {
            let plane = surface.plane()?;
            (plane.axis(), plane)
        }
        SurfaceType::Cylinder => (surface.cylinder()?.axis(), Plane3::default()),
        SurfaceType::Cone => (surface.cone()?.axis(), Plane3::default()),
        SurfaceType::Torus => (surface.torus()?.axis(), Plane3::default()),
        _ => return Ok(None),
    };
    Ok(Some((axis, kind, plane)))
}

/// The geometry an angle without a plane attribute measures in: the line where two
/// non-parallel planes meet (`Ok(None)` when the native code nullifies the object); a
/// null geometry means curvilinear faces (decided by the second face's surface type).
fn computed_angle_geometry(
    first: &mut Shape,
    second: &mut Shape,
) -> Result<Option<Handle<dyn GeomGeometry>>, Failure> {
    let Some(face1) = angle_face(first, &Face::default())? else {
        return Ok(None);
    };
    let Some((axis1, _, plane1)) = face_axis(&face1)? else {
        return Ok(None);
    };
    let Some(face2) = angle_face(second, &face1)? else {
        return Ok(None);
    };
    let Some((axis2, kind2, plane2)) = face_axis(&face2)? else {
        return Ok(None);
    };
    if kind2 != SurfaceType::Plane {
        return Ok(Some(Handle::default()));
    }
    if axis1.is_parallel(axis2, precision::angular()) {
        return Ok(None);
    }
    let mut intersection = QuadQuadGeo::new();
    intersection.perform_planes(plane1, plane2, precision::angular(), precision::angular())?;
    if !intersection.is_done() || intersection.type_inter()? == ResultType::Empty {
        return Ok(None);
    }
    Ok(Some(
        Handle::new(GeomLine::from_lin(intersection.line(1)?)).cast(),
    ))
}

impl TPrsStdConstraintTools {
    /// `ComputeAngleForOneFace`: the `PrsDim_AngleDimension` of a cone face.
    pub fn compute_angle_for_one_face(
        constraint: &Handle<TDataXtdConstraint>,
    ) -> ConstraintPresentation {
        let shape = one_shape(constraint)?;
        if shape.is_null() {
            return no_presentation();
        }
        let (mut value, mut text) = (0.0, ExtendedString::default());
        Self::compute_text_and_value(constraint, &mut value, &mut text, true)?;
        Ok(PrsDimAngleDimension::from_cone(&topods::face(&shape)?)?.cast())
    }
    /// `ComputeAngle`: one geometry is a cone angle; two edges in a plane, or two faces
    /// (planar faces meeting along a line, or curvilinear faces), give a
    /// `PrsDim_AngleDimension`. Natively two planar faces with a plane attribute give no
    /// object.
    pub fn compute_angle(constraint: &Handle<TDataXtdConstraint>) -> ConstraintPresentation {
        let count = live_constraint(constraint)?.nb_geometries();
        if count < 2 {
            return if count == 1 {
                Self::compute_angle_for_one_face(constraint)
            } else {
                no_presentation()
            };
        }
        let (mut first, mut second) = two_shapes(constraint)?;
        let mut geometry = constraint_geometry(constraint)?;
        if first.is_null() || second.is_null() {
            return no_presentation();
        }
        if geometry.is_null() {
            match computed_angle_geometry(&mut first, &mut second)? {
                Some(computed) => geometry = computed,
                None => return no_presentation(),
            }
        }
        let curvilinear = geometry.is_null();
        let mut planar = false;
        if !curvilinear {
            if !geometry.cast::<GeomPlane>().is_null() {
                planar = true;
            } else if geometry.cast::<GeomLine>().is_null() {
                return no_presentation();
            }
        }
        let (mut value, mut text) = (0.0, ExtendedString::default());
        Self::compute_text_and_value(constraint, &mut value, &mut text, true)?;
        let is_face = first.shape_type()? == ShapeEnum::Face;
        if planar {
            if is_face {
                return no_presentation();
            }
            external_shape(constraint)?;
            good_shape(&mut first)?;
            good_shape(&mut second)?;
            let (edge1, edge2) = (topods::edge(&first)?, topods::edge(&second)?);
            return Ok(PrsDimAngleDimension::from_edges(&edge1, &edge2)?.cast());
        }
        if curvilinear || is_face {
            let (face1, face2) = (topods::face(&first)?, topods::face(&second)?);
            return Ok(PrsDimAngleDimension::from_faces(&face1, &face2)?.cast());
        }
        no_presentation()
    }
}

/// The shapes of an offset: the old and new shapes of the first evolution of a single
/// geometry, or the two geometries.
fn offset_shapes(
    constraint: &Handle<TDataXtdConstraint>,
    count: i32,
) -> Result<(Shape, Shape), Failure> {
    if count == 2 {
        return two_shapes(constraint);
    }
    if count != 1 {
        return Ok((Shape::default(), Shape::default()));
    }
    let named = live_constraint(constraint)?.get_geometry(1)?;
    let iterator = TNamingIterator::new(&named)?;
    if !iterator.more() {
        return Ok((Shape::default(), Shape::default()));
    }
    Ok((iterator.old_shape()?, iterator.new_shape()?))
}

/// `BRepAdaptor_Curve::Curve().Curve()->Transformed(Trsf())`: the edge's 3D curve in place.
fn placed_curve(edge: &topods::Edge) -> Result<(CurveType, Handle<dyn GeomGeometry>), Failure> {
    let adaptor = BRepAdaptorCurve::with_edge(edge)?;
    let kind = adaptor.get_type()?;
    let curve = adaptor.transformed_curve().curve();
    let basis = rustycad_tklcaf::live(&curve, "TPrsStd_ConstraintTools::ComputeOffset")?.curve();
    let basis = rustycad_tklcaf::live(&basis, "TPrsStd_ConstraintTools::ComputeOffset")?;
    Ok((kind, basis.transformed(adaptor.transformed_curve().trsf())?))
}

/// The edge-edge offset of a single geometry: two lines give a length dimension on the
/// plane through the second line and the offset direction (returned as done); two
/// circles give the plane of the first circle (returned as the plane to use).
enum EdgeOffset {
    Done(ConstraintPresentation),
    Plane(Handle<GeomPlane>),
    Neither,
}

/// The two-edge case of a single-geometry offset.
fn edge_offset(old: &Shape, new: &Shape) -> Result<EdgeOffset, Failure> {
    let (kind, placed) = placed_curve(&topods::edge(old)?)?;
    if kind == CurveType::Line {
        let old_line = rustycad_tklcaf::live(&placed.cast::<GeomLine>(), "ComputeOffset")?.lin();
        let (_, placed_new) = placed_curve(&topods::edge(new)?)?;
        let new_line =
            rustycad_tklcaf::live(&placed_new.cast::<GeomLine>(), "ComputeOffset")?.lin();
        let offset = Direction3::from_xyz(new_line.location().xyz() - old_line.location().xyz())?;
        let normal = new_line.direction().crossed(offset)?;
        let plane = GeomPlane::from_point(new_line.location(), normal)?;
        let dimension = PrsDimLengthDimension::from_shapes(old, new, &plane.pln())?;
        return Ok(EdgeOffset::Done(Ok(dimension.cast())));
    }
    if kind == CurveType::Circle {
        let circle =
            rustycad_tklcaf::live(&placed.cast::<GeomCircle>(), "ComputeOffset")?.circ()?;
        let axis = circle.axis();
        return Ok(EdgeOffset::Plane(Handle::new(GeomPlane::from_point(
            axis.location(),
            axis.direction(),
        )?)));
    }
    Ok(EdgeOffset::Neither)
}

/// The plane of the face made of every edge of an offset compound (`None` when no planar
/// face can be made: the native code then keeps `is_planar` false).
fn compound_offset_plane(old: &Shape) -> Result<Option<Handle<GeomPlane>>, Failure> {
    let mut wire = Wire::default();
    let builder = BRepBuilder::default();
    builder.make_wire(&mut wire);
    let mut edges = Explorer::new(old, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while edges.more() {
        builder.add(&mut wire, edges.current())?;
        edges.next()?;
    }
    let mut maker = BRepBuilderAPIMakeFace::from_wire(&wire, true)?;
    if !maker.is_done() {
        return Ok(None);
    }
    let face = topods::face(maker.shape()?)?;
    Ok(Some(BRepTool::surface(&face)?.cast::<GeomPlane>()))
}

/// The first vertex of the old shape and the nearest vertex of the new shape.
fn nearest_vertices(old: &Shape, new: &Shape) -> Result<(Shape, Shape), Failure> {
    let first = Explorer::new(old, ShapeEnum::Vertex, ShapeEnum::Shape)?;
    let first = first.current().clone();
    let point = BRepTool::pnt(&topods::vertex(&first)?)?;
    let (mut nearest, mut distance) = (Vertex::default(), f64::MAX);
    let mut vertices = Explorer::new(new, ShapeEnum::Vertex, ShapeEnum::Shape)?;
    while vertices.more() {
        let vertex = topods::vertex(vertices.current())?;
        let current = point.square_distance(&BRepTool::pnt(&vertex)?);
        if current < distance {
            (nearest, distance) = (vertex, current);
        }
        vertices.next()?;
    }
    Ok((first, nearest.into_shape()))
}

impl TPrsStdConstraintTools {
    /// `ComputeOffset`: a `PrsDim_LengthDimension` between two faces, two lines, or the
    /// nearest vertices of the two shapes on the constraint's (or a computed) plane.
    pub fn compute_offset(constraint: &Handle<TDataXtdConstraint>) -> ConstraintPresentation {
        let value_ref = live_constraint(constraint)?;
        let mut planar = value_ref.is_planar();
        let mut plane = Handle::default();
        if planar {
            plane = constraint_geometry(constraint)?.cast::<GeomPlane>();
            if plane.is_null() {
                return no_presentation();
            }
        }
        let count = value_ref.nb_geometries();
        let (old, new) = offset_shapes(constraint, count)?;
        if old.is_null() || new.is_null() {
            return no_presentation();
        }
        let (mut value, mut text) = (0.0, ExtendedString::default());
        if count == 1 {
            Self::compute_text_and_value(constraint, &mut value, &mut text, false)?;
            let (kind1, kind2) = (old.shape_type()?, new.shape_type()?);
            if kind1 == ShapeEnum::Face && kind2 == ShapeEnum::Face {
                let dimension =
                    PrsDimLengthDimension::from_faces(&topods::face(&old)?, &topods::face(&new)?)?;
                if let Some(plane) = plane.get().filter(|_| planar) {
                    dimension.set_custom_plane(&plane.pln());
                }
                return Ok(dimension.cast());
            }
            if kind1 == ShapeEnum::Edge && kind2 == ShapeEnum::Edge {
                match edge_offset(&old, &new)? {
                    EdgeOffset::Done(result) => return result,
                    EdgeOffset::Plane(circle_plane) => (plane, planar) = (circle_plane, true),
                    EdgeOffset::Neither => {}
                }
            }
        }
        let compounds =
            old.shape_type()? == ShapeEnum::Compound && new.shape_type()? == ShapeEnum::Compound;
        if !planar
            && compounds
            && let Some(found) = compound_offset_plane(&old)?
        {
            (plane, planar) = (found, true);
        }
        if !planar {
            return no_presentation();
        }
        Self::compute_text_and_value(constraint, &mut value, &mut text, false)?;
        let (first, nearest) = nearest_vertices(&old, &new)?;
        let pln = rustycad_tklcaf::live(&plane, "TPrsStd_ConstraintTools::ComputeOffset")?.pln();
        Ok(PrsDimLengthDimension::from_shapes(&first, &nearest, &pln)?.cast())
    }
}
