//! `PrsDim_MaxRadiusDimension` and `PrsDim_MinRadiusDimension`: the major or minor radius of
//! an ellipse (or elliptic arc), drawn from the center to the nearest apex.
use super::dimension_impls::prs_dim_relation_class;
use super::ellipse_radius_dimension::init_ellipse_radius;
use super::relation_support::{null_object, relation_arrow, relation_owner};
use super::*;
use crate::dsg_prs::{DsgPrsArrowSide, DsgPrsEllipseRadiusPresentation};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::select_mgr::SelectMgrSelection;
use crate::select3d::{Select3DSensitiveBox, Select3DSensitiveCurve, Select3DSensitiveSegment};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::{GeomCurve, GeomEllipse, GeomTrimmedCurve};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis1, Direction3, Ellipse3, Line3, Point3, Vector3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;

/// The axis and radius setter distinguishing the maximum and minimum radius.
#[derive(Clone, Copy)]
struct RadiusKind {
    /// Whether this is the major radius (`XAxis`, `SetMajorRadius`).
    major: bool,
}
impl RadiusKind {
    fn axis(self, ellipse: &Ellipse3) -> Axis1 {
        if self.major {
            ellipse.x_axis()
        } else {
            ellipse.y_axis()
        }
    }
    fn set_radius(self, ellipse: &mut Ellipse3, value: f64) -> Result<(), Failure> {
        if self.major {
            ellipse.set_major_radius(value)
        } else {
            ellipse.set_minor_radius(value)
        }
    }
}

/// The automatic label position: the center (moved to the bounding box).
fn automatic_position<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
    center: Point3,
) -> Result<Point3, Failure> {
    let r = {
        let mut f = relation.relation_base().fields.write();
        f.position = center;
        f.automatic_position = true;
        f.clone()
    };
    if !r.is_set_bnd_box {
        return Ok(center);
    }
    // The direction from the center to itself raises, as natively.
    let direction = Direction3::from_vector(Vector3::between(center, r.position))?;
    let position = PrsDim::translate_point_to_bound(&r.position, &direction, &r.bnd_box)?;
    relation.relation_base().fields.write().position = position;
    Ok(position)
}

/// The arrow size (at most a fifth of the value unless defined) applied to the arrow aspect.
fn apply_arrow_size<R: PrsDimRelationSource + ?Sized>(relation: &R) -> Result<(), Failure> {
    let r = relation.relation_base().fields.read().clone();
    if !r.arrow_size_is_defined {
        relation.relation_base().fields.write().arrow_size = r.arrow_size.min(r.val / 5.0);
    }
    relation_arrow(relation)?
        .get()
        .ok_or_else(null_object)?
        .set_length(relation.arrow_size());
    Ok(())
}

/// The position projected on the radius axis and the apex nearer to it.
fn projected_apex(
    fields: &PrsDimEllipseRadiusFields,
    axis: Axis1,
    position: Point3,
) -> Result<(Point3, Point3), Failure> {
    let line = Line3::from_axis(axis);
    let current = elclib::value(elclib::parameter(&line, position)?, &line);
    let end = if current.distance(&fields.apex_p) < current.distance(&fields.apex_n) {
        fields.apex_p
    } else {
        fields.apex_n
    };
    Ok((current, end))
}

/// The arc end nearer (along the ellipse) to the end of the arrow.
fn nearest_arc_end(f: &PrsDimEllipseRadiusFields) -> Result<f64, Failure> {
    let first = PrsDim::distance_from_apex(&f.ellipse, &f.end_of_arrow, f.first_par)?;
    let last = PrsDim::distance_from_apex(&f.ellipse, &f.end_of_arrow, f.last_par)?;
    Ok(if first < last {
        f.first_par
    } else {
        f.last_par
    })
}

