//! `PrsDim_IdenticRelation` between two circles or two ellipses: the attach arc on the shared
//! conic (whole conics, an arc and a whole conic, or two arcs) and its label position.
use super::identic_relation::PrsDimIdenticRelation;
use super::relation_support::null_object;
use super::*;
use crate::dsg_prs::DsgPrsIdenticPresentation;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::{GeomCurve, GeomEllipse};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle3, Ellipse3, Point3, Vector3};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use std::f64::consts::PI;

/// The file-local `Modulo2PI`: an angle brought into `[0, 2 pi)`.
pub(super) fn modulo_2pi(angle: f64) -> f64 {
    if angle < 0.0 {
        return modulo_2pi(angle + 2.0 * PI);
    }
    if angle >= 2.0 * PI {
        return modulo_2pi(angle - 2.0 * PI);
    }
    angle
}

/// The file-local `IsEqual2PI`: equal angles modulo a full turn.
pub(super) fn is_equal_2pi(first: f64, second: f64, precision: f64) -> bool {
    let difference = (first - second).abs();
    difference < precision || (difference - 2.0 * PI).abs() < precision
}

/// A circle or an ellipse of the relation: its parameters, values and symbol size.
pub(super) trait IdenticConic {
    /// `Radius()` or `MajorRadius()` over five.
    fn seg_size(&self) -> f64;
    /// The center.
    fn center(&self) -> Point3;
    /// `ElCLib::Parameter`.
    fn param(&self, point: Point3) -> Result<f64, Failure>;
    /// The parameter of a free point: `ElCLib::Parameter` on a circle, the projection on an
    /// ellipse (`GeomAPI_ProjectPointOnCurve`).
    fn free_param(&self, point: Point3) -> Result<f64, Failure>;
    /// `ElCLib::Value`.
    fn at(&self, parameter: f64) -> Point3;
}
impl IdenticConic for Circle3 {
    fn seg_size(&self) -> f64 {
        self.radius() / 5.0
    }
    fn center(&self) -> Point3 {
        self.location()
    }
    fn param(&self, point: Point3) -> Result<f64, Failure> {
        elclib::parameter(self, point)
    }
    fn free_param(&self, point: Point3) -> Result<f64, Failure> {
        elclib::parameter(self, point)
    }
    fn at(&self, parameter: f64) -> Point3 {
        elclib::value(parameter, self)
    }
}
impl IdenticConic for Ellipse3 {
    fn seg_size(&self) -> f64 {
        self.major_radius() / 5.0
    }
    fn center(&self) -> Point3 {
        self.location()
    }
    fn param(&self, point: Point3) -> Result<f64, Failure> {
        elclib::parameter(self, point)
    }
    fn free_param(&self, point: Point3) -> Result<f64, Failure> {
        let curve: Handle<dyn GeomCurve> = Handle::new(GeomEllipse::from_elips(*self)).cast();
        ProjectPointOnCurve::with_curve(point, &curve)?.lower_distance_parameter()
    }
    fn at(&self, parameter: f64) -> Point3 {
        elclib::value(parameter, self)
    }
}

/// A point moved off the conic center toward a point when they coincide.
fn off_center<C: IdenticConic>(
    conic: &C,
    point: Point3,
    toward: Point3,
) -> Result<Point3, Failure> {
    let center = conic.center();
    if center.distance(&point) <= precision::confusion() {
        let shift = Vector3::between(center, toward).normalized()?;
        return Ok(point.translated(shift * 1e-5));
    }
    Ok(point)
}

/// The file-local `ComputeAttach`: a position brought onto the arc from the first to the
/// second attach point (the nearer arc end when outside).
pub(super) fn compute_attach<C: IdenticConic>(
    conic: &C,
    first: Point3,
    second: Point3,
    position: Point3,
) -> Result<Point3, Failure> {
    let current = off_center(conic, position, first)?;
    let mut parameter = conic.free_param(current)?;
    let start = conic.param(first)?;
    let end = conic.param(second)?;
    let mut end_m = end;
    let mut delta = end_m - start;
    if delta < 0.0 {
        delta += 2.0 * PI;
        end_m += 2.0 * PI;
    }
    end_m -= start;
    let middle_out = end_m / 2.0 + PI;
    let mut relative = parameter;
    if relative < start {
        relative = relative + 2.0 * PI - start;
        if relative > end_m {
            parameter = if relative > middle_out { start } else { end };
        }
    } else if relative > start + delta {
        relative -= start;
        parameter = if relative > middle_out { start } else { end };
    }
    Ok(conic.at(parameter))
}

