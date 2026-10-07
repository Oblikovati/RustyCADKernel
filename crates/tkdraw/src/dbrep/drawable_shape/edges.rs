// Rust translation of the edge and vertex parts of DBRep_DrawableShape::DrawOn
// (Draw/TKDraw/DBRep/DBRep_DrawableShape.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edges, edge polygons and free vertices of a drawable shape.
use super::plot::{Plot, draw_and_check, record_pick};
use super::{DBRepDrawableShape, color_orientation};
use crate::dbrep::edge_face::DBRepEdge;
use crate::draw::{DrawConsole, DrawDisplay, DrawMarkerShape, DrawOStream};
use crate::draw_tr_surf::{bound, draw_arrow, interval_bounds};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, vertex as to_vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity};
use rustycad_tkmath::gp::Vector2;

/// The marker size of the free vertices (the `DrawMarker` default).
const VERTEX_MARKER_SIZE: i32 = 5;

impl DBRepDrawableShape {
    /// The edge part of `DrawOn`: each edge's curve (with an orientation arrow when
    /// orientations are shown) and its polygon when it has no curve or polygons are
    /// shown. An edge with a reversed range is reported on `out` and skipped.
    pub(super) fn draw_edges(
        &self,
        display: &mut DrawDisplay<'_>,
        halt: &mut bool,
        out: &dyn DrawConsole,
    ) -> Result<(), Failure> {
        let edges = self.edges.borrow().clone();
        for record in &edges {
            if *halt {
                break;
            }
            let color = if self.display_orientation.get() {
                color_orientation(record.edge().orientation())
            } else {
                record.color()
            };
            display.set_color(color);
            let geometric = BRepTool::is_geometric_edge(record.edge())?;
            if geometric && !self.draw_edge_curve(record, display, halt, out)? {
                continue;
            }
            if !geometric || self.polygons.get() {
                draw_edge_polygon(record.edge(), display, halt)?;
            }
        }
        Ok(())
    }

