//! The static length members of `PrsDim_EqualDistanceRelation`: the interval between two
//! edges (lines or circles), two vertices, or an edge and a vertex.
use super::relation_support::null_object;
use super::*;
use crate::dsg_prs::{DsgPrsArrowSide, DsgPrsEqualDistancePresentation};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomLine, GeomPlane};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle3, Direction3, Line3, Point3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers as _, Graphic3dStructureSource};
use std::f64::consts::PI;

/// The inputs of the length members: drawer, arrow size, plane, placement and symbol.
#[derive(Clone)]
pub struct PrsDimEqualDistanceContext {
    /// `aDrawer`.
    pub drawer: Handle<Prs3dDrawer>,
    /// `ArrowSize`.
    pub arrow_size: f64,
    /// `Plane`.
    pub plane: Handle<GeomPlane>,
    /// `AutomaticPos`.
    pub automatic: bool,
    /// `IsSetBndBox`.
    pub is_set_bnd_box: bool,
    /// `BndBox`.
    pub bnd_box: BoundingBox3,
    /// `SymbolPrs`.
    pub symbol: DsgPrsArrowSide,
}

/// The in/out points of the length members.
#[derive(Clone, Copy, Debug, Default)]
pub struct PrsDimEqualDistancePoints {
    /// `Position` (in/out).
    pub position: Point3,
    /// `FirstAttach`.
    pub first_attach: Point3,
    /// `SecondAttach`.
    pub second_attach: Point3,
    /// `FirstExtreme`.
    pub first_extreme: Point3,
    /// `SecondExtreme`.
    pub second_extreme: Point3,
}

impl PrsDimEqualDistanceContext {
    fn pln(&self) -> Result<rustycad_tkmath::gp::Plane3, Failure> {
        Ok(self.plane.get().ok_or_else(null_object)?.pln())
    }
    fn set_arrow_length(&self, size: f64) -> Result<(), Failure> {
        let drawer = self.drawer.get().ok_or_else(null_object)?;
        let aspect = drawer.dimension_aspect();
        let aspect = aspect.get().ok_or_else(null_object)?;
        aspect
            .arrow_aspect()
            .get()
            .ok_or_else(null_object)?
            .set_length(size);
        Ok(())
    }
    /// The interval of two attach points along a direction, with its extremes.
    fn interval(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        direction: &Direction3,
        points: &mut PrsDimEqualDistancePoints,
    ) -> Result<(), Failure> {
        if self.automatic && self.is_set_bnd_box {
            points.position =
                PrsDim::translate_point_to_bound(&points.position, direction, &self.bnd_box)?;
        }
        let mut extremes = (points.first_extreme, points.second_extreme);
        let result = DsgPrsEqualDistancePresentation::add_interval_into(
            prs,
            &self.drawer,
            &points.first_attach,
            &points.second_attach,
            direction,
            &points.position,
            self.symbol,
            &mut extremes,
        );
        (points.first_extreme, points.second_extreme) = extremes;
        result
    }
}

/// The automatic position: the middle of two points moved back along a direction.
fn offset_middle(a: Point3, b: Point3, direction: Direction3, arrow_size: f64) -> Point3 {
    let middle = Point3::from((a.xyz() + b.xyz()) * 0.5);
    middle.translated(Vector3::from(direction) * arrow_size * -10.0)
}

/// The angle of a point around a center in a plane, in `[0, 2 pi)`.
fn plane_angle(
    point: Point3,
    center: Point3,
    pln: &rustycad_tkmath::gp::Plane3,
) -> Result<f64, Failure> {
    if point.distance(&center) <= precision::confusion() {
        return Ok(0.0);
    }
    let direction = Direction3::from_xyz(point.xyz() - center.xyz())?;
    let mut angle = direction.angle(&pln.x_axis().direction());
    if direction.dot(pln.y_axis().direction()) < 0.0 {
        angle = 2.0 * PI - angle;
    }
    Ok(angle)
}

/// The end nearer to a point.
fn nearer(position: &Point3, a: Point3, b: Point3) -> Point3 {
    if position.distance(&a) > position.distance(&b) {
        b
    } else {
        a
    }
}