impl PrsDimIdenticRelation {
    /// `ComputeTwoCirclesPresentation` / `ComputeTwoEllipsesPresentation` on a conic: the
    /// attach arc for the four ends of the two curves, then the label.
    pub(super) fn compute_two_conics<C: IdenticConic>(
        &self,
        conic: &C,
        (first1, last1): (Point3, Point3),
        (first2, last2): (Point3, Point3),
        add: impl FnOnce(&Point3, &Point3, &Point3, &Point3) -> Result<(), Failure>,
    ) -> Result<(), Failure> {
        let confusion = precision::confusion();
        let complete1 = first1.is_equal(&last1, confusion);
        let complete2 = first2.is_equal(&last2, confusion);
        self.fields.write().center = conic.center();
        let seg_size = conic.seg_size();
        let mut rad = PI / 5.0;
        let automatic = self.automatic_position();
        if complete1 && complete2 {
            if automatic {
                let first = conic.param(first1)?;
                self.set_attaches(
                    conic.at(modulo_2pi(first - rad)),
                    conic.at(modulo_2pi(first + rad)),
                );
                self.set_label_beside(conic.at(first), seg_size)?;
            } else {
                self.compute_not_auto_whole(conic)?;
            }
        } else if complete1 != complete2 {
            let (first, last) = if complete1 {
                (first2, last2)
            } else {
                (first1, last1)
            };
            self.arc_presentation(conic, first, last, false)?;
        } else {
            let (pf1, pf2) = (conic.param(first1)?, conic.param(first2)?);
            let (pl1, pl2) = (conic.param(last1)?, conic.param(last2)?);
            if is_equal_2pi(pl1, pf2, confusion) || is_equal_2pi(pf1, pl2, confusion) {
                let (att, current) = if is_equal_2pi(pl1, pf2, confusion) {
                    (pl1, last1)
                } else {
                    (pf1, first1)
                };
                let max_rad = modulo_2pi(pl1 - pf1).min(modulo_2pi(pl2 - pf2)) * 3.0 / 4.0;
                if rad > max_rad {
                    rad = max_rad;
                }
                self.set_attaches(
                    conic.at(modulo_2pi(att - rad)),
                    conic.at(modulo_2pi(att + rad)),
                );
                if automatic {
                    self.set_label_beside(current, seg_size)?;
                }
            } else if is_equal_2pi(pf1, pf2, confusion) || is_equal_2pi(pl1, pl2, confusion) {
                let (first, last) = if modulo_2pi(pl1 - pf1) < modulo_2pi(pl2 - pf2) {
                    (first1, last1)
                } else {
                    (first2, last2)
                };
                self.arc_presentation(conic, first, last, false)?;
            } else {
                let (first, last, case1) =
                    overlap_arc(rad, (pf1, pl1, pf2, pl2), (first1, last1, first2, last2));
                if !automatic && case1 {
                    self.set_attaches(first, last);
                } else {
                    self.arc_presentation(conic, first, last, case1)?;
                }
            }
        }
        let f = self.fields.read().clone();
        let position = self.position();
        let attach = compute_attach(conic, f.f_attach, f.s_attach, position)?;
        add(&f.f_attach, &f.s_attach, &position, &attach)
    }

    /// The automatic or dragged arc presentation between two arc ends.
    fn arc_presentation<C: IdenticConic>(
        &self,
        conic: &C,
        first: Point3,
        last: Point3,
        is_static: bool,
    ) -> Result<(), Failure> {
        if self.automatic_position() {
            self.compute_auto_arc(conic, first, last, is_static)
        } else {
            self.compute_not_auto_arc(conic, first, last)
        }
    }
    fn set_attaches(&self, first: Point3, second: Point3) {
        let mut f = self.fields.write();
        f.f_attach = first;
        f.s_attach = second;
    }
    /// The label position: a point moved away from the center by the symbol size.
    fn set_label_beside(&self, point: Point3, seg_size: f64) -> Result<(), Failure> {
        let center = self.fields.read().center;
        let shift = Vector3::between(center, point).normalized()? * seg_size;
        self.relation_base().fields.write().position = point.translated(shift);
        Ok(())
    }

    /// `ComputeAutoArcPresentation`: the attach arc at the middle of an arc (its half-span
    /// when static or smaller than a fifth of pi), the label beside it.
    pub(super) fn compute_auto_arc<C: IdenticConic>(
        &self,
        conic: &C,
        first: Point3,
        last: Point3,
        is_static: bool,
    ) -> Result<(), Failure> {
        let seg_size = conic.seg_size();
        let mut rad = PI / 5.0;
        let start = conic.param(first)?;
        let end = conic.param(last)?;
        let max_rad = modulo_2pi(end - start) / 2.0;
        if rad > max_rad || is_static {
            rad = max_rad;
        }
        let middle = modulo_2pi(start + modulo_2pi(end - start) / 2.0);
        self.set_attaches(
            conic.at(modulo_2pi(middle - rad)),
            conic.at(modulo_2pi(middle + rad)),
        );
        self.set_label_beside(conic.at(middle), seg_size)
    }

