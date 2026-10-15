//! `PrsDim_SymmetricRelation`: two edges (lines or arcs) or vertices symmetric about an axis
//! edge, drawn as the symmetry symbol between their attach points.
use super::dimension_impls::prs_dim_relation_class;
use super::relation_support::{null_object, relation_arrow};
use super::*;
use crate::dsg_prs::DsgPrsSymmetricPresentation;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomPlane};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle3, Direction3, Line3, Point3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers as _, Graphic3dStructureSource};

/// The tool, attach points and directions of `PrsDim_SymmetricRelation`.
#[derive(Clone, Default)]
pub struct PrsDimSymmetricFields {
    /// `myTool`.
    pub tool: Shape,
    /// `myFAttach`.
    pub f_attach: Point3,
    /// `mySAttach`.
    pub s_attach: Point3,
    /// `myFDirAttach`.
    pub f_dir_attach: Direction3,
    /// `myAxisDirAttach`.
    pub axis_dir_attach: Direction3,
}

prs_dim_relation_class!(
    PrsDimSymmetricRelation,
    "PrsDim_SymmetricRelation",
    PrsDimSymmetricFields,
    "`PrsDim_SymmetricRelation`; `PrsDimSymmetricRelation::new(&axis, &s1, &s2, &plane)?`.",
    movable = true
);

/// The projection of a point on a line.
pub(super) fn project(line: &Line3, point: Point3) -> Result<Point3, Failure> {
    Ok(elclib::value(elclib::parameter(line, point)?, line))
}

/// The line of a curve known to be a line (a null dereference natively otherwise).
pub(super) fn lin_of(curve: &Handle<dyn GeomCurve>) -> Result<Line3, Failure> {
    Ok(curve
        .cast::<GeomLine>()
        .get()
        .ok_or_else(null_object)?
        .lin())
}

