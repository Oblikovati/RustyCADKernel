//! `PrsDim_PerpendicularRelation`: two perpendicular edges (lines, or ellipses through their
//! focus axis), drawn as a right-angle symbol at their intersection.
use super::dimension_impls::prs_dim_relation_class;
use super::relation_support::{null_object, relation_owner};
use super::*;
use crate::dsg_prs::DsgPrsPerpenPresentation;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use crate::select3d::Select3DSensitiveSegment;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dLine;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomConic, GeomCurve, GeomEllipse, GeomLine, GeomPlane};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_api;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers as _, Graphic3dStructureSource};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

/// The attach points of `PrsDim_PerpendicularRelation` (`myFAttach`, `mySAttach`).
#[derive(Clone, Copy, Debug, Default)]
pub struct PrsDimPerpendicularFields {
    /// `myFAttach`.
    pub f_attach: Point3,
    /// `mySAttach`.
    pub s_attach: Point3,
}

prs_dim_relation_class!(
    PrsDimPerpendicularRelation,
    "PrsDim_PerpendicularRelation",
    PrsDimPerpendicularFields,
    "`PrsDim_PerpendicularRelation`; `PrsDimPerpendicularRelation::new(&s1, &s2, Some(&plane))`."
);

/// `gce_MakeDir(P1, P2)` as a direction (StdFail_NotDone for equal points).
pub(super) fn make_dir(from: &Point3, to: &Point3) -> Result<Direction3, Failure> {
    Direction3::from_xyz(to.xyz() - from.xyz())
        .map_err(|_| Failure::with_message("StdFail_NotDone", "gce_MakeDir::Value() - no result"))
}

/// A line with its two reference points and whether it comes from an ellipse.
pub(super) type EdgeLine = (Handle<GeomLine>, (Point3, Point3), bool);

/// The line of an edge curve: a line itself, or the major axis of an ellipse with the points
/// between its foci (then out of the intersection); `None` for other curves.
pub(super) fn line_of(
    curve: &Handle<dyn GeomCurve>,
    ends: (Point3, Point3),
) -> Result<Option<EdgeLine>, Failure> {
    if let Some(ellipse) = curve.cast::<GeomEllipse>().get() {
        let axis = ellipse.x_axis();
        let line = Handle::new(GeomLine::from_lin(Line3::from_axis(axis)));
        let focex = ellipse.major_radius() - ellipse.focal() / 2.0;
        let shift = Vector3::from(axis.direction()) * focex;
        let ends = (
            ellipse.focus1().translated(shift),
            ellipse.focus2().translated(shift.reversed()),
        );
        return Ok(Some((line, ends, true)));
    }
    let line = curve.cast::<GeomLine>();
    if line.is_null() {
        return Ok(None);
    }
    Ok(Some((line, ends, false)))
}

/// The 2D line of a 3D line in a plane (`GeomAPI::To2d` then `Geom2d_Line::Lin2d`).
fn line_2d(
    line: &Handle<GeomLine>,
    plane: &rustycad_tkmath::gp::Plane3,
) -> Result<rustycad_tkmath::gp::Line2, Failure> {
    let curve = geom_api::to_2d(&line.clone().cast(), plane)?;
    Ok(curve
        .cast::<Geom2dLine>()
        .get()
        .ok_or_else(null_object)?
        .lin2d())
}

