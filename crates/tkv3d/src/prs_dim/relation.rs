//! `PrsDim_Relation`: the abstract relation between shapes (parallel, tangent, equal ...),
//! a trait over embedded base state with the shapes, plane, value, position and text.
use super::tool_presentation::{call_line, projected_edge, set_point_style, set_wire_style};
use super::{PrsDimKindOfDimension, PrsDimKindOfSurface, PrsDimProjectionStyle};
use crate::ais::{
    AisGraphicTool, AisInteractiveObject, AisInteractiveObjectBase, AisTypeOfAttribute,
};
use crate::dsg_prs::DsgPrsArrowSide;
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrTypeOfPresentation3d};
use crate::prs3d::{Prs3dDimensionAspect, Prs3dLineAspect, Prs3dTextAspect};
use crate::std_prs::StdPrsWfShape;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{self, Plane3, Point3};
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkservice::graphic3d::*;

/// The protected fields of `PrsDim_Relation`.
#[derive(Clone)]
pub struct PrsDimRelationFields {
    /// `myFShape`.
    pub f_shape: Shape,
    /// `mySShape`.
    pub s_shape: Shape,
    /// `myPlane`.
    pub plane: Handle<GeomPlane>,
    /// `myVal`.
    pub val: f64,
    /// `myPosition`.
    pub position: Point3,
    /// `myText`.
    pub text: ExtendedString,
    /// `myArrowSize`.
    pub arrow_size: f64,
    /// `myAutomaticPosition`.
    pub automatic_position: bool,
    /// `mySymbolPrs`.
    pub symbol_prs: DsgPrsArrowSide,
    /// `myExtShape`.
    pub ext_shape: i32,
    /// `myFirstPlane`.
    pub first_plane: Plane3,
    /// `mySecondPlane`.
    pub second_plane: Plane3,
    /// `myFirstBasisSurf`.
    pub first_basis_surf: Handle<dyn GeomSurface>,
    /// `mySecondBasisSurf`.
    pub second_basis_surf: Handle<dyn GeomSurface>,
    /// `myFirstSurfType`.
    pub first_surf_type: PrsDimKindOfSurface,
    /// `mySecondSurfType`.
    pub second_surf_type: PrsDimKindOfSurface,
    /// `myFirstOffset`.
    pub first_offset: f64,
    /// `mySecondOffset`.
    pub second_offset: f64,
    /// `myBndBox`.
    pub bnd_box: BoundingBox3,
    /// `myIsSetBndBox`.
    pub is_set_bnd_box: bool,
    /// `myArrowSizeIsDefined`.
    pub arrow_size_is_defined: bool,
}

/// The embedded base of every relation; `PrsDimRelationBase::new(PrsMgrTypeOfPresentation3d::AllView)`.
pub struct PrsDimRelationBase {
    /// The interactive object base.
    pub interactive: AisInteractiveObjectBase,
    /// The protected fields.
    pub fields: SharedState<PrsDimRelationFields>,
}
impl PrsDimRelationBase {
    /// `PrsDim_Relation(type)`: value 1, arrow size 0.1, automatic position.
    /// Example: `PrsDimRelationBase::new(PrsMgrTypeOfPresentation3d::AllView)`.
    pub fn new(kind: PrsMgrTypeOfPresentation3d) -> Self {
        let val = 1.0;
        Self {
            interactive: AisInteractiveObjectBase::new(kind),
            fields: SharedState::new(PrsDimRelationFields {
                f_shape: Shape::default(),
                s_shape: Shape::default(),
                plane: Handle::default(),
                val,
                position: Point3::new(0.0, 0.0, 0.0),
                text: ExtendedString::default(),
                arrow_size: val / 10.0,
                automatic_position: true,
                symbol_prs: DsgPrsArrowSide::None,
                ext_shape: 0,
                first_plane: Plane3::default(),
                second_plane: Plane3::default(),
                first_basis_surf: Handle::default(),
                second_basis_surf: Handle::default(),
                first_surf_type: PrsDimKindOfSurface::Plane,
                second_surf_type: PrsDimKindOfSurface::Plane,
                first_offset: 0.0,
                second_offset: 0.0,
                bnd_box: BoundingBox3::default(),
                is_set_bnd_box: false,
                arrow_size_is_defined: false,
            }),
        }
    }
}

/// `PrsDim_Relation` (abstract): the virtual members with their native default bodies.
pub trait PrsDimRelationSource: AisInteractiveObject {
    /// The embedded base state.
    fn relation_base(&self) -> &PrsDimRelationBase;
    /// `KindOfDimension`: none by default; `relation.kind_of_dimension()`.
    fn kind_of_dimension(&self) -> PrsDimKindOfDimension {
        PrsDimKindOfDimension::None
    }
    /// `IsMovable`: false by default; `relation.is_movable()`.
    fn is_movable(&self) -> bool {
        false
    }
    /// `SetFirstShape`; `relation.set_first_shape(&shape)`.
    fn set_first_shape(&self, shape: &Shape) {
        self.relation_base().fields.write().f_shape = shape.clone();
    }
    /// `SetSecondShape`; `relation.set_second_shape(&shape)`.
    fn set_second_shape(&self, shape: &Shape) {
        self.relation_base().fields.write().s_shape = shape.clone();
    }
}