impl PrsDimEqualDistanceRelation {
    /// `ComputeTwoEdgesLength`: the interval between two lines, or between two arcs.
    /// Example: `PrsDimEqualDistanceRelation::compute_two_edges_length(&prs, &ctx, &e1, &e2, &mut points)?`.
    pub fn compute_two_edges_length(
        prs: &Handle<dyn Graphic3dStructureSource>,
        ctx: &PrsDimEqualDistanceContext,
        first: &Edge,
        second: &Edge,
        points: &mut PrsDimEqualDistancePoints,
    ) -> Result<(), Failure> {
        let kind1 = BRepAdaptorCurve::with_edge(first)?.get_type()?;
        let kind2 = BRepAdaptorCurve::with_edge(second)?.get_type()?;
        let Some(edge1) = PrsDim::compute_edge_geometry_on_plane(first, &ctx.plane)? else {
            return Ok(());
        };
        let Some(edge2) = PrsDim::compute_edge_geometry_on_plane(second, &ctx.plane)? else {
            return Ok(());
        };
        prs.get()
            .ok_or_else(null_object)?
            .set_infinite_state(edge1.infinite || edge2.infinite);
        let ((p11, p12), (p21, p22)) = (edge1.ends, edge2.ends);
        if kind1 == CurveType::Line && kind2 == CurveType::Line {
            let l1 = edge1
                .curve
                .cast::<GeomLine>()
                .get()
                .ok_or_else(null_object)?
                .lin();
            let l2 = edge2
                .curve
                .cast::<GeomLine>()
                .get()
                .ok_or_else(null_object)?
                .lin();
            let value = l1.distance_line(l2)?;
            let direction = l1.direction();
            if ctx.automatic {
                points.position = Self::lines_position(&l1, &l2, &edge1, &edge2, ctx.arrow_size)?;
            } else {
                points.position = PrsDim::project_point_on_plane(&points.position, &ctx.pln()?);
            }
            points.first_attach = if !edge1.infinite {
                nearer(&points.position, p11, p12)
            } else {
                elclib::value(elclib::parameter(&l1, points.position)?, &l1)
            };
            points.second_attach = if !edge2.infinite {
                nearer(&points.position, p21, p22)
            } else {
                elclib::value(elclib::parameter(&l2, points.position)?, &l2)
            };
            let mut size = ctx.arrow_size;
            if size < precision::confusion() {
                size = value * 0.1;
            }
            if value.abs() <= precision::confusion() {
                size = 0.0;
            }
            ctx.set_arrow_length(size)?;
            ctx.interval(prs, &direction, points)?;
        }
        if kind1 == CurveType::Circle && kind2 == CurveType::Circle {
            Self::two_arcs(prs, ctx, &edge1, &edge2, points)?;
        }
        let style = PrsDimProjectionStyle::default();
        if !edge1.on_plane {
            PrsDim::compute_proj_edge_presentation(
                prs,
                &ctx.drawer,
                first,
                &edge1.curve,
                edge1.ends,
                &style,
            )?;
        }
        if !edge2.on_plane {
            PrsDim::compute_proj_edge_presentation(
                prs,
                &ctx.drawer,
                second,
                &edge2.curve,
                edge2.ends,
                &style,
            )?;
        }
        Ok(())
    }

    /// The automatic position of two lines: beside the start of the earlier one.
    fn lines_position(
        l1: &Line3,
        l2: &Line3,
        edge1: &PrsDimProjectedEdge,
        edge2: &PrsDimProjectedEdge,
        arrow_size: f64,
    ) -> Result<Point3, Failure> {
        let (p11, p21) = (edge1.ends.0, edge2.ends.0);
        let (a, b) = if !(edge1.infinite || edge2.infinite) {
            let par1 = elclib::parameter(l1, p11)?;
            let par2 = elclib::parameter(l1, p21)?;
            if par1 < par2 {
                (p11, elclib::value(elclib::parameter(l2, p11)?, l2))
            } else {
                (p21, elclib::value(par2, l1))
            }
        } else if !edge1.infinite {
            (p21, elclib::value(elclib::parameter(l1, p21)?, l1))
        } else if !edge2.infinite {
            (p11, elclib::value(elclib::parameter(l2, p11)?, l2))
        } else {
            (l1.location(), l2.location())
        };
        Ok(offset_middle(a, b, l1.direction(), arrow_size))
    }

