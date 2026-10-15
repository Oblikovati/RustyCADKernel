//! `PrsDim_ParallelRelation`: two parallel edges (lines, or ellipses through their focus
//! axis), drawn as a length line labelled " //" between them.
use super::dimension_impls::prs_dim_relation_class;
use super::perpendicular_relation::line_of;
use super::relation_support::{null_object, relation_arrow, relation_owner};
use super::*;
use crate::dsg_prs::{DsgPrsArrowSide, DsgPrsLengthPresentation};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use crate::select3d::{Select3DSensitiveBox, Select3DSensitiveSegment};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gce::MakeLin;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers as _, Graphic3dStructureSource};

/// The attach points and direction of `PrsDim_ParallelRelation`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PrsDimParallelFields {
    /// `myFAttach`.
    pub f_attach: Point3,
    /// `mySAttach`.
    pub s_attach: Point3,
    /// `myDirAttach`.
    pub dir_attach: Direction3,
}

prs_dim_relation_class!(
    PrsDimParallelRelation,
    "PrsDim_ParallelRelation",
    PrsDimParallelFields,
    "`PrsDim_ParallelRelation`; `PrsDimParallelRelation::new(&s1, &s2, &plane)`.",
    movable = true
);

/// The attach point of a side: the edge end nearer to the position (the farther for an
/// ellipse), or the position projected on an infinite line.
fn attach_of(
    position: &Point3,
    line: &Line3,
    ends: (Point3, Point3),
    infinite: bool,
    ellipse: bool,
) -> Result<Point3, Failure> {
    if infinite {
        return Ok(elclib::value(elclib::parameter(line, *position)?, line));
    }
    let (d1, d2) = (position.distance(&ends.0), position.distance(&ends.1));
    Ok(if ellipse {
        if d1 < d2 { ends.1 } else { ends.0 }
    } else if d1 > d2 {
        ends.1
    } else {
        ends.0
    })
}