rustycad_tkernel::impl_transient_interface!(
    dyn PrsDimRelationSource,
    "PrsDim_Relation",
    dyn AisInteractiveObject
);

/// The non-virtual members of `PrsDim_Relation`.
pub trait PrsDimRelationMembers: PrsDimRelationSource {
    /// `FirstShape`; `relation.first_shape()`.
    fn first_shape(&self) -> Shape {
        self.relation_base().fields.read().f_shape.clone()
    }
    /// `SecondShape`; `relation.second_shape()`.
    fn second_shape(&self) -> Shape {
        self.relation_base().fields.read().s_shape.clone()
    }
    /// `SetBndBox`: a bounding box for the selection; `relation.set_bnd_box(0., 0., 0., 1., 1., 1.)`.
    fn set_bnd_box(&self, xmin: f64, ymin: f64, zmin: f64, xmax: f64, ymax: f64, zmax: f64) {
        let mut f = self.relation_base().fields.write();
        f.bnd_box
            .update_bounds([xmin, ymin, zmin], [xmax, ymax, zmax]);
        f.is_set_bnd_box = true;
    }
    /// `UnsetBndBox`; `relation.unset_bnd_box()`.
    fn unset_bnd_box(&self) {
        self.relation_base().fields.write().is_set_bnd_box = false;
    }
    /// `Plane`; `relation.plane()`.
    fn plane(&self) -> Handle<GeomPlane> {
        self.relation_base().fields.read().plane.clone()
    }
    /// `SetPlane`; `relation.set_plane(&plane)`.
    fn set_plane(&self, plane: &Handle<GeomPlane>) {
        self.relation_base().fields.write().plane = plane.clone();
    }
    /// `Value`; `relation.value()`.
    fn value(&self) -> f64 {
        self.relation_base().fields.read().val
    }
    /// `SetValue`; `relation.set_value(2.0)`.
    fn set_value(&self, value: f64) {
        self.relation_base().fields.write().val = value;
    }
    /// `Position`; `relation.position()`.
    fn position(&self) -> Point3 {
        self.relation_base().fields.read().position
    }
    /// `SetPosition`: a position, no longer automatic; `relation.set_position(&point)`.
    fn set_position(&self, position: &Point3) {
        let mut f = self.relation_base().fields.write();
        f.position = *position;
        f.automatic_position = false;
    }
    /// `Text`; `relation.text()`.
    fn text(&self) -> ExtendedString {
        self.relation_base().fields.read().text.clone()
    }
    /// `SetText`; `relation.set_text(&text)`.
    fn set_text(&self, text: &ExtendedString) {
        self.relation_base().fields.write().text = text.clone();
    }
    /// `ArrowSize`; `relation.arrow_size()`.
    fn arrow_size(&self) -> f64 {
        self.relation_base().fields.read().arrow_size
    }
    /// `SetArrowSize`: a defined arrow size; `relation.set_arrow_size(2.0)`.
    fn set_arrow_size(&self, size: f64) {
        let mut f = self.relation_base().fields.write();
        f.arrow_size = size;
        f.arrow_size_is_defined = true;
    }
    /// `SymbolPrs`; `relation.symbol_prs()`.
    fn symbol_prs(&self) -> DsgPrsArrowSide {
        self.relation_base().fields.read().symbol_prs
    }
    /// `SetSymbolPrs`; `relation.set_symbol_prs(DsgPrsArrowSide::BothAr)`.
    fn set_symbol_prs(&self, symbol: DsgPrsArrowSide) {
        self.relation_base().fields.write().symbol_prs = symbol;
    }
    /// `SetExtShape`; `relation.set_ext_shape(1)`.
    fn set_ext_shape(&self, index: i32) {
        self.relation_base().fields.write().ext_shape = index;
    }
    /// `ExtShape`; `relation.ext_shape()`.
    fn ext_shape(&self) -> i32 {
        self.relation_base().fields.read().ext_shape
    }
    /// `SetAutomaticPosition`; `relation.set_automatic_position(true)`.
    fn set_automatic_position(&self, status: bool) {
        self.relation_base().fields.write().automatic_position = status;
    }
    /// `AutomaticPosition`; `relation.automatic_position()`.
    fn automatic_position(&self) -> bool {
        self.relation_base().fields.read().automatic_position
    }