macro_rules! radius_dimension {
    ($type:ident, $native:literal, $doc:literal, $major:expr) => {
        prs_dim_relation_class!(
            $type,
            $native,
            PrsDimEllipseRadiusFields,
            $doc,
            kind = PrsDimKindOfDimension::EllipseRadius,
            movable = true
        );
        impl PrsDimEllipseRadiusDimension for $type {
            fn ellipse_fields(&self) -> &SharedState<PrsDimEllipseRadiusFields> {
                &self.fields
            }
        }
        impl $type {
            const KIND: RadiusKind = RadiusKind { major: $major };

            /// The automatic constructor: an arrow at the end, size value / 100.
            pub fn new(shape: &Shape, value: f64, text: &ExtendedString) -> Handle<Self> {
                let relation = Self::alloc(PrsDimEllipseRadiusFields::default());
                init_ellipse_radius(&*relation, shape, text);
                let mut f = relation.base.fields.write();
                f.val = value;
                f.symbol_prs = DsgPrsArrowSide::LastAr;
                f.automatic_position = true;
                f.arrow_size = value / 100.0;
                drop(f);
                relation
            }
            /// The constructor with a position, symbol and arrow size.
            pub fn with_position(
                shape: &Shape,
                value: f64,
                text: &ExtendedString,
                position: &Point3,
                symbol: DsgPrsArrowSide,
                arrow_size: f64,
            ) -> Handle<Self> {
                let relation = Self::alloc(PrsDimEllipseRadiusFields::default());
                init_ellipse_radius(&*relation, shape, text);
                {
                    let mut f = relation.base.fields.write();
                    f.val = value;
                    f.symbol_prs = symbol;
                    f.position = *position;
                    f.automatic_position = false;
                }
                relation.set_arrow_size(arrow_size);
                relation
            }

            fn compute_relation(
                &self,
                prs: &Handle<dyn Graphic3dStructureSource>,
                _: i32,
            ) -> Result<(), Failure> {
                self.compute_geometry()?;
                let value = self.value();
                {
                    let mut f = self.fields.write();
                    Self::KIND.set_radius(&mut f.ellipse, value)?;
                    let shift = Vector3::from(Self::KIND.axis(&f.ellipse).direction()) * value;
                    f.apex_p = f.ellipse.location().translated(shift);
                    f.apex_n = f.ellipse.location().translated(shift.reversed());
                }
                if self.fields.read().is_an_arc {
                    self.compute_arc_of_ellipse(prs)
                } else {
                    self.compute_ellipse(prs)
                }
            }

            /// `ComputeEllipse`: the radius of a whole ellipse.
            /// Example: `dimension.compute_ellipse(&prs)?`.
            pub fn compute_ellipse(
                &self,
                prs: &Handle<dyn Graphic3dStructureSource>,
            ) -> Result<(), Failure> {
                apply_arrow_size(self)?;
                let mut f = self.fields.read().clone();
                let center = f.ellipse.location();
                let position = if self.automatic_position() {
                    f.end_of_arrow = f.apex_p;
                    *self.fields.write() = f.clone();
                    automatic_position(self, center)?
                } else {
                    let (current, end) =
                        projected_apex(&f, Self::KIND.axis(&f.ellipse), self.position())?;
                    f.end_of_arrow = end;
                    if !$major {
                        self.relation_base().fields.write().position = current;
                    }
                    current
                };
                *self.fields.write() = f.clone();
                let r = self.relation_base().fields.read().clone();
                DsgPrsEllipseRadiusPresentation::add(
                    prs,
                    &self.attributes(),
                    r.val,
                    &r.text,
                    &position,
                    &f.end_of_arrow,
                    &center,
                    $major,
                    r.symbol_prs,
                )
            }

            /// `ComputeArcOfEllipse`: the radius of an elliptic arc, with the arc extension
            /// to an apex outside it.
            /// Example: `dimension.compute_arc_of_ellipse(&prs)?`.
            pub fn compute_arc_of_ellipse(
                &self,
                prs: &Handle<dyn Graphic3dStructureSource>,
            ) -> Result<(), Failure> {
                apply_arrow_size(self)?;
                let mut f = self.fields.read().clone();
                let center = f.ellipse.location();
                let (position, in_domain) = if self.automatic_position() {
                    let (end, in_domain) = PrsDim::nearest_apex(
                        &f.ellipse,
                        &f.apex_p,
                        &f.apex_n,
                        f.first_par,
                        f.last_par,
                    )?;
                    f.end_of_arrow = end;
                    *self.fields.write() = f.clone();
                    (automatic_position(self, center)?, in_domain)
                } else {
                    let (current, end) =
                        projected_apex(&f, Self::KIND.axis(&f.ellipse), self.position())?;
                    f.end_of_arrow = end;
                    let parameter = elclib::parameter(&f.ellipse, end)?;
                    let in_domain = PrsDim::in_domain(f.first_par, f.last_par, parameter);
                    self.relation_base().fields.write().position = current;
                    (current, in_domain)
                };
                *self.fields.write() = f.clone();
                let start = if in_domain { 0.0 } else { nearest_arc_end(&f)? };
                let r = self.relation_base().fields.read().clone();
                if !f.is_offset {
                    DsgPrsEllipseRadiusPresentation::add_ellipse(
                        prs,
                        &self.attributes(),
                        r.val,
                        &r.text,
                        &f.ellipse,
                        &position,
                        &f.end_of_arrow,
                        &center,
                        start,
                        in_domain,
                        true,
                        r.symbol_prs,
                    )
                } else {
                    DsgPrsEllipseRadiusPresentation::add_offset_curve(
                        prs,
                        &self.attributes(),
                        r.val,
                        &r.text,
                        &f.offset_curve,
                        &position,
                        &f.end_of_arrow,
                        &center,
                        start,
                        in_domain,
                        true,
                        r.symbol_prs,
                    )
                }
            }

            fn compute_relation_selection(
                &self,
                selection: &Handle<SelectMgrSelection>,
                _: i32,
            ) -> Result<(), Failure> {
                let f = self.fields.read().clone();
                ellipse_radius_selection(self, selection, &f)
            }
        }
    };
}

