//! `PrsDim_EqualRadiusRelation`: two circular edges of equal radius, drawn as both radii
//! joined by their center line with an equality mark.
use super::dimension_impls::prs_dim_relation_class;
use super::relation_support::{null_object, relation_arrow, relation_owner};
use super::*;
use crate::dsg_prs::DsgPrsEqualRadiusPresentation;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use crate::select3d::{Select3DSensitiveBox, Select3DSensitiveSegment};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Edge};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomPlane};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Direction3, Point3, Vector3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use std::f64::consts::PI;

/// The centers and radius points of `PrsDim_EqualRadiusRelation`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PrsDimEqualRadiusFields {
    /// `myFirstCenter`.
    pub first_center: Point3,
    /// `mySecondCenter`.
    pub second_center: Point3,
    /// `myFirstPoint`.
    pub first_point: Point3,
    /// `mySecondPoint`.
    pub second_point: Point3,
}

prs_dim_relation_class!(
    PrsDimEqualRadiusRelation,
    "PrsDim_EqualRadiusRelation",
    PrsDimEqualRadiusFields,
    "`PrsDim_EqualRadiusRelation`; `PrsDimEqualRadiusRelation::new(&e1, &e2, &plane)`."
);

/// The untransformed curve of an edge adaptor with its parameter range.
fn edge_curve(edge: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let adaptor = BRepAdaptorCurve::with_edge(edge)?;
    let curve = adaptor
        .transformed_curve()
        .curve()
        .get()
        .map(|c| c.curve())
        .unwrap_or_default();
    Ok((curve, adaptor.first_parameter()?, adaptor.last_parameter()?))
}

/// A user point moved onto the arc of a circle: onto the circle when off it, then to the
/// nearest arc end when outside the arc.
fn point_on_arc(
    point: Point3,
    circle: &Circle3,
    (first, last): (f64, f64),
    (first_end, last_end): (Point3, Point3),
) -> Result<Point3, Failure> {
    let mut point = point;
    let mut parameter = elclib::parameter(circle, point)?;
    let turns = (0.5 * last / PI).trunc();
    if turns != 0.0 && parameter < first {
        parameter += 2.0 * PI * turns;
    }
    if (point.distance(&circle.location()) - circle.radius()).abs() >= precision::confusion() {
        point = elclib::value(parameter, circle);
    }
    if first_end.distance(&last_end) > precision::confusion()
        && (parameter > last || parameter < first)
    {
        point = if first_end.distance(&point) < last_end.distance(&point) {
            first_end
        } else {
            last_end
        };
    }
    Ok(point)
}

impl PrsDimEqualRadiusRelation {
    /// `PrsDim_EqualRadiusRelation(edge1, edge2, plane)`.
    /// Example: `PrsDimEqualRadiusRelation::new(&e1, &e2, &plane)`.
    pub fn new(first: &Edge, second: &Edge, plane: &Handle<GeomPlane>) -> Handle<Self> {
        let relation = Self::alloc(PrsDimEqualRadiusFields::default());
        {
            let mut f = relation.base.fields.write();
            f.f_shape = first.clone().into();
            f.s_shape = second.clone().into();
            f.plane = plane.clone();
        }
        relation
    }