    /// `ComputeNotAutoCircPresentation` / `ComputeNotAutoElipsPresentation`: the attach arc
    /// around the dragged position on a whole conic.
    pub(super) fn compute_not_auto_whole<C: IdenticConic>(&self, conic: &C) -> Result<(), Failure> {
        let f_attach = self.fields.read().f_attach;
        let current = off_center(conic, self.position(), f_attach)?;
        let rad = PI / 5.0;
        let parameter = conic.free_param(current)?;
        self.set_attaches(conic.at(parameter - rad), conic.at(parameter + rad));
        Ok(())
    }

    /// `ComputeNotAutoArcPresentation`: the attach arc around the dragged position kept
    /// within an arc (the whole arc when shorter than two fifths of pi).
    pub(super) fn compute_not_auto_arc<C: IdenticConic>(
        &self,
        conic: &C,
        first: Point3,
        last: Point3,
    ) -> Result<(), Failure> {
        let start = conic.param(first)?;
        let end = conic.param(last)?;
        let delta = modulo_2pi(end - start) / 2.0;
        let rad = PI / 5.0;
        if delta < rad {
            self.set_attaches(first, last);
            return Ok(());
        }
        let a = conic.at(modulo_2pi(start + rad));
        let b = conic.at(modulo_2pi(end - rad));
        let current = compute_attach(conic, a, b, self.position())?;
        let parameter = conic.free_param(current)?;
        self.set_attaches(conic.at(parameter - rad), conic.at(parameter + rad));
        Ok(())
    }

    /// The circle presentation: the plane frame, the center and the attach arc.
    pub(super) fn add_circle_label(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        circle: &Circle3,
        ends: ((Point3, Point3), (Point3, Point3)),
    ) -> Result<(), Failure> {
        let frame = self
            .plane()
            .get()
            .ok_or_else(null_object)?
            .pln()
            .position()
            .ax2()?;
        let drawer = self.attributes();
        let text = ExtendedString::from_utf8(" ==");
        self.compute_two_conics(circle, ends.0, ends.1, |f, s, position, attach| {
            let center = self.fields.read().center;
            DsgPrsIdenticPresentation::add_arc_point(
                prs, &drawer, &text, &frame, &center, f, s, position, attach,
            )
        })
    }

    /// The ellipse presentation: the ellipse and the attach arc.
    pub(super) fn add_ellipse_label(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        ellipse: &Ellipse3,
        ends: ((Point3, Point3), (Point3, Point3)),
    ) -> Result<(), Failure> {
        let drawer = self.attributes();
        let text = ExtendedString::from_utf8(" ==");
        self.compute_two_conics(ellipse, ends.0, ends.1, |f, s, position, attach| {
            DsgPrsIdenticPresentation::add_ellipse(
                prs, &drawer, &text, ellipse, f, s, position, attach,
            )
        })
    }
}

/// The arc shared by two overlapping arcs from their parameters relative to the first start,
/// and whether it is the static case 1.
fn overlap_arc(
    rad: f64,
    (pf1, pl1, pf2, pl2): (f64, f64, f64, f64),
    (first1, last1, first2, last2): (Point3, Point3, Point3, Point3),
) -> (Point3, Point3, bool) {
    let pl1m = modulo_2pi(pl1 - pf1);
    let pf2m = modulo_2pi(pf2 - pf1);
    let pl2m = modulo_2pi(pl2 - pf1);
    if pl1m < pf2m {
        if pl1m < pl2m {
            if pl2m < pf2m {
                return (first1, last1, false);
            }
            let deltap1 = modulo_2pi(pf1 - pl2);
            let deltap2 = modulo_2pi(pf2 - pl1);
            if (deltap1 < deltap2 && deltap1 < 2.0 * rad)
                || (deltap2 < deltap1 && deltap2 > 2.0 * rad)
            {
                return (last1, first2, true);
            }
            return (last2, first1, true);
        }
        return (first1, last2, false);
    }
    if pl1m < pl2m {
        return (first2, last1, false);
    }
    if pl2m > pf2m {
        return (first2, last2, false);
    }
    let deltap1 = modulo_2pi(pl1 - pf2);
    let deltap2 = modulo_2pi(pl2 - pf1);
    if (deltap1 < deltap2 && deltap1 < 2.0 * rad) || (deltap2 < deltap1 && deltap2 > 2.0 * rad) {
        (first1, last2, false)
    } else {
        (first2, last1, false)
    }
}