    /// The interval between two arcs at their last points (or the position).
    fn two_arcs(
        prs: &Handle<dyn Graphic3dStructureSource>,
        ctx: &PrsDimEqualDistanceContext,
        edge1: &PrsDimProjectedEdge,
        edge2: &PrsDimProjectedEdge,
        points: &mut PrsDimEqualDistancePoints,
    ) -> Result<(), Failure> {
        let circ1 = edge1
            .curve
            .cast::<GeomCircle>()
            .get()
            .ok_or_else(null_object)?
            .circ()?;
        let mut circ2 = edge2
            .curve
            .cast::<GeomCircle>()
            .get()
            .ok_or_else(null_object)?
            .circ()?;
        let tol = precision::confusion();
        if circ2.axis().is_opposite(circ1.axis(), tol)
            || circ2.x_axis().is_opposite(circ1.x_axis(), tol)
            || circ2.y_axis().is_opposite(circ1.y_axis(), tol)
        {
            circ2.set_position(circ1.position());
            circ2.set_axis(circ1.axis())?;
        }
        let (p12, p22) = (edge1.ends.1, edge2.ends.1);
        if ctx.automatic {
            let pln = ctx.pln()?;
            let pr12 = PrsDim::project_point_on_plane(&p12, &pln);
            let pr22 = PrsDim::project_point_on_plane(&p22, &pln);
            let center = PrsDim::project_point_on_plane(&circ1.location(), &pln);
            let par1 = plane_angle(pr12, center, &pln)?;
            let par2 = plane_angle(pr22, center, &pln)?;
            if par1 > par2 {
                points.first_extreme = p12;
                points.second_extreme = elclib::value(elclib::parameter(&circ2, p12)?, &circ2);
            } else {
                points.first_extreme = elclib::value(elclib::parameter(&circ1, p22)?, &circ1);
                points.second_extreme = p22;
            }
        } else {
            points.first_extreme =
                elclib::value(elclib::parameter(&circ1, points.position)?, &circ1);
            points.second_extreme =
                elclib::value(elclib::parameter(&circ2, points.position)?, &circ2);
        }
        DsgPrsEqualDistancePresentation::add_interval_between_two_arcs(
            prs,
            &ctx.drawer,
            &circ1,
            &circ2,
            &p12,
            &points.first_extreme,
            &p22,
            &points.second_extreme,
            ctx.symbol,
        )?;
        points.first_attach = p12;
        points.second_attach = p22;
        points.position = Point3::from((p12.xyz() + p22.xyz()) * 0.5);
        Ok(())
    }

    /// `ComputeTwoVerticesLength`: the interval between two vertices (projected on the plane),
    /// across them or along a plane axis.
    /// Example: `PrsDimEqualDistanceRelation::compute_two_vertices_length(&prs, &ctx, &v1, &v2, kind, &mut points)?`.
    pub fn compute_two_vertices_length(
        prs: &Handle<dyn Graphic3dStructureSource>,
        ctx: &PrsDimEqualDistanceContext,
        first: &Vertex,
        second: &Vertex,
        type_dist: PrsDimTypeOfDist,
        points: &mut PrsDimEqualDistancePoints,
    ) -> Result<(), Failure> {
        let (a1, on1) = PrsDim::compute_vertex_geometry(first, &ctx.plane)?;
        let (a2, on2) = PrsDim::compute_vertex_geometry(second, &ctx.plane)?;
        points.first_attach = a1;
        points.second_attach = a2;
        let same = a1.is_equal(&a2, precision::confusion());
        let pln = ctx.pln()?;
        let mut direction = Direction3::default();
        match type_dist {
            PrsDimTypeOfDist::Vertical => direction = pln.x_axis().direction(),
            PrsDimTypeOfDist::Horizontal => direction = pln.y_axis().direction(),
            _ if !same => {
                direction =
                    Direction3::from_xyz(a2.xyz() - a1.xyz())?.rotated(pln.axis(), PI / 2.0)?;
            }
            _ => {}
        }
        if ctx.automatic {
            if !same {
                points.position = offset_middle(a1, a2, direction, ctx.arrow_size);
            } else {
                let normal = pln.axis().direction();
                let shifted = a1.translated(Vector3::from(normal.xyz() * 10.0 * ctx.arrow_size));
                points.position = PrsDim::project_point_on_plane(&shifted, &pln);
                direction = Direction3::from_xyz(points.position.xyz() - a1.xyz())?;
            }
        } else {
            points.position = PrsDim::project_point_on_plane(&points.position, &pln);
        }
        ctx.set_arrow_length(ctx.arrow_size)?;
        ctx.interval(prs, &direction, points)?;
        let style = PrsDimProjectionStyle::default();
        if !on1 {
            PrsDim::compute_proj_vertex_presentation(prs, &ctx.drawer, first, &a1, &style)?;
        }
        if !on2 {
            PrsDim::compute_proj_vertex_presentation(prs, &ctx.drawer, second, &a2, &style)?;
        }
        Ok(())
    }

