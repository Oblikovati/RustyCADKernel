// Rust port of ApplicationFramework/TKCAF/TDataXtd/TDataXtd_Point, TDataXtd_Axis,
// TDataXtd_Plane, TDataXtd_Placement and TDataXtd_Shape (.hxx and .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction-geometry attributes: content-free attributes (TDataStd_GenericEmpty)
//! whose geometry is the TNaming named shape of their label.
use super::TDataXtd;
use crate::tnaming::{TNamingBuilder, TNamingNamedShape, TNamingTool};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::{Failure, Handle, exceptions};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_lib::IsPlanarSurface;
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdata_std::support::find_or_add;
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Line3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex,
};

rustycad_tklcaf::tdata_std_empty_attribute!(
    /// `TDataXtd_Point`: a construction point, the vertex of the label's named shape.
    TDataXtdPoint, "TDataXtd_Point", guid_from_text("2a96b60d-ec8b-11d0-bee7-080009dc3333"), "Point",
    on_create: TDataXtd::register_derived_attributes
);
rustycad_tklcaf::tdata_std_empty_attribute!(
    /// `TDataXtd_Axis`: a construction axis, the line edge of the label's named shape.
    TDataXtdAxis, "TDataXtd_Axis", guid_from_text("2a96b601-ec8b-11d0-bee7-080009dc3333"), "Axis",
    on_create: TDataXtd::register_derived_attributes
);
rustycad_tklcaf::tdata_std_empty_attribute!(
    /// `TDataXtd_Plane`: a construction plane, the planar face of the label's named shape.
    TDataXtdPlane, "TDataXtd_Plane", guid_from_text("2a96b60c-ec8b-11d0-bee7-080009dc3333"), "Plane",
    on_create: TDataXtd::register_derived_attributes
);
rustycad_tklcaf::tdata_std_empty_attribute!(
    /// `TDataXtd_Placement`: marks a placement label.
    TDataXtdPlacement, "TDataXtd_Placement", guid_from_text("2a96b60b-ec8b-11d0-bee7-080009dc3333"), "Placement",
    on_create: TDataXtd::register_derived_attributes
);
rustycad_tklcaf::tdata_std_empty_attribute!(
    /// `TDataXtd_Shape`: a shape attribute over the label's named shape (its `References`
    /// adds nothing, as the inherited default).
    TDataXtdShape, "TDataXtd_Shape", guid_from_text("2a96b620-ec8b-11d0-bee7-080009dc3333"), "Shape",
    on_create: TDataXtd::register_derived_attributes
);

/// The current shape of the label's named shape (`aNS->Get()`), null without one.
fn named_shape_of(label: &TdfLabel) -> Result<Shape, Failure> {
    let mut named: Handle<TNamingNamedShape> = Handle::default();
    if !label.find_attribute_as(&TNamingNamedShape::get_id(), &mut named)? {
        return Ok(Shape::default());
    }
    rustycad_tklcaf::live(&named, "TDataXtd")?.get()
}

/// `TNaming_Builder(L).Generated(shape)`.
fn generate(label: &TdfLabel, shape: &Shape) -> Result<(), Failure> {
    TNamingBuilder::new(label)?.generated(shape)
}

impl TDataXtdPoint {
    /// `Set(label)`: finds or adds the point.
    pub fn set(label: &TdfLabel) -> Result<Handle<TDataXtdPoint>, Failure> {
        find_or_add(label, &Self::get_id(), false, Self::new)
    }
    /// `Set(label, P)`: the point with a new vertex at `P` as the label's named shape, unless
    /// the named shape is already a vertex there (OCC2932).
    pub fn set_point(label: &TdfLabel, point: Point3) -> Result<Handle<TDataXtdPoint>, Failure> {
        let attribute = Self::set(label)?;
        let shape = named_shape_of(label)?;
        if !shape.is_null() && shape.shape_type()? == ShapeEnum::Vertex {
            let old = BRepTool::pnt(&topods::vertex(&shape)?)?;
            if old.x() == point.x() && old.y() == point.y() && old.z() == point.z() {
                return Ok(attribute);
            }
        }
        let vertex = BRepBuilderAPIMakeVertex::new(point)?.vertex()?;
        generate(label, &vertex.into_shape())?;
        Ok(attribute)
    }
}

impl TDataXtdAxis {
    /// `Set(label)`: finds or adds the axis.
    pub fn set(label: &TdfLabel) -> Result<Handle<TDataXtdAxis>, Failure> {
        find_or_add(label, &Self::get_id(), false, Self::new)
    }
    /// `Set(label, line)`: the axis with a new edge on the line as the label's named shape,
    /// unless the named shape is already an edge on the same line (OCC2932).
    pub fn set_line(label: &TdfLabel, line: &Line3) -> Result<Handle<TDataXtdAxis>, Failure> {
        let attribute = Self::set(label)?;
        let shape = named_shape_of(label)?;
        if !shape.is_null() && shape.shape_type()? == ShapeEnum::Edge && same_line(&shape, line)? {
            return Ok(attribute);
        }
        let edge = BRepBuilderAPIMakeEdge::from_lin(*line)?.edge()?;
        generate(label, &edge.into_shape())?;
        Ok(attribute)
    }
}

