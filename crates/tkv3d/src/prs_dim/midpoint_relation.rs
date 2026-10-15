//! `PrsDim_MidPointRelation`: a vertex at the middle of two shapes (edges or vertices), drawn
//! as a mark at the mid point with segments to its attach points on the shapes.
use super::dimension_impls::prs_dim_relation_class;
use super::midpoint_points::{points_on_conic, points_on_line, points_on_segment};
use super::relation_support::{null_object, relation_owner};
use super::*;
use crate::dsg_prs::DsgPrsMidPointPresentation;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use crate::select3d::{Select3DSensitiveCurve, Select3DSensitiveSegment};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomEllipse, GeomLine, GeomPlane, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Point3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;

/// The tool, mid point and the attach points with their mark ends of both shapes.
#[derive(Clone, Default)]
pub struct PrsDimMidPointFields {
    /// `myTool`.
    pub tool: Shape,
    /// `myMidPoint`.
    pub mid_point: Point3,
    /// `myFAttach`.
    pub f_attach: Point3,
    /// `myFirstPnt1`.
    pub first_pnt1: Point3,
    /// `myFirstPnt2`.
    pub first_pnt2: Point3,
    /// `mySAttach`.
    pub s_attach: Point3,
    /// `mySecondPnt1`.
    pub second_pnt1: Point3,
    /// `mySecondPnt2`.
    pub second_pnt2: Point3,
}

impl PrsDimMidPointFields {
    fn side(&self, first: bool) -> (Point3, Point3, Point3) {
        if first {
            (self.f_attach, self.first_pnt1, self.first_pnt2)
        } else {
            (self.s_attach, self.second_pnt1, self.second_pnt2)
        }
    }
    fn set_side(&mut self, first: bool, (attach, p1, p2): (Point3, Point3, Point3)) {
        if first {
            (self.f_attach, self.first_pnt1, self.first_pnt2) = (attach, p1, p2);
        } else {
            (self.s_attach, self.second_pnt1, self.second_pnt2) = (attach, p1, p2);
        }
    }
}

prs_dim_relation_class!(
    PrsDimMidPointRelation,
    "PrsDim_MidPointRelation",
    PrsDimMidPointFields,
    "`PrsDim_MidPointRelation`; `PrsDimMidPointRelation::new(&tool, &s1, &s2, &plane)?`.",
    movable = true
);