    fn compute_relation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        let first_edge = topods::edge(&self.first_shape())?;
        let second_edge = topods::edge(&self.second_shape())?;
        let (mut first_curve, first1, last1) = edge_curve(&first_edge)?;
        let (mut second_curve, first2, last2) = edge_curve(&second_edge)?;
        let plane = self.plane();
        let mut first_ends = (Point3::default(), Point3::default());
        let mut second_ends = first_ends;
        let first_on_plane =
            PrsDim::compute_geom_curve(&mut first_curve, first1, last1, &mut first_ends, &plane)?;
        let second_on_plane =
            PrsDim::compute_geom_curve(&mut second_curve, first2, last2, &mut second_ends, &plane)?;
        let style = PrsDimProjectionStyle::default();
        if !first_on_plane {
            self.compute_proj_edge_presentation(
                prs,
                &first_edge,
                &first_curve,
                first_ends.0,
                first_ends.1,
                &style,
            )?;
        }
        if !second_on_plane {
            self.compute_proj_edge_presentation(
                prs,
                &second_edge,
                &second_curve,
                second_ends.0,
                second_ends.1,
                &style,
            )?;
        }
        let circle = |curve: &Handle<dyn GeomCurve>| -> Result<Circle3, Failure> {
            curve
                .cast::<GeomCircle>()
                .get()
                .ok_or_else(null_object)?
                .circ()
        };
        let (first_circle, second_circle) = (circle(&first_curve)?, circle(&second_curve)?);
        let mut f = *self.fields.read();
        f.first_center = first_circle.location();
        f.second_center = second_circle.location();
        if self.automatic_position() {
            f.first_point = elclib::value((first1 + last1) * 0.5, &first_circle);
            f.second_point = elclib::value((first2 + last2) * 0.5, &second_circle);
        } else {
            f.first_point =
                point_on_arc(f.first_point, &first_circle, (first1, last1), first_ends)?;
            f.second_point =
                point_on_arc(f.second_point, &second_circle, (first2, last2), second_ends)?;
        }
        *self.fields.write() = f;
        if !self.relation_base().fields.read().arrow_size_is_defined {
            let size = f
                .first_center
                .distance(&f.first_point)
                .min(f.second_center.distance(&f.second_point))
                * 0.05;
            self.relation_base().fields.write().arrow_size = size;
        }
        relation_arrow(self)?
            .get()
            .ok_or_else(null_object)?
            .set_length(self.arrow_size());
        DsgPrsEqualRadiusPresentation::add(
            prs,
            &self.attributes(),
            &f.first_center,
            &f.second_center,
            &f.first_point,
            &f.second_point,
            &plane,
        )
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
        segment(f.first_center, f.first_point)?;
        if !self.automatic_position() {
            self.compute_radius_position()?;
        }
        let f = *self.fields.read();
        segment(f.second_center, f.second_point)?;
        segment(f.first_center, f.second_center)?;
        let middle = Point3::from((f.first_center.xyz() + f.second_center.xyz()) * 0.5);
        let small = 0.001;
        let bounds = Select3DSensitiveBox::from_bounds(
            &owner,
            middle.x() - small,
            middle.y() - small,
            middle.z() - small,
            middle.x() + small,
            middle.y() + small,
            middle.z() + small,
        );
        target.add(&Handle::new(bounds).cast())
    }

    /// `ComputeRadiusPosition`: the radius point nearer to the position (projected on the
    /// plane) turned toward it.
    /// Example: `relation.compute_radius_position()?`.
    pub fn compute_radius_position(&self) -> Result<(), Failure> {
        let mut f = *self.fields.read();
        let position = self.position();
        if self.automatic_position()
            || f.first_center.distance(&position) < precision::confusion()
            || f.second_center.distance(&position) < precision::confusion()
        {
            return Ok(());
        }
        let plane = self.plane();
        let projected = ProjectPointOnSurf::with_surface(position, &plane.cast(), ExtAlgo::Grad)?
            .nearest_point()?;
        let toward = |center: Point3, point: Point3| -> Result<Point3, Failure> {
            let radius = point.distance(&center);
            let direction = Direction3::from_xyz(projected.xyz() - center.xyz())?;
            Ok(center.translated(Vector3::from(direction.xyz() * radius)))
        };
        if f.first_point.distance(&projected) < f.second_point.distance(&projected) {
            f.first_point = toward(f.first_center, f.first_point)?;
        } else {
            f.second_point = toward(f.second_center, f.second_point)?;
        }
        *self.fields.write() = f;
        Ok(())
    }
}