impl PrsDimParallelRelation {
    fn with_shapes(first: &Shape, second: &Shape, plane: &Handle<GeomPlane>) -> Handle<Self> {
        let relation = Self::alloc(PrsDimParallelFields::default());
        {
            let mut f = relation.base.fields.write();
            f.f_shape = first.clone();
            f.s_shape = second.clone();
            f.plane = plane.clone();
        }
        relation
    }
    /// `PrsDim_ParallelRelation(shape1, shape2, plane)`: automatic, both arrows, size 0.01.
    /// Example: `PrsDimParallelRelation::new(&s1, &s2, &plane)`.
    pub fn new(first: &Shape, second: &Shape, plane: &Handle<GeomPlane>) -> Handle<Self> {
        let relation = Self::with_shapes(first, second, plane);
        {
            let mut f = relation.base.fields.write();
            f.automatic_position = true;
            f.arrow_size = 0.01;
            f.symbol_prs = DsgPrsArrowSide::BothAr;
        }
        relation
    }
    /// `PrsDim_ParallelRelation(shape1, shape2, plane, position, symbol, arrowSize)`.
    /// Example: `PrsDimParallelRelation::with_position(&s1, &s2, &pl, &p, side, 1.0)`.
    pub fn with_position(
        first: &Shape,
        second: &Shape,
        plane: &Handle<GeomPlane>,
        position: &Point3,
        symbol: DsgPrsArrowSide,
        arrow_size: f64,
    ) -> Handle<Self> {
        let relation = Self::with_shapes(first, second, plane);
        relation.base.fields.write().automatic_position = false;
        relation.set_arrow_size(arrow_size);
        let mut f = relation.base.fields.write();
        f.position = *position;
        f.symbol_prs = symbol;
        drop(f);
        relation
    }

    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        match self.first_shape().shape_type()? {
            ShapeEnum::Face => self.compute_two_faces_parallel(prs),
            ShapeEnum::Edge => self.compute_two_edges_parallel(prs),
            _ => Ok(()),
        }
    }

    /// `ComputeTwoFacesParallel`: `Standard_NotImplemented`, as natively.
    pub fn compute_two_faces_parallel(
        &self,
        _: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        Err(Failure::with_message(
            "Standard_NotImplemented",
            "PrsDim_ParallelRelation::ComputeTwoFacesParallel not implemented",
        ))
    }

    /// `ComputeTwoEdgesParallel`: the length line between the two edge lines, the arrow size
    /// from the edge lengths and the automatic position beside them.
    /// Example: `relation.compute_two_edges_parallel(&prs)?`.
    pub fn compute_two_edges_parallel(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let first = topods::edge(&self.first_shape())?;
        let second = topods::edge(&self.second_shape())?;
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
        let Some((lin1, ends1, el1)) = line_of(&edges.first_curve, edges.first_ends)? else {
            return Ok(());
        };
        let Some((lin2, ends2, el2)) = line_of(&edges.second_curve, edges.second_ends)? else {
            return Ok(());
        };
        let l1 = lin1.get().ok_or_else(null_object)?.lin();
        let l2 = lin2.get().ok_or_else(null_object)?.lin();
        let mut fields = *self.fields.read();
        fields.dir_attach = l1.direction();
        let mut r = self.relation_base().fields.read().clone();
        if !r.arrow_size_is_defined {
            let size1 = if inf1 {
                r.arrow_size
            } else {
                ends1.0.distance(&ends1.1) / 50.0
            };
            let size2 = if inf2 {
                r.arrow_size
            } else {
                ends2.0.distance(&ends2.1) / 50.0
            };
            r.arrow_size = r.arrow_size.max(size1.max(size2));
        }
        if r.automatic_position {
            let mut position = if !inf1 {
                let p2 = elclib::value(elclib::parameter(&l2, ends1.0)?, &l2);
                Point3::from((ends1.0.xyz() + p2.xyz()) / 2.0)
            } else if !inf2 {
                let p2 = elclib::value(elclib::parameter(&l1, ends2.0)?, &l1);
                Point3::from((ends2.0.xyz() + p2.xyz()) / 2.0)
            } else {
                Point3::from((l1.location().xyz() + l2.location().xyz()) / 2.0)
            };
            let offset = Vector3::from(fields.dir_attach) * r.arrow_size * -10.0;
            position = position.translated(offset);
            r.position = position;
        }
        fields.f_attach = attach_of(&r.position, &l1, ends1, inf1, el1)?;
        fields.s_attach = attach_of(&r.position, &l2, ends2, inf2, el2)?;
        if l1.distance_line(l2)? <= precision::confusion() {
            r.arrow_size = 0.0;
        }
        match r.ext_shape {
            1 => r.symbol_prs = DsgPrsArrowSide::FirstPtLastAr,
            2 => r.symbol_prs = DsgPrsArrowSide::FirstArLastPt,
            _ => {}
        }
        *self.relation_base().fields.write() = r.clone();
        *self.fields.write() = fields;
        let arrow = relation_arrow(self)?;
        arrow
            .get()
            .ok_or_else(null_object)?
            .set_length(r.arrow_size);
        DsgPrsLengthPresentation::add_with_arrow(
            prs,
            &self.attributes(),
            &ExtendedString::from_utf8(" //"),
            &fields.f_attach,
            &fields.s_attach,
            &fields.dir_attach,
            &r.position,
            r.symbol_prs,
        )?;
        if r.ext_shape == 0 || edges.ext_curve.is_null() {
            return Ok(());
        }
        let style = PrsDimProjectionStyle::default();
        let unset = (Point3::default(), Point3::default());
        if r.ext_shape == 1 {
            let ends = if inf1 { unset } else { ends1 };
            self.compute_proj_edge_presentation(prs, &first, &lin1.cast(), ends.0, ends.1, &style)
        } else {
            let ends = if inf2 { unset } else { ends2 };
            self.compute_proj_edge_presentation(prs, &second, &lin2.cast(), ends.0, ends.1, &style)
        }
    }

    fn compute_relation_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        let f = *self.fields.read();
        let r = self.relation_base().fields.read().clone();
        let l1 = Line3::new(f.f_attach, f.dir_attach);
        let l2 = Line3::new(f.s_attach, f.dir_attach);
        let proj1 = elclib::value(elclib::parameter(&l1, r.position)?, &l1);
        let proj2 = elclib::value(elclib::parameter(&l2, r.position)?, &l2);
        let owner = relation_owner(self, 7);
        let target = selection.get().ok_or_else(null_object)?;
        let l3 = if !proj1.is_equal(&proj2, precision::confusion()) {
            MakeLin::from_points(proj1, proj2)?.value()?
        } else {
            let size = (r.val / 100.0 + 1.0e-6).min(r.arrow_size + 1.0e-6);
            let p = r.position;
            let bounds = Select3DSensitiveBox::from_bounds(
                &owner,
                p.x(),
                p.y(),
                p.z(),
                p.x() + size,
                p.y() + size,
                p.z() + size,
            );
            target.add(&Handle::new(bounds).cast())?;
            Line3::new(proj1, f.dir_attach)
        };
        let mut min = elclib::parameter(&l3, proj1)?;
        let mut max = min;
        for point in [proj2, r.position] {
            let current = elclib::parameter(&l3, point)?;
            min = min.min(current);
            max = max.max(current);
        }
        let (point_min, point_max) = (elclib::value(min, &l3), elclib::value(max, &l3));
        let segment = |a: Point3, b: Point3| {
            target.add(&Handle::new(Select3DSensitiveSegment::new(&owner, a, b)).cast())
        };
        if !point_min.is_equal(&point_max, precision::confusion()) {
            segment(point_min, point_max)?;
        }
        if !f.f_attach.is_equal(&proj1, precision::confusion()) {
            segment(f.f_attach, proj1)?;
        }
        if !f.s_attach.is_equal(&proj2, precision::confusion()) {
            segment(f.s_attach, proj2)?;
        }
        Ok(())
    }
}
