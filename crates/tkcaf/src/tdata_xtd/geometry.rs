// Rust port of ApplicationFramework/TKCAF/TDataXtd/TDataXtd_Geometry.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDataXtd_Geometry`: the geometry type of a label, and the queries that read the
//! geometry of a named shape (point, axis, line, circle, ellipse, plane, cylinder).
use super::{TDataXtd, TDataXtdGeometryEnum};
use crate::tnaming::{TNamingNamedShape, TNamingTool};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Guid, Handle, RuntimeTyped, SharedState, StandardTransient},
};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomCylindricalSurface, GeomEllipse, GeomLine, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdata_std::support::{cast_attribute, find_or_add, set_backed_up};
use rustycad_tklcaf::tdf::{TdfAttribute, TdfAttributeBase, TdfLabel, TdfRelocationTable};
use rustycad_tkmath::gp::{Axis1, Circle3, Cylinder3, Ellipse3, Line3, Plane3, Point3};
use std::io;

/// `TDataXtd_Geometry::GetID()`.
const GEOMETRY_ID: Guid = guid_from_text("2a96b604-ec8b-11d0-bee7-080009dc3333");

/// `TDataXtd_Geometry`.
///
/// ```
/// use rustycad_tkcaf::tdata_xtd::{TDataXtdGeometry, TDataXtdGeometryEnum};
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// let geometry = TDataXtdGeometry::set(&data.root()).unwrap();
/// assert_eq!(geometry.get_type(), TDataXtdGeometryEnum::AnyGeom);
/// ```
pub struct TDataXtdGeometry {
    transient: StandardTransient,
    base: TdfAttributeBase,
    geometry_type: SharedState<TDataXtdGeometryEnum>,
}
impl_transient!(TDataXtdGeometry, "TDataXtd_Geometry", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl Default for TDataXtdGeometry {
    fn default() -> Self {
        Self::new()
    }
}

/// The named shape of a label, null without one.
fn named_shape(label: &TdfLabel) -> Result<Handle<TNamingNamedShape>, Failure> {
    let mut named: Handle<TNamingNamedShape> = Handle::default();
    label.find_attribute_as(&TNamingNamedShape::get_id(), &mut named)?;
    Ok(named)
}

/// The 3d curve of an edge shape, without its trimming; null for another shape.
fn basis_curve(shape: &Shape) -> Result<Handle<dyn GeomCurve>, Failure> {
    if shape.is_null() || shape.shape_type()? != ShapeEnum::Edge {
        return Ok(Handle::default());
    }
    let (curve, _, _) = BRepTool::curve(&topods::edge(shape)?)?;
    if curve.is_instance(GeomTrimmedCurve::static_type()) {
        let trimmed = curve.cast::<GeomTrimmedCurve>();
        return Ok(rustycad_tklcaf::live(&trimmed, "TDataXtd_Geometry")?.basis_curve());
    }
    Ok(curve)
}

/// The surface of a face shape, without its rectangular trimming; null for another shape.
fn basis_surface(shape: &Shape) -> Result<Handle<dyn GeomSurface>, Failure> {
    if shape.is_null() || shape.shape_type()? != ShapeEnum::Face {
        return Ok(Handle::default());
    }
    let surface = BRepTool::surface(&topods::face(shape)?)?;
    if surface.is_instance(GeomRectangularTrimmedSurface::static_type()) {
        let trimmed = surface.cast::<GeomRectangularTrimmedSurface>();
        return Ok(rustycad_tklcaf::live(&trimmed, "TDataXtd_Geometry")?.basis_surface());
    }
    Ok(surface)
}

/// The query of a label: its named shape's, false without one.
fn on_label<T>(
    label: &TdfLabel,
    geometry: &mut T,
    query: fn(&Handle<TNamingNamedShape>, &mut T) -> Result<bool, Failure>,
) -> Result<bool, Failure> {
    let named = named_shape(label)?;
    if named.is_null() {
        return Ok(false);
    }
    query(&named, geometry)
}

impl TDataXtdGeometry {
    /// `Set(label)`: finds or adds the geometry attribute.
    pub fn set(label: &TdfLabel) -> Result<Handle<TDataXtdGeometry>, Failure> {
        find_or_add(label, &Self::get_id(), false, Self::new)
    }
    /// `Type(L)`: the geometry type of the label's named shape (`ANY_GEOM` without one).
    pub fn type_of_label(label: &TdfLabel) -> Result<TDataXtdGeometryEnum, Failure> {
        let named = named_shape(label)?;
        if named.is_null() {
            return Ok(TDataXtdGeometryEnum::AnyGeom);
        }
        Self::type_of(&named)
    }
    /// `Type(S)`: POINT for a vertex, LINE/CIRCLE/ELLIPSE for an edge on such a curve,
    /// CYLINDER/PLANE for a face on such a surface, else `ANY_GEOM`.
    pub fn type_of(named: &Handle<TNamingNamedShape>) -> Result<TDataXtdGeometryEnum, Failure> {
        let shape = TNamingTool::get_shape(named)?;
        Ok(match shape.shape_type()? {
            ShapeEnum::Vertex => TDataXtdGeometryEnum::Point,
            ShapeEnum::Edge => curve_type(&basis_curve(&shape)?),
            ShapeEnum::Face => surface_type(&basis_surface(&shape)?),
            _ => TDataXtdGeometryEnum::AnyGeom,
        })
    }
    /// `Point(L, G)`.
    pub fn point_of_label(label: &TdfLabel, point: &mut Point3) -> Result<bool, Failure> {
        on_label(label, point, Self::point)
    }
    /// `Point(S, G)`: the point of a vertex.
    pub fn point(named: &Handle<TNamingNamedShape>, point: &mut Point3) -> Result<bool, Failure> {
        let shape = TNamingTool::get_shape(named)?;
        if shape.is_null() || shape.shape_type()? != ShapeEnum::Vertex {
            return Ok(false);
        }
        *point = BRepTool::pnt(&topods::vertex(&shape)?)?;
        Ok(true)
    }
    /// `Axis(L, G)`.
    pub fn axis_of_label(label: &TdfLabel, axis: &mut Axis1) -> Result<bool, Failure> {
        on_label(label, axis, Self::axis)
    }
    /// `Axis(S, G)`: the position of the line.
    pub fn axis(named: &Handle<TNamingNamedShape>, axis: &mut Axis1) -> Result<bool, Failure> {
        let mut line = Line3::default();
        if !Self::line(named, &mut line)? {
            return Ok(false);
        }
        *axis = line.position();
        Ok(true)
    }
    /// `Line(L, G)`.
    pub fn line_of_label(label: &TdfLabel, line: &mut Line3) -> Result<bool, Failure> {
        on_label(label, line, Self::line)
    }
    /// `Line(S, G)`: the line of an edge.
    pub fn line(named: &Handle<TNamingNamedShape>, line: &mut Line3) -> Result<bool, Failure> {
        let curve = basis_curve(&TNamingTool::get_shape(named)?)?.cast::<GeomLine>();
        let Some(curve) = curve.get() else {
            return Ok(false);
        };
        *line = curve.lin();
        Ok(true)
    }
    /// `Circle(L, G)`.
    pub fn circle_of_label(label: &TdfLabel, circle: &mut Circle3) -> Result<bool, Failure> {
        on_label(label, circle, Self::circle)
    }
    /// `Circle(S, G)`: the circle of an edge.
    pub fn circle(
        named: &Handle<TNamingNamedShape>,
        circle: &mut Circle3,
    ) -> Result<bool, Failure> {
        let curve = basis_curve(&TNamingTool::get_shape(named)?)?.cast::<GeomCircle>();
        let Some(curve) = curve.get() else {
            return Ok(false);
        };
        *circle = curve.circ()?;
        Ok(true)
    }
    /// `Ellipse(L, G)`.
    pub fn ellipse_of_label(label: &TdfLabel, ellipse: &mut Ellipse3) -> Result<bool, Failure> {
        on_label(label, ellipse, Self::ellipse)
    }
    /// `Ellipse(S, G)`: the ellipse of an edge.
    pub fn ellipse(
        named: &Handle<TNamingNamedShape>,
        ellipse: &mut Ellipse3,
    ) -> Result<bool, Failure> {
        let curve = basis_curve(&TNamingTool::get_shape(named)?)?.cast::<GeomEllipse>();
        let Some(curve) = curve.get() else {
            return Ok(false);
        };
        *ellipse = curve.elips()?;
        Ok(true)
    }
    /// `Plane(L, G)`.
    pub fn plane_of_label(label: &TdfLabel, plane: &mut Plane3) -> Result<bool, Failure> {
        on_label(label, plane, Self::plane)
    }
    /// `Plane(S, G)`: the plane of a face.
    pub fn plane(named: &Handle<TNamingNamedShape>, plane: &mut Plane3) -> Result<bool, Failure> {
        let surface = basis_surface(&TNamingTool::get_shape(named)?)?.cast::<GeomPlane>();
        let Some(surface) = surface.get() else {
            return Ok(false);
        };
        *plane = surface.pln();
        Ok(true)
    }
    /// `Cylinder(L, G)`.
    pub fn cylinder_of_label(label: &TdfLabel, cylinder: &mut Cylinder3) -> Result<bool, Failure> {
        on_label(label, cylinder, Self::cylinder)
    }
    /// `Cylinder(S, G)`: the cylinder of a face.
    pub fn cylinder(
        named: &Handle<TNamingNamedShape>,
        cylinder: &mut Cylinder3,
    ) -> Result<bool, Failure> {
        let surface =
            basis_surface(&TNamingTool::get_shape(named)?)?.cast::<GeomCylindricalSurface>();
        let Some(surface) = surface.get() else {
            return Ok(false);
        };
        *cylinder = surface.cylinder()?;
        Ok(true)
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        GEOMETRY_ID
    }
    /// `TDataXtd_Geometry()`: `ANY_GEOM`.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            geometry_type: SharedState::new(TDataXtdGeometryEnum::AnyGeom),
        }
    }
    /// `SetType(T)`: backed up when it changes.
    pub fn set_type(&self, geometry_type: TDataXtdGeometryEnum) -> Result<(), Failure> {
        set_backed_up(self, &self.geometry_type, &geometry_type)
    }
    /// `GetType`.
    pub fn get_type(&self) -> TDataXtdGeometryEnum {
        *self.geometry_type.read()
    }
}