    /// `ComputeOneEdgeOneVertexLength`: the interval between a line or arc and a vertex.
    /// Example: `PrsDimEqualDistanceRelation::compute_one_edge_one_vertex_length(&prs, &ctx, &s1, &s2, &mut points)?`.
    pub fn compute_one_edge_one_vertex_length(
        prs: &Handle<dyn Graphic3dStructureSource>,
        ctx: &PrsDimEqualDistanceContext,
        first: &Shape,
        second: &Shape,
        points: &mut PrsDimEqualDistancePoints,
    ) -> Result<(), Failure> {
        let (vertex, edge, edge_first) = if first.shape_type()? == ShapeEnum::Vertex {
            (topods::vertex(first)?, topods::edge(second)?, false)
        } else {
            (topods::vertex(second)?, topods::edge(first)?, true)
        };
        let Some(projected) = PrsDim::compute_edge_geometry_on_plane(&edge, &ctx.plane)? else {
            return Ok(());
        };
        prs.get()
            .ok_or_else(null_object)?
            .set_infinite_state(projected.infinite);
        let (attach, vertex_on_plane) = PrsDim::compute_vertex_geometry(&vertex, &ctx.plane)?;
        points.first_attach = attach;
        let (e1, e2) = projected.ends;
        if let Some(line) = projected.curve.cast::<GeomLine>().get() {
            let lin = line.lin();
            let value = lin.distance(points.first_attach);
            let direction = lin.direction();
            let size = if value.abs() <= precision::confusion() {
                0.0
            } else {
                ctx.arrow_size
            };
            if ctx.automatic {
                let p = elclib::value(elclib::parameter(&lin, points.first_attach)?, &lin);
                points.position = offset_middle(points.first_attach, p, direction, ctx.arrow_size);
            } else {
                points.position = PrsDim::project_point_on_plane(&points.position, &ctx.pln()?);
            }
            points.second_attach = if !projected.infinite {
                nearer(&points.position, e1, e2)
            } else {
                elclib::value(elclib::parameter(&lin, points.position)?, &lin)
            };
            ctx.set_arrow_length(size)?;
            ctx.interval(prs, &direction, points)?;
        }
        if let Some(circle) = projected.curve.cast::<GeomCircle>().get() {
            let circ1 = circle.circ()?;
            let mut circ2: Circle3 = circ1;
            circ2.set_radius(0.0)?;
            if ctx.automatic {
                points.second_attach = e2;
                points.position = Point3::from((e2.xyz() + circ1.location().xyz()) * 0.5);
            } else {
                points.second_attach =
                    elclib::value(elclib::parameter(&circ1, points.position)?, &circ1);
            }
            let vertex_point = points.first_attach;
            DsgPrsEqualDistancePresentation::add_interval_between_two_arcs(
                prs,
                &ctx.drawer,
                &circ1,
                &circ2,
                &e2,
                &points.second_attach,
                &vertex_point,
                &vertex_point,
                ctx.symbol,
            )?;
            if edge_first {
                points.first_extreme = points.second_attach;
                points.second_extreme = points.first_attach;
                points.second_attach = points.first_attach;
                points.first_attach = e2;
            } else {
                points.first_extreme = points.first_attach;
                points.second_extreme = points.second_attach;
                points.second_attach = e2;
            }
        }
        let style = PrsDimProjectionStyle::default();
        if !projected.on_plane {
            PrsDim::compute_proj_edge_presentation(
                prs,
                &ctx.drawer,
                &edge,
                &projected.curve,
                projected.ends,
                &style,
            )?;
        }
        if !vertex_on_plane {
            PrsDim::compute_proj_vertex_presentation(
                prs,
                &ctx.drawer,
                &vertex,
                &points.first_attach,
                &style,
            )?;
        }
        Ok(())
    }
}