    /// The curve of one edge; false when the edge is skipped (bad range, or
    /// degenerated), which also skips its polygon.
    fn draw_edge_curve(
        &self,
        record: &DBRepEdge,
        display: &mut DrawDisplay<'_>,
        halt: &mut bool,
        out: &dyn DrawConsole,
    ) -> Result<bool, Failure> {
        let edge = record.edge();
        let (u1, u2) = BRepTool::range(edge)?;
        if u2 < u1 {
            out.write_out("DBRep_DrawableShape : Bad parameters on edge.\n");
            let mut text = DrawOStream::new();
            text.write_through(|io| brep_tools::dump(edge.as_ref(), io))?;
            out.write_out(text.text());
            return Ok(false);
        }
        if BRepTool::degenerated(edge)? {
            return Ok(false);
        }
        let curve = BRepAdaptorCurve::with_edge(edge)?;
        let (first, last) =
            self.finite_edge_range(curve.first_parameter()?, curve.last_parameter()?);
        let trimmed = curve.trim(first, last, precision::confusion())?;
        let trimmed = trimmed
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "trimmed edge curve"))?;
        self.plot_edge_intervals(record, (&curve, trimmed), display, halt)?;
        let end = curve.d0(trimmed.last_parameter()?)?;
        draw_and_check(display, end, (edge.as_ref(), last, 0.0), halt);
        if self.display_orientation.get() {
            let (point, vector) = curve.d1(last)?;
            let p1 = display.project(point);
            let p2 = display.project(point.translated(vector));
            draw_arrow(display, p1, Vector2::between(p1, p2))?;
        }
        Ok(true)
    }

    /// An infinite end is replaced by `Size` from the other end (or `[-Size, Size]`).
    fn finite_edge_range(&self, first: f64, last: f64) -> (f64, f64) {
        let size = self.size;
        if precision::is_negative_infinite(first) {
            if precision::is_positive_infinite(last) {
                return (-size, size);
            }
            return (last - size, last);
        }
        if precision::is_positive_infinite(last) {
            return (first, first + size);
        }
        (first, last)
    }

    /// The points of each CN interval of the trimmed curve: none on lines, `discret`
    /// steps on circles and ellipses (evaluated on the untrimmed curve), adaptive double
    /// steps on the other curves.
    fn plot_edge_intervals(
        &self,
        record: &DBRepEdge,
        (curve, trimmed): (&BRepAdaptorCurve, &dyn Adaptor3dCurve),
        display: &mut DrawDisplay<'_>,
        halt: &mut bool,
    ) -> Result<(), Failure> {
        let curve_type = trimmed.get_type()?;
        let bounds = interval_bounds(trimmed.nb_intervals(Continuity::CN)?, |b| {
            trimmed.intervals(b, Continuity::CN)
        })?;
        display.move_to_3d(trimmed.d0(trimmed.first_parameter()?)?);
        let discret = self.discret.get();
        for interval in 1..bounds.len() as i32 {
            let mut t = bound(&bounds, interval)?;
            let step = (bound(&bounds, interval + 1)? - t) / f64::from(discret);
            match curve_type {
                CurveType::Line => {}
                CurveType::Circle | CurveType::Ellipse => {
                    for _ in 1..discret {
                        t += step;
                        draw_and_check(
                            display,
                            curve.d0(t)?,
                            (record.edge().as_ref(), t, 0.0),
                            halt,
                        );
                    }
                }
                _ => {
                    for _ in 1..=discret / 2 {
                        Plot::new(display, record.edge().as_ref(), halt).edge(
                            trimmed,
                            t,
                            step * 2.,
                        )?;
                        t += step * 2.;
                    }
                }
            }
        }
        Ok(())
    }

    /// The vertex part of `DrawOn`: a diamond at each vertex that is not in an edge,
    /// in the connected color (or by orientation).
    pub(super) fn draw_vertices(
        &self,
        display: &mut DrawDisplay<'_>,
        halt: &mut bool,
    ) -> Result<(), Failure> {
        display.set_color(self.colors.connected);
        let mut explorer = Explorer::new(&self.shape, ShapeEnum::Vertex, ShapeEnum::Edge)?;
        while explorer.more() && !*halt {
            let current = explorer.current().clone();
            if self.display_orientation.get() {
                display.set_color(color_orientation(current.orientation()));
            }
            let point = BRepTool::pnt(&to_vertex(&current)?)?;
            display.draw_marker_3d(point, DrawMarkerShape::Losange, VERTEX_MARKER_SIZE);
            if display.has_picked() {
                record_pick(&current, 0.0, 0.0);
                *halt = true;
            }
            explorer.next()?;
        }
        Ok(())
    }
}

/// The 3D polygon of an edge, or else its polygon on a triangulation (located).
fn draw_edge_polygon(
    edge: &Edge,
    display: &mut DrawDisplay<'_>,
    halt: &mut bool,
) -> Result<(), Failure> {
    let (polygon, location) = BRepTool::polygon3d(edge)?;
    let transform = location.transformation();
    let points: Vec<rustycad_tkmath::gp::Point3> = if let Some(polygon) = polygon.get() {
        polygon.nodes().iter().copied().collect()
    } else {
        let (on_mesh, mesh, location) = BRepTool::polygon_on_triangulation_by_index(edge, 1)?;
        let (Some(on_mesh), Some(mesh)) = (on_mesh.get(), mesh.get()) else {
            return Ok(());
        };
        let transform = location.transformation();
        let mut nodes = Vec::new();
        for index in on_mesh.nodes().iter() {
            nodes.push(mesh.node(*index)?.transformed(transform));
        }
        return draw_polyline(edge, &nodes, display, halt);
    };
    let located: Vec<_> = points.iter().map(|p| p.transformed(transform)).collect();
    draw_polyline(edge, &located, display, halt)
}

/// `dis.Draw(P(i - 1), P(i))` for each pair, recording a pick of the edge.
fn draw_polyline(
    edge: &Edge,
    points: &[rustycad_tkmath::gp::Point3],
    display: &mut DrawDisplay<'_>,
    halt: &mut bool,
) -> Result<(), Failure> {
    for pair in points.windows(2) {
        display.draw_3d(pair[0], pair[1]);
        if display.has_picked() {
            record_pick(edge.as_ref(), 0.0, 0.0);
            *halt = true;
        }
    }
    Ok(())
}