/// The type of an edge's basis curve (exact classes, `IsInstance`).
fn curve_type(curve: &Handle<dyn GeomCurve>) -> TDataXtdGeometryEnum {
    if curve.is_instance(GeomLine::static_type()) {
        TDataXtdGeometryEnum::Line
    } else if curve.is_instance(GeomCircle::static_type()) {
        TDataXtdGeometryEnum::Circle
    } else if curve.is_instance(GeomEllipse::static_type()) {
        TDataXtdGeometryEnum::Ellipse
    } else {
        TDataXtdGeometryEnum::AnyGeom
    }
}

/// The type of a face's basis surface (exact classes, `IsInstance`).
fn surface_type(surface: &Handle<dyn GeomSurface>) -> TDataXtdGeometryEnum {
    if surface.is_instance(GeomCylindricalSurface::static_type()) {
        TDataXtdGeometryEnum::Cylinder
    } else if surface.is_instance(GeomPlane::static_type()) {
        TDataXtdGeometryEnum::Plane
    } else {
        TDataXtdGeometryEnum::AnyGeom
    }
}

impl TdfAttribute for TDataXtdGeometry {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        GEOMETRY_ID
    }
    fn restore(&self, attribute: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<TDataXtdGeometry>(attribute, "TDataXtd_Geometry::Restore")?;
        let geometry_type = rustycad_tklcaf::live(&other, "TDataXtd_Geometry::Restore")?.get_type();
        *self.geometry_type.write() = geometry_type;
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<TDataXtdGeometry>(into, "TDataXtd_Geometry::Paste")?;
        rustycad_tklcaf::live(&target, "TDataXtd_Geometry::Paste")?.set_type(self.get_type())
    }
    /// `Dump`: `Geometry ` and the type name.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        write!(stream, "Geometry ")?;
        TDataXtd::print_geometry(self.get_type(), stream)
    }
}