    /// `ComputeProjEdgePresentation`: the projected edge in the projection style, then the
    /// call lines (or points) from its ends to the edge ends.
    /// Example: `relation.compute_proj_edge_presentation(&prs, &edge, &curve, a, b, &style)?`.
    fn compute_proj_edge_presentation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        edge: &Edge,
        projected: &Handle<dyn GeomCurve>,
        first: Point3,
        last: Point3,
        style: &PrsDimProjectionStyle,
    ) -> Result<(), Failure> {
        let drawer = self.attributes();
        let value = drawer.get().ok_or_else(super::tool_geometry::null_curve)?;
        set_wire_style(value, style.color, style.projection_line, style.width)?;
        let found = BRepTool::curve_with_location(edge)?;
        let infinite = precision::is_infinite(found.first) || precision::is_infinite(found.last);
        let shape = projected_edge(projected, (first, last), infinite)?;
        StdPrsWfShape::add(prs, &shape, &drawer)?;
        value.wire_aspect().set_type_of_line(style.call_line);
        if infinite {
            return Ok(());
        }
        let first_end = BRepTool::pnt(&topexp::first_vertex(edge, false))?;
        let last_end = BRepTool::pnt(&topexp::last_vertex(edge, false))?;
        let far = |a: &Point3, b: &Point3| a.distance(b) > gp::resolution();
        call_line(prs, &drawer, first, first_end, far(&first, &first_end))?;
        call_line(prs, &drawer, last, last_end, far(&last, &last_end))
    }

    /// `ComputeProjVertexPresentation`: the projected point marker, then the call line to the
    /// vertex; `relation.compute_proj_vertex_presentation(&prs, &vertex, &point, &style)?`.
    fn compute_proj_vertex_presentation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        vertex: &Vertex,
        projected: &Point3,
        style: &PrsDimProjectionStyle,
    ) -> Result<(), Failure> {
        let drawer = self.attributes();
        let value = drawer.get().ok_or_else(super::tool_geometry::null_curve)?;
        set_point_style(value, style.color, style.projection_marker)?;
        let group = prs.new_group()?;
        let points = Handle::new(Graphic3dArrayOfPoints::new(1, GRAPHIC3D_ARRAY_FLAGS_NONE)?);
        points.add_vertex_pnt(projected)?;
        group.set_group_primitives_aspect(&value.point_aspect().aspect().cast());
        group.add_primitive_array(&points.cast(), true);
        set_wire_style(value, style.color, style.call_line, style.width)?;
        let original = BRepTool::pnt(vertex)?;
        if !projected.is_equal(&original, precision::confusion()) {
            let group = prs.new_group()?;
            let lines = Handle::new(Graphic3dArrayOfSegments::new(
                2,
                0,
                GRAPHIC3D_ARRAY_FLAGS_NONE,
            )?);
            lines.add_vertex_pnt(projected)?;
            lines.add_vertex_pnt(&original)?;
            group.set_group_primitives_aspect(&value.wire_aspect().aspect().cast());
            group.add_primitive_array(&lines.cast(), true);
        }
        Ok(())
    }
}
impl<T: PrsDimRelationSource + ?Sized> PrsDimRelationMembers for T {}

/// `PrsDim_Relation::SetColor`: the drawer, text, line and dimension aspects in a color.
/// Example: `prs_dim_relation_set_color(&relation, color)?`.
pub fn prs_dim_relation_set_color<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
    color: Color,
) -> Result<(), Failure> {
    let drawer = relation.attributes();
    let value = drawer.get().ok_or_else(super::tool_geometry::null_curve)?;
    let has_own = relation.presentable_base().read().has_own_color;
    if has_own && value.color() == color {
        return Ok(());
    }
    if !value.has_own_text_aspect() {
        value.set_text_aspect(Handle::new(Prs3dTextAspect::new()));
    }
    relation.presentable_base().fields.write().has_own_color = true;
    value.set_color(color);
    value.text_aspect().set_color(color);
    let width = if relation.has_width() {
        relation.width()
    } else if value.has_link() {
        let link = value.link();
        let link = link.get().ok_or_else(super::tool_geometry::null_curve)?;
        AisGraphicTool::get_line_width(link, AisTypeOfAttribute::Line)?
    } else {
        1.0
    };
    if !value.has_own_line_aspect() {
        value.set_line_aspect(Handle::new(Prs3dLineAspect::new(
            color,
            AspectTypeOfLine::Solid,
            width,
        )?));
    }
    if !value.has_own_dimension_aspect() {
        value.set_dimension_aspect(Handle::new(Prs3dDimensionAspect::new()?));
    }
    value.line_aspect().set_color(color);
    let dimension = value.dimension_aspect();
    dimension.set_line_aspect(value.line_aspect());
    dimension.set_text_aspect(value.text_aspect());
    Ok(())
}

/// `PrsDim_Relation::UnsetColor`: the line back to the link color (yellow without a link).
/// Example: `prs_dim_relation_unset_color(&relation)?`.
pub fn prs_dim_relation_unset_color<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
) -> Result<(), Failure> {
    if !relation.presentable_base().read().has_own_color {
        return Ok(());
    }
    relation.presentable_base().fields.write().has_own_color = false;
    let drawer = relation.attributes();
    let value = drawer.get().ok_or_else(super::tool_geometry::null_curve)?;
    let line = value.line_aspect();
    let mut color = Color::named(NamedColor::YELLOW);
    if value.has_link() {
        let link = value.link();
        let link = link.get().ok_or_else(super::tool_geometry::null_curve)?;
        color = AisGraphicTool::get_line_color_value(link, AisTypeOfAttribute::Line)?;
        value.set_text_aspect(link.text_aspect());
    }
    line.set_color(color);
    value.dimension_aspect().set_line_aspect(line);
    Ok(())
}