/// The OCC2932 test of `TDataXtd_Axis::Set`: the edge's adaptor is a line with the same
/// direction and location coordinates.
fn same_line(edge: &Shape, line: &Line3) -> Result<bool, Failure> {
    let adaptor = BRepAdaptorCurve::with_edge(&topods::edge(edge)?)?;
    if adaptor.get_type()? != CurveType::Line {
        return Ok(false);
    }
    let old = adaptor.line()?;
    let (old_direction, direction) = (old.direction(), line.direction());
    let (old_location, location) = (old.location(), line.location());
    Ok(old_direction.x() == direction.x()
        && old_direction.y() == direction.y()
        && old_direction.z() == direction.z()
        && old_location.x() == location.x()
        && old_location.y() == location.y()
        && old_location.z() == location.z())
}

impl TDataXtdPlane {
    /// `Set(label)`: finds or adds the plane.
    pub fn set(label: &TdfLabel) -> Result<Handle<TDataXtdPlane>, Failure> {
        find_or_add(label, &Self::get_id(), false, Self::new)
    }
    /// `Set(label, P)`: the plane with a new face on `P` as the label's named shape, unless
    /// the named shape is already a planar face on the same plane (OCC2932).
    pub fn set_plane(label: &TdfLabel, plane: &Plane3) -> Result<Handle<TDataXtdPlane>, Failure> {
        let attribute = Self::set(label)?;
        let shape = named_shape_of(label)?;
        if !shape.is_null() && shape.shape_type()? == ShapeEnum::Face && same_plane(&shape, plane)?
        {
            return Ok(attribute);
        }
        let face = BRepBuilderAPIMakeFace::from_plane(*plane)?.face()?;
        generate(label, &face.into_shape())?;
        Ok(attribute)
    }
}

/// The OCC2932 test of `TDataXtd_Plane::Set`: the face's surface is planar (default
/// tolerance) with the same location and axis coordinates.
fn same_plane(face: &Shape, plane: &Plane3) -> Result<bool, Failure> {
    let surface = BRepTool::surface(&topods::face(face)?)?;
    let checker = IsPlanarSurface::new(&surface, 1.0e-7)?;
    if !checker.is_planar() {
        return Ok(false);
    }
    let old = checker.plan()?;
    let coordinates = |p: &Plane3| {
        let axis = p.axis();
        let (location, axis_location, direction) =
            (p.location(), axis.location(), axis.direction());
        [
            location.x(),
            location.y(),
            location.z(),
            axis_location.x(),
            axis_location.y(),
            axis_location.z(),
            direction.x(),
            direction.y(),
            direction.z(),
        ]
    };
    Ok(coordinates(&old) == coordinates(plane))
}

impl TDataXtdPlacement {
    /// `Set(label)`: finds or adds the placement.
    pub fn set(label: &TdfLabel) -> Result<Handle<TDataXtdPlacement>, Failure> {
        find_or_add(label, &Self::get_id(), false, Self::new)
    }
}

impl TDataXtdShape {
    /// `Find(current, S)`: the shape attribute of the label or of its closest ancestor.
    pub fn find(current: &TdfLabel, shape: &mut Handle<TDataXtdShape>) -> Result<bool, Failure> {
        let mut label = current.clone();
        while !label.is_null() {
            let mut found: Handle<TDataXtdShape> = Handle::default();
            if label.find_attribute_as(&Self::get_id(), &mut found)? {
                *shape = found;
                return Ok(true);
            }
            label = label.father();
        }
        Ok(false)
    }
    /// `New(label)`: a shape attribute on an empty label (`Standard_DomainError` otherwise).
    pub fn new_on(label: &TdfLabel) -> Result<Handle<TDataXtdShape>, Failure> {
        if label.has_attribute()? {
            return Err(exceptions::domain_error(
                "TDataXtd_Shape::New : not an empty label",
            ));
        }
        let attribute = Handle::new(Self::new());
        label.add_attribute(&attribute.cast(), true)?;
        Ok(attribute)
    }
    /// `Set(label, shape)`: the shape attribute (created on an empty label) with `shape` as
    /// the label's named shape, unless it already is (OCC2932, `IsEqual`).
    pub fn set(label: &TdfLabel, shape: &Shape) -> Result<Handle<TDataXtdShape>, Failure> {
        let mut attribute: Handle<TDataXtdShape> = Handle::default();
        if !label.find_attribute_as(&Self::get_id(), &mut attribute)? {
            attribute = Self::new_on(label)?;
        }
        let current = named_shape_of(label)?;
        if !current.is_null() && current == *shape {
            return Ok(attribute);
        }
        generate(label, shape)?;
        Ok(attribute)
    }
    /// `Get(label)`: the shape of the label's named shape (`TNaming_Tool::GetShape`), null
    /// without one.
    pub fn get(label: &TdfLabel) -> Result<Shape, Failure> {
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        if label.find_attribute_as(&TNamingNamedShape::get_id(), &mut named)? {
            return TNamingTool::get_shape(&named);
        }
        Ok(Shape::default())
    }
}