impl PrsDimMidPointRelation {
    /// `PrsDim_MidPointRelation(tool, shape1, shape2, plane)`: positioned at the plane origin.
    /// Example: `PrsDimMidPointRelation::new(&vertex, &s1, &s2, &plane)?`.
    pub fn new(
        tool: &Shape,
        first: &Shape,
        second: &Shape,
        plane: &Handle<GeomPlane>,
    ) -> Result<Handle<Self>, Failure> {
        let relation = Self::alloc(PrsDimMidPointFields {
            tool: tool.clone(),
            ..PrsDimMidPointFields::default()
        });
        relation.set_first_shape(first);
        relation.set_second_shape(second);
        relation.set_plane(plane);
        let location = plane.get().ok_or_else(null_object)?.pln().location();
        relation.base.fields.write().position = location;
        Ok(relation)
    }
    /// `SetTool`; `relation.set_tool(&vertex)`.
    pub fn set_tool(&self, tool: &Shape) {
        self.fields.write().tool = tool.clone();
    }
    /// `GetTool`; `relation.get_tool()`.
    pub fn get_tool(&self) -> Shape {
        self.fields.read().tool.clone()
    }
    fn axes(&self) -> Result<rustycad_tkmath::gp::Axis2, Failure> {
        self.plane()
            .get()
            .ok_or_else(null_object)?
            .pln()
            .position()
            .ax2()
    }

    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let tool = self.get_tool();
        if tool.shape_type()? != ShapeEnum::Vertex {
            return Ok(());
        }
        let vertex = topods::vertex(&tool)?;
        let (point, on_plane) = PrsDim::compute_vertex_geometry(&vertex, &self.plane())?;
        if !on_plane {
            let style = PrsDimProjectionStyle::default();
            self.compute_proj_vertex_presentation(prs, &vertex, &point, &style)?;
        }
        self.fields.write().mid_point = point;
        if self.automatic_position() {
            self.relation_base().fields.write().position = point;
        }
        for (shape, first) in [(self.first_shape(), true), (self.second_shape(), false)] {
            match shape.shape_type()? {
                ShapeEnum::Face => self.compute_face_from_pnt(prs, first),
                ShapeEnum::Edge => self.compute_edge_from_pnt(prs, first)?,
                ShapeEnum::Vertex => self.compute_vertex_from_pnt(prs, first)?,
                _ => {}
            }
        }
        Ok(())
    }

    /// `ComputeFaceFromPnt`: nothing, as natively.
    pub fn compute_face_from_pnt(&self, _: &Handle<dyn Graphic3dStructureSource>, _: bool) {}

    /// `ComputeEdgeFromPnt`: the attach point and marks on a line, circle or ellipse edge.
    /// Example: `relation.compute_edge_from_pnt(&prs, true)?`.
    pub fn compute_edge_from_pnt(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        first: bool,
    ) -> Result<(), Failure> {
        let shape = if first {
            self.first_shape()
        } else {
            self.second_shape()
        };
        let edge = topods::edge(&shape)?;
        let Some(projected) = PrsDim::compute_edge_geometry_on_plane(&edge, &self.plane())? else {
            return Ok(());
        };
        let (ptat1, ptat2) = projected.ends;
        let geom = projected.curve.clone();
        let mut f = self.fields.read().clone();
        let mid = f.mid_point;
        let position = self.position();
        let drawer = self.attributes();
        if let Some(line) = geom.cast::<GeomLine>().get() {
            let side = if projected.infinite {
                points_on_line(&line.lin(), &mid)?
            } else {
                points_on_segment(&ptat1, &ptat2, &mid)?
            };
            f.set_side(first, side);
            *self.fields.write() = f.clone();
            let (attach, p1, p2) = f.side(first);
            DsgPrsMidPointPresentation::add_segment(
                prs,
                &drawer,
                &self.axes()?,
                &mid,
                &position,
                &attach,
                &p1,
                &p2,
                first,
            )?;
        } else if let Some(circle) = geom.cast::<GeomCircle>().get() {
            let circ = circle.circ()?;
            f.set_side(
                first,
                points_on_conic(&circ, circ.location(), &mid, &ptat1, &ptat2)?,
            );
            *self.fields.write() = f.clone();
            let (attach, p1, p2) = f.side(first);
            DsgPrsMidPointPresentation::add_circle(
                prs, &drawer, &circ, &mid, &position, &attach, &p1, &p2, first,
            )?;
        } else if let Some(ellipse) = geom.cast::<GeomEllipse>().get() {
            let elips = ellipse.elips()?;
            f.set_side(
                first,
                points_on_conic(&elips, elips.location(), &mid, &ptat1, &ptat2)?,
            );
            *self.fields.write() = f.clone();
            let (attach, p1, p2) = f.side(first);
            DsgPrsMidPointPresentation::add_ellipse(
                prs, &drawer, &elips, &mid, &position, &attach, &p1, &p2, first,
            )?;
        } else {
            return Ok(());
        }
        if !projected.on_plane {
            let style = PrsDimProjectionStyle::default();
            self.compute_proj_edge_presentation(prs, &edge, &geom, ptat1, ptat2, &style)?;
        }
        Ok(())
    }

    /// `ComputeVertexFromPnt`: the vertex (projected on the plane) as the attach point.
    /// Example: `relation.compute_vertex_from_pnt(&prs, false)?`.
    pub fn compute_vertex_from_pnt(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        first: bool,
    ) -> Result<(), Failure> {
        let shape = if first {
            self.first_shape()
        } else {
            self.second_shape()
        };
        let vertex = topods::vertex(&shape)?;
        let (attach, on_plane) = PrsDim::compute_vertex_geometry(&vertex, &self.plane())?;
        let mid = {
            let mut f = self.fields.write();
            if first {
                f.f_attach = attach;
            } else {
                f.s_attach = attach;
            }
            f.mid_point
        };
        DsgPrsMidPointPresentation::add(
            prs,
            &self.attributes(),
            &self.axes()?,
            &mid,
            &self.position(),
            &attach,
            first,
        )?;
        if !on_plane {
            let style = PrsDimProjectionStyle::default();
            self.compute_proj_vertex_presentation(prs, &vertex, &attach, &style)?;
        }
        Ok(())
    }

    fn compute_relation_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        let owner = relation_owner(self, 7);
        let target = selection.get().ok_or_else(null_object)?;
        let segment = |a: Point3, b: Point3| {
            target.add(&Handle::new(Select3DSensitiveSegment::new(&owner, a, b)).cast())
        };
        let curve_entity = |curve: &dyn GeomCurve| -> Result<(), Failure> {
            let entity = Select3DSensitiveCurve::from_curve(&owner, curve, 17)?;
            target.add(&Handle::new(entity).cast())
        };
        let f = self.fields.read().clone();
        let position = self.position();
        if !f.mid_point.is_equal(&f.f_attach, precision::confusion()) {
            segment(f.f_attach, f.mid_point)?;
            segment(f.s_attach, f.mid_point)?;
        }
        if !f.mid_point.is_equal(&position, precision::confusion()) {
            segment(f.mid_point, position)?;
        }
        let mut axes = self.axes()?;
        axes.set_location(f.mid_point);
        let radius = f.f_attach.distance(&f.mid_point) / 20.0;
        curve_entity(&GeomCircle::from_circ(Circle3::new(axes, radius)?))?;
        for (shape, first) in [(self.first_shape(), true), (self.second_shape(), false)] {
            if shape.shape_type()? != ShapeEnum::Edge {
                continue;
            }
            let edge = topods::edge(&shape)?;
            let Some(projected) = PrsDim::compute_edge_geometry_on_plane(&edge, &self.plane())?
            else {
                return Ok(());
            };
            let (_, p1, p2) = f.side(first);
            let curve = &projected.curve;
            let Some(value) = curve.get() else {
                continue;
            };
            use rustycad_tkernel::standard::RuntimeTyped;
            if value.is_instance(GeomLine::static_type()) {
                segment(p1, p2)?;
            } else if value.is_instance(GeomCircle::static_type()) {
                let circ = curve
                    .cast::<GeomCircle>()
                    .get()
                    .ok_or_else(null_object)?
                    .circ()?;
                let (u1, u2) = (elclib::parameter(&circ, p1)?, elclib::parameter(&circ, p2)?);
                curve_entity(&GeomTrimmedCurve::new(curve, u1, u2, true, true)?)?;
            } else if value.is_instance(GeomEllipse::static_type()) {
                let elips = curve
                    .cast::<GeomEllipse>()
                    .get()
                    .ok_or_else(null_object)?
                    .elips()?;
                let (u1, u2) = (
                    elclib::parameter(&elips, p1)?,
                    elclib::parameter(&elips, p2)?,
                );
                curve_entity(&GeomTrimmedCurve::new(curve, u1, u2, true, true)?)?;
            }
        }
        Ok(())
    }
}
