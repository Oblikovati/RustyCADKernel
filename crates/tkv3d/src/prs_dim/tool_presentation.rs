//! `PrsDim::ComputeProjEdgePresentation` and `ComputeProjVertexPresentation`: an edge or
//! vertex projected on the dimension plane, drawn with call lines to the original.
use super::PrsDim;
use super::tool_geometry::SQUARE_TOLERANCE;
use crate::prs3d::{Prs3dDrawer, Prs3dLineAspect, Prs3dPointAspect};
use crate::std_prs::{StdPrsPoint, StdPrsWfShape};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomCircle, GeomCurve, GeomLine};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::{AspectTypeOfLine, AspectTypeOfMarker};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeVertex};

/// The style of a projected edge or vertex: its color, width, projection line or marker and
/// call line types; the native defaults are purple, 2, dash or plus, and dot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrsDimProjectionStyle {
    /// `aColor`.
    pub color: NamedColor,
    /// `aWidth`.
    pub width: f64,
    /// `aProjTOL` (edges).
    pub projection_line: AspectTypeOfLine,
    /// `aProjTOM` (vertices).
    pub projection_marker: AspectTypeOfMarker,
    /// `aCallTOL`.
    pub call_line: AspectTypeOfLine,
}
impl Default for PrsDimProjectionStyle {
    fn default() -> Self {
        Self {
            color: NamedColor::PURPLE,
            width: 2.0,
            projection_line: AspectTypeOfLine::Dash,
            projection_marker: AspectTypeOfMarker::Plus,
            call_line: AspectTypeOfLine::Dot,
        }
    }
}

/// The wire aspect of the drawer set to a color, line type and width.
pub(super) fn set_wire_style(
    drawer: &Prs3dDrawer,
    color: NamedColor,
    kind: AspectTypeOfLine,
    width: f64,
) -> Result<(), Failure> {
    if !drawer.has_own_wire_aspect() {
        drawer.set_wire_aspect(Handle::new(Prs3dLineAspect::new(
            Color::named(color),
            kind,
            2.0,
        )?));
        return Ok(());
    }
    let aspect = drawer.wire_aspect();
    aspect.set_color(Color::named(color));
    aspect.set_type_of_line(kind);
    aspect.set_width(width)
}

/// The point aspect of the drawer set to a color and marker.
pub(super) fn set_point_style(
    drawer: &Prs3dDrawer,
    color: NamedColor,
    marker: AspectTypeOfMarker,
) -> Result<(), Failure> {
    if !drawer.has_own_point_aspect() {
        drawer.set_point_aspect(Handle::new(Prs3dPointAspect::new(
            marker,
            Color::named(color),
            1.0,
        )?));
        return Ok(());
    }
    let aspect = drawer.point_aspect();
    aspect.set_color(Color::named(color));
    aspect.set_type_of_marker(marker);
    Ok(())
}

/// The projected edge of a line (bounded unless infinite) or circle curve.
pub(super) fn projected_edge(
    curve: &Handle<dyn GeomCurve>,
    ends: (Point3, Point3),
    infinite: bool,
) -> Result<Shape, Failure> {
    if let Some(line) = curve.cast::<GeomLine>().get() {
        let lin = line.lin();
        let maker = if infinite {
            BRepBuilderAPIMakeEdge::from_lin(lin)?
        } else {
            let first = elclib::parameter(&lin, ends.0)?;
            let last = elclib::parameter(&lin, ends.1)?;
            BRepBuilderAPIMakeEdge::from_lin_params(lin, first, last)?
        };
        return Ok(maker.edge()?.into());
    }
    if let Some(circle) = curve.cast::<GeomCircle>().get() {
        let circ = circle.circ()?;
        let first = elclib::parameter(&circ, ends.0)?;
        let last = elclib::parameter(&circ, ends.1)?;
        return Ok(BRepBuilderAPIMakeEdge::from_circ_params(circ, first, last)?
            .edge()?
            .into());
    }
    Ok(Shape::default())
}

/// A call line from a projected end to the edge end (a vertex when they coincide within
/// `limit`, compared by `distance`).
pub(super) fn call_line(
    prs: &Handle<dyn Graphic3dStructureSource>,
    drawer: &Handle<Prs3dDrawer>,
    projected: Point3,
    end: Point3,
    far: bool,
) -> Result<(), Failure> {
    let shape: Shape = if far {
        BRepBuilderAPIMakeEdge::from_points(projected, end)?
            .edge()?
            .into()
    } else {
        BRepBuilderAPIMakeVertex::new(projected)?.vertex()?.into()
    };
    StdPrsWfShape::add(prs, &shape, drawer)
}

impl PrsDim {
    /// `ComputeProjEdgePresentation`: the projected edge in the projection style, then the
    /// call lines from its ends to the edge ends; `PrsDim::compute_proj_edge_presentation(...)?`.
    pub fn compute_proj_edge_presentation(
        prs: &Handle<dyn Graphic3dStructureSource>,
        drawer: &Handle<Prs3dDrawer>,
        edge: &Edge,
        projected: &Handle<dyn GeomCurve>,
        ends: (Point3, Point3),
        style: &PrsDimProjectionStyle,
    ) -> Result<(), Failure> {
        let value = drawer.get().ok_or_else(super::tool_geometry::null_curve)?;
        set_wire_style(value, style.color, style.projection_line, style.width)?;
        let found = BRepTool::curve_with_location(edge)?;
        let infinite = precision::is_infinite(found.first) || precision::is_infinite(found.last);
        let shape = projected_edge(projected, ends, infinite)?;
        StdPrsWfShape::add(prs, &shape, drawer)?;
        value.wire_aspect().set_type_of_line(style.call_line);
        if infinite {
            return Ok(());
        }
        let first = BRepTool::pnt(&topexp::first_vertex(edge, false))?;
        let last = BRepTool::pnt(&topexp::last_vertex(edge, false))?;
        call_line(
            prs,
            drawer,
            ends.0,
            first,
            ends.0.square_distance(&first) > SQUARE_TOLERANCE,
        )?;
        call_line(
            prs,
            drawer,
            ends.1,
            last,
            ends.1.square_distance(&last) > SQUARE_TOLERANCE,
        )
    }

    /// `ComputeProjVertexPresentation`: the projected point in the projection marker, then the
    /// call line to the vertex; `PrsDim::compute_proj_vertex_presentation(...)?`.
    pub fn compute_proj_vertex_presentation(
        prs: &Handle<dyn Graphic3dStructureSource>,
        drawer: &Handle<Prs3dDrawer>,
        vertex: &Vertex,
        projected: &Point3,
        style: &PrsDimProjectionStyle,
    ) -> Result<(), Failure> {
        let value = drawer.get().ok_or_else(super::tool_geometry::null_curve)?;
        set_point_style(value, style.color, style.projection_marker)?;
        let point = Handle::new(GeomCartesianPoint::from_pnt(*projected));
        StdPrsPoint::add(prs, &point.cast(), drawer)?;
        set_wire_style(value, style.color, style.call_line, style.width)?;
        let original = BRepTool::pnt(vertex)?;
        if !projected.is_equal(&original, precision::confusion()) {
            let shape: Shape = BRepBuilderAPIMakeEdge::from_points(*projected, original)?
                .edge()?
                .into();
            StdPrsWfShape::add(prs, &shape, drawer)?;
        }
        Ok(())
    }
}