/// The selection of a max/min radius: the radius segment, a text box and the arc extension.
fn ellipse_radius_selection<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
    selection: &Handle<SelectMgrSelection>,
    f: &PrsDimEllipseRadiusFields,
) -> Result<(), Failure> {
    let r = relation.relation_base().fields.read().clone();
    let center = f.ellipse.location();
    let attachment = r.position;
    let first = if center.distance(&attachment) > r.val {
        attachment
    } else {
        f.end_of_arrow
    };
    let owner = relation_owner(relation, 7);
    let target = selection.get().ok_or_else(null_object)?;
    target.add(&Handle::new(Select3DSensitiveSegment::new(&owner, center, first)).cast())?;
    let size = (r.val / 100.0 + 1.0e-6).min(r.arrow_size + 1.0e-6);
    let bounds = Select3DSensitiveBox::from_bounds(
        &owner,
        attachment.x(),
        attachment.y(),
        attachment.z(),
        attachment.x() + size,
        attachment.y() + size,
        attachment.z() + size,
    );
    target.add(&Handle::new(bounds).cast())?;
    if !f.is_an_arc {
        return Ok(());
    }
    let mut end = elclib::parameter(&f.ellipse, f.end_of_arrow)?;
    if PrsDim::in_domain(f.first_par, f.last_par, end) {
        return Ok(());
    }
    let parameter = nearest_arc_end(f)?;
    let apex = Vector3::between(center, elclib::value(end, &f.ellipse));
    let point = Vector3::between(center, elclib::value(parameter, &f.ellipse));
    let direction = Direction3::from_vector(point.crossed(apex))?;
    let start = if f
        .ellipse
        .position()
        .direction()
        .is_opposite(direction, precision::angular())
    {
        let start = end;
        end = parameter;
        start
    } else {
        parameter
    };
    let basis: Handle<dyn GeomCurve> = if f.is_offset {
        f.offset_curve.clone().cast()
    } else {
        Handle::new(GeomEllipse::from_elips(f.ellipse)).cast()
    };
    let trimmed = GeomTrimmedCurve::new(&basis, start, end, true, true)?;
    let arc = Select3DSensitiveCurve::from_curve(&owner, &trimmed, 17)?;
    target.add(&Handle::new(arc).cast())
}

radius_dimension!(
    PrsDimMaxRadiusDimension,
    "PrsDim_MaxRadiusDimension",
    "`PrsDim_MaxRadiusDimension`; `PrsDimMaxRadiusDimension::new(&edge, 5.0, &text)`.",
    true
);
radius_dimension!(
    PrsDimMinRadiusDimension,
    "PrsDim_MinRadiusDimension",
    "`PrsDim_MinRadiusDimension`; `PrsDimMinRadiusDimension::new(&edge, 3.0, &text)`.",
    false
);