impl PrsDimSymmetricRelation {
    /// `PrsDim_SymmetricRelation(tool, shape1, shape2, plane)`: positioned at the plane origin.
    /// Example: `PrsDimSymmetricRelation::new(&axis, &s1, &s2, &plane)?`.
    pub fn new(
        tool: &Shape,
        first: &Shape,
        second: &Shape,
        plane: &Handle<GeomPlane>,
    ) -> Result<Handle<Self>, Failure> {
        let relation = Self::alloc(PrsDimSymmetricFields {
            tool: tool.clone(),
            ..PrsDimSymmetricFields::default()
        });
        relation.set_first_shape(first);
        relation.set_second_shape(second);
        relation.set_plane(plane);
        let location = plane.get().ok_or_else(null_object)?.pln().location();
        relation.base.fields.write().position = location;
        Ok(relation)
    }
    /// `SetTool`; `relation.set_tool(&axis)`.
    pub fn set_tool(&self, tool: &Shape) {
        self.fields.write().tool = tool.clone();
    }
    /// `GetTool`; `relation.get_tool()`.
    pub fn get_tool(&self) -> Shape {
        self.fields.read().tool.clone()
    }
    /// The axis of the tool edge on the plane, with whether it is infinite and its ends and
    /// original curve when off the plane (`None` when not computed).
    pub(super) fn tool_axis(&self) -> Result<Option<PrsDimProjectedEdge>, Failure> {
        let tool = topods::edge(&self.get_tool())?;
        PrsDim::compute_edge_geometry_on_plane(&tool, &self.plane())
    }

    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        match self.first_shape().shape_type()? {
            ShapeEnum::Face => self.compute_two_faces_symmetric(prs),
            ShapeEnum::Edge => self.compute_two_edges_symmetric(prs)?,
            ShapeEnum::Vertex => self.compute_two_vertices_symmetric(prs)?,
            _ => {}
        }
        if self.get_tool().shape_type()? != ShapeEnum::Edge {
            return Ok(());
        }
        let Some(axis) = self.tool_axis()? else {
            return Ok(());
        };
        if axis.ext_curve.is_null() {
            return Ok(());
        }
        let ends = if axis.infinite {
            prs.get().ok_or_else(null_object)?.set_infinite_state(true);
            (Point3::default(), Point3::default())
        } else {
            axis.ends
        };
        let tool = topods::edge(&self.get_tool())?;
        let style = PrsDimProjectionStyle::default();
        self.compute_proj_edge_presentation(prs, &tool, &axis.curve, ends.0, ends.1, &style)
    }

    /// `ComputeTwoFacesSymmetric`: nothing, as natively.
    pub fn compute_two_faces_symmetric(&self, _: &Handle<dyn Graphic3dStructureSource>) {}

    /// The attach points of two finite edges: their far ends (when consecutive), else the
    /// ends nearer to the offset projection.
    fn finite_attaches(
        projected: Point3,
        (p11, p12): (Point3, Point3),
        (p21, p22): (Point3, Point3),
    ) -> (Point3, Point3) {
        let conf = precision::confusion();
        let mut found = None;
        for (shared, first, second) in [
            (p11.is_equal(&p21, conf), p12, p22),
            (p11.is_equal(&p22, conf), p12, p21),
            (p12.is_equal(&p21, conf), p11, p22),
            (p12.is_equal(&p22, conf), p11, p21),
        ] {
            if shared {
                found = Some((first, second));
            }
        }
        found.unwrap_or_else(|| {
            let first = if projected.square_distance(&p11) > projected.square_distance(&p12) {
                p12
            } else {
                p11
            };
            let second = if projected.square_distance(&p21) > projected.square_distance(&p22) {
                p22
            } else {
                p21
            };
            (first, second)
        })
    }

    /// `ComputeTwoEdgesSymmetric`: the symbol between two lines or arcs.
    /// Example: `relation.compute_two_edges_symmetric(&prs)?`.
    pub fn compute_two_edges_symmetric(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let first = topods::edge(&self.first_shape())?;
        let second = topods::edge(&self.second_shape())?;
        let kind1 = BRepAdaptorCurve::with_edge(&first)?.get_type()?;
        if kind1 != CurveType::Line && kind1 != CurveType::Circle {
            return Ok(());
        }
        let kind2 = BRepAdaptorCurve::with_edge(&second)?.get_type()?;
        if kind2 != CurveType::Line && kind2 != CurveType::Circle {
            return Ok(());
        }
        let Some((done, edges)) = PrsDim::compute_two_edges_ext(&first, &second, &self.plane())?
        else {
            return Ok(());
        };
        self.set_ext_shape(edges.ext_index);
        if !done {
            return Ok(());
        }
        let (inf1, inf2) = (edges.first_infinite, edges.second_infinite);
        prs.get()
            .ok_or_else(null_object)?
            .set_infinite_state((inf1 || inf2) && self.ext_shape() != 0);
        let Some(axis) = self.tool_axis()? else {
            return Ok(());
        };
        let laxis = lin_of(&axis.curve)?;
        let mut f = self.fields.read().clone();
        f.axis_dir_attach = laxis.direction();
        if kind1 == CurveType::Line {
            f.f_dir_attach = lin_of(&edges.first_curve)?.direction();
        }
        let mut circle = Circle3::default();
        if kind1 == CurveType::Circle {
            circle = edges
                .first_curve
                .cast::<GeomCircle>()
                .get()
                .ok_or_else(null_object)?
                .circ()?;
        }
        let projected = project(&laxis, self.position())?;
        let (ends1, ends2) = (edges.first_ends, edges.second_ends);
        if inf1 && inf2 {
            let line2 = lin_of(&edges.second_curve)?;
            f.f_attach = if self.automatic_position() {
                lin_of(&edges.first_curve)?.location()
            } else {
                project(&lin_of(&edges.first_curve)?, self.position())?
            };
            f.s_attach = project(&line2, f.f_attach)?;
        } else if !inf1 && !inf2 {
            (f.f_attach, f.s_attach) = Self::finite_attaches(projected, ends1, ends2);
        } else if inf1 {
            f.s_attach = ends2.0;
            f.f_attach = project(&lin_of(&edges.first_curve)?, f.s_attach)?;
        } else {
            f.f_attach = ends1.0;
            f.s_attach = project(&lin_of(&edges.second_curve)?, f.f_attach)?;
        }
        if !self.relation_base().fields.read().arrow_size_is_defined {
            self.relation_base().fields.write().arrow_size =
                f.f_attach.distance(&f.s_attach) / 50.0;
        }
        let mut pj_first = project(&laxis, f.f_attach)?;
        if pj_first.is_equal(&f.f_attach, precision::confusion()) {
            f.f_dir_attach = lin_of(&edges.second_curve)?.direction();
            std::mem::swap(&mut f.f_attach, &mut f.s_attach);
            pj_first = project(&laxis, f.f_attach)?;
        }
        if self.automatic_position() {
            self.set_symmetric_position(&f, pj_first);
        }
        let pj1 = project(&laxis, f.f_attach)?;
        let pj2 = project(&laxis, f.s_attach)?;
        if f.f_attach.square_distance(&pj1) + f.s_attach.square_distance(&pj2)
            <= precision::confusion()
        {
            self.relation_base().fields.write().arrow_size = 0.0;
        }
        *self.fields.write() = f.clone();
        self.apply_arrow_length()?;
        let position = self.position();
        if kind1 == CurveType::Line {
            DsgPrsSymmetricPresentation::add_edges(
                prs,
                &self.attributes(),
                &f.f_attach,
                &f.s_attach,
                &f.f_dir_attach,
                &laxis,
                &position,
            )?;
        }
        if kind1 == CurveType::Circle {
            DsgPrsSymmetricPresentation::add_arcs(
                prs,
                &self.attributes(),
                &f.f_attach,
                &f.s_attach,
                &circle,
                &laxis,
                &position,
            )?;
        }
        let ext = self.ext_shape();
        if ext == 0 || edges.ext_curve.is_null() {
            return Ok(());
        }
        let style = PrsDimProjectionStyle::default();
        let unset = (Point3::default(), Point3::default());
        if ext == 1 {
            let ends = if inf1 { unset } else { ends1 };
            self.compute_proj_edge_presentation(
                prs,
                &first,
                &edges.first_curve,
                ends.0,
                ends.1,
                &style,
            )
        } else {
            let ends = if inf2 { unset } else { ends2 };
            self.compute_proj_edge_presentation(
                prs,
                &second,
                &edges.second_curve,
                ends.0,
                ends.1,
                &style,
            )
        }
    }

    /// The automatic position: beside the first attach projection, offset along the axis.
    fn set_symmetric_position(&self, f: &PrsDimSymmetricFields, pj_first: Point3) {
        let offset = Vector3::from(f.axis_dir_attach) * self.arrow_size() * -5.0;
        let toward = Vector3::between(f.f_attach, pj_first) * 0.15;
        self.relation_base().fields.write().position = pj_first.translated(offset + toward);
    }
    fn apply_arrow_length(&self) -> Result<(), Failure> {
        let arrow = relation_arrow(self)?;
        arrow
            .get()
            .ok_or_else(null_object)?
            .set_length(self.arrow_size());
        Ok(())
    }

    /// `ComputeTwoVerticesSymmetric`: the symbol between two vertices (projected on the plane;
    /// nothing when both are off it).
    /// Example: `relation.compute_two_vertices_symmetric(&prs)?`.
    pub fn compute_two_vertices_symmetric(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let (first, second) = (self.first_shape(), self.second_shape());
        if first.shape_type()? != ShapeEnum::Vertex || second.shape_type()? != ShapeEnum::Vertex {
            return Ok(());
        }
        let Some(axis) = self.tool_axis()? else {
            return Ok(());
        };
        let (v1, v2) = (topods::vertex(&first)?, topods::vertex(&second)?);
        let plane = self.plane();
        let (a1, on1) = PrsDim::compute_vertex_geometry(&v1, &plane)?;
        let (a2, on2) = PrsDim::compute_vertex_geometry(&v2, &plane)?;
        let mut f = self.fields.read().clone();
        f.f_attach = a1;
        f.s_attach = a2;
        *self.fields.write() = f.clone();
        if !self.relation_base().fields.read().arrow_size_is_defined {
            self.relation_base().fields.write().arrow_size = a1.distance(&a2) / 50.0;
        }
        let ext = match (on1, on2) {
            (true, true) => 0,
            (true, false) => 2,
            (false, true) => 1,
            (false, false) => return Ok(()),
        };
        self.set_ext_shape(ext);
        let laxis = lin_of(&axis.curve)?;
        f.axis_dir_attach = laxis.direction();
        *self.fields.write() = f.clone();
        if self.automatic_position() {
            let pj_first = project(&laxis, f.f_attach)?;
            self.set_symmetric_position(&f, pj_first);
        }
        if 2.0 * a1.distance(&a2) <= precision::confusion() {
            self.relation_base().fields.write().arrow_size = 0.0;
        }
        self.apply_arrow_length()?;
        DsgPrsSymmetricPresentation::add_vertices(
            prs,
            &self.attributes(),
            &f.f_attach,
            &f.s_attach,
            &laxis,
            &self.position(),
        )?;
        let style = PrsDimProjectionStyle::default();
        match ext {
            1 => self.compute_proj_vertex_presentation(prs, &v1, &f.f_attach, &style),
            2 => self.compute_proj_vertex_presentation(prs, &v2, &f.s_attach, &style),
            _ => Ok(()),
        }
    }

    fn compute_relation_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        super::symmetric_selection::symmetric_selection(self, selection)
    }
}