impl PrsDimPerpendicularRelation {
    /// `PrsDim_PerpendicularRelation(shape1, shape2[, plane])`.
    /// Example: `PrsDimPerpendicularRelation::new(&s1, &s2, Some(&plane))`.
    pub fn new(first: &Shape, second: &Shape, plane: Option<&Handle<GeomPlane>>) -> Handle<Self> {
        let relation = Self::alloc(PrsDimPerpendicularFields::default());
        {
            let mut f = relation.base.fields.write();
            f.f_shape = first.clone();
            f.s_shape = second.clone();
            if let Some(plane) = plane {
                f.plane = plane.clone();
            }
        }
        relation
    }

    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let (first, second) = (self.first_shape(), self.second_shape());
        if first.shape_type()? == second.shape_type()? {
            match first.shape_type()? {
                ShapeEnum::Face => self.compute_two_faces_perpendicular(prs),
                ShapeEnum::Edge => self.compute_two_edges_perpendicular(prs)?,
                _ => {}
            }
        }
        Ok(())
    }

    /// `ComputeTwoFacesPerpendicular`: nothing, as natively.
    pub fn compute_two_faces_perpendicular(&self, _: &Handle<dyn Graphic3dStructureSource>) {}

    /// `ComputeTwoEdgesPerpendicular`: the right-angle symbol at the intersection of the two
    /// edge lines, with attach points along them.
    /// Example: `relation.compute_two_edges_perpendicular(&prs)?`.
    pub fn compute_two_edges_perpendicular(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let first = topods::edge(&self.first_shape())?;
        let second = topods::edge(&self.second_shape())?;
        let plane = self.plane();
        let Some((done, edges)) = PrsDim::compute_two_edges_ext(&first, &second, &plane)? else {
            return Ok(());
        };
        self.set_ext_shape(edges.ext_index);
        if !done {
            return Ok(());
        }
        let Some((lin1, ends1, mut out1)) = line_of(&edges.first_curve, edges.first_ends)? else {
            return Ok(());
        };
        let Some((lin2, ends2, mut out2)) = line_of(&edges.second_curve, edges.second_ends)? else {
            return Ok(());
        };
        let pln = plane.get().ok_or_else(null_object)?.pln();
        let face = BRepBuilderAPIMakeFace::from_plane(pln)?.face()?;
        let surface = BRepAdaptorSurface::with_face(&face, true)?;
        let intersection = AnaIntersection::lines(line_2d(&lin1, &pln)?, line_2d(&lin2, &pln)?);
        if !intersection.is_done() || intersection.nb_points()? == 0 {
            return Ok(());
        }
        let point2d = intersection.point(1)?.value();
        let position = surface.value(point2d.x(), point2d.y())?;
        self.relation_base().fields.write().position = position;
        let (l1, l2) = (
            lin1.get().ok_or_else(null_object)?.lin(),
            lin2.get().ok_or_else(null_object)?.lin(),
        );
        let (inf1, inf2) = (edges.first_infinite, edges.second_infinite);
        let mut f = *self.fields.read();
        let (mut p1, mut p2) = (Point3::default(), Point3::default());
        if inf1 && inf2 {
            let at1 = elclib::parameter(&l1, position)?;
            let at2 = elclib::parameter(&l2, position)?;
            p1 = position;
            p2 = position;
            f.f_attach = elclib::value(at1 + 50.0, &l1);
            f.s_attach = elclib::value(at2 + 50.0, &l2);
        } else {
            let mut length = 0.0;
            let mut length_computed = false;
            let side = |line: &Line3, ends: (Point3, Point3)| -> Result<(Point3, bool), Failure> {
                let current = elclib::parameter(line, position)?;
                let a = elclib::parameter(line, ends.0)?;
                let b = elclib::parameter(line, ends.1)?;
                let far = if position.square_distance(&ends.0) > position.square_distance(&ends.1) {
                    ends.0
                } else {
                    ends.1
                };
                Ok((far, current < a.min(b) || current > a.max(b)))
            };
            if !inf1 {
                let (far, out) = side(&l1, ends1)?;
                p1 = far;
                out1 |= out;
                length = if !inf2 {
                    2.0 * ends1.0.distance(&ends1.1).min(ends2.0.distance(&ends2.1)) / 5.0
                } else {
                    2.0 * ends1.0.distance(&ends1.1) / 5.0
                };
                length_computed = true;
                let direction = Vector3::from(make_dir(&position, &p1)?) * length;
                f.f_attach = position.translated(direction);
            }
            if !inf2 {
                let (far, out) = side(&l2, ends2)?;
                p2 = far;
                out2 |= out;
                let direction = make_dir(&position, &p2)?;
                if !length_computed {
                    length = if !inf1 {
                        2.0 * ends1.0.distance(&ends1.1).min(ends2.0.distance(&ends2.1)) / 5.0
                    } else {
                        2.0 * ends2.0.distance(&ends2.1) / 5.0
                    };
                }
                f.s_attach = position.translated(Vector3::from(direction) * length);
            }
            if inf1 {
                p1 = position;
                f.f_attach = position.translated(Vector3::from(l1.direction()) * length);
            }
            if inf2 {
                p2 = position;
                f.s_attach = position.translated(Vector3::from(l2.direction()) * length);
            }
        }
        *self.fields.write() = f;
        DsgPrsPerpenPresentation::add(
            prs,
            &self.attributes(),
            &f.f_attach,
            &f.s_attach,
            &p1,
            &p2,
            &position,
            out1,
            out2,
        )?;
        let ext = self.ext_shape();
        if ext == 0 || edges.ext_curve.is_null() {
            return Ok(());
        }
        let style = PrsDimProjectionStyle::default();
        let unset = (Point3::default(), Point3::default());
        let value = prs.get().ok_or_else(null_object)?;
        if ext == 1 {
            let ends = if inf1 { unset } else { ends1 };
            value.set_infinite_state(inf1);
            self.compute_proj_edge_presentation(prs, &first, &lin1.cast(), ends.0, ends.1, &style)
        } else {
            let ends = if inf2 { unset } else { ends2 };
            value.set_infinite_state(inf2);
            self.compute_proj_edge_presentation(prs, &second, &lin2.cast(), ends.0, ends.1, &style)
        }
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
        let f = *self.fields.read();
        let position = self.position();
        let ok1 = !f.f_attach.is_equal(&position, precision::confusion());
        if ok1 {
            segment(f.f_attach, position)?;
        }
        let ok2 = !f.s_attach.is_equal(&position, precision::confusion());
        if ok2 {
            segment(f.s_attach, position)?;
        }
        if ok1 && ok2 {
            let vec1 = Vector3::from(make_dir(&position, &f.f_attach)?)
                * position.distance(&f.f_attach)
                * 0.2;
            let vec2 = Vector3::from(make_dir(&position, &f.s_attach)?)
                * position.distance(&f.s_attach)
                * 0.2;
            let ax1 = position.translated(vec1);
            let ax2 = position.translated(vec2);
            let symbol = ax2.translated(vec1);
            segment(ax1, symbol)?;
            segment(symbol, ax2)?;
        }
        Ok(())
    }
}
