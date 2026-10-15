//! `PrsDim` package functions on boxes and ellipse arcs: translation of a point to a box
//! boundary, parameter domains and ellipse apexes.
use super::PrsDim;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{self, Direction3, Ellipse3, Point3, Vector3};
use std::f64::consts::PI;

impl PrsDim {
    /// `TranslatePointToBound`: a point inside a box moved along a direction to the first box
    /// face it reaches (unchanged when outside); `PrsDim::translate_point_to_bound(&p, &d, &b)?`.
    pub fn translate_point_to_bound(
        point: &Point3,
        direction: &Direction3,
        bounds: &BoundingBox3,
    ) -> Result<Point3, Failure> {
        if bounds.is_out_point(*point) {
            return Ok(*point);
        }
        let [xmin, ymin, zmin, xmax, ymax, zmax] = bounds.get()?;
        let bound = [[xmin, xmax], [ymin, ymax], [zmin, zmax]];
        let origin = [point.x(), point.y(), point.z()];
        let dir = [direction.x(), direction.y(), direction.z()];
        let mut enlarged = bounds.clone();
        enlarged.enlarge(bounds.gap() + precision::confusion());
        let mut result = Point3::new(0.0, 0.0, 0.0);
        for axis in 0..3 {
            if dir[axis].abs() <= gp::resolution() {
                continue;
            }
            for side in bound[axis] {
                let t = (side - origin[axis]) / dir[axis];
                if t < 0.0 {
                    continue;
                }
                result = point.translated(Vector3::from(*direction) * t);
                if !enlarged.is_out_point(result) {
                    return Ok(result);
                }
            }
        }
        Ok(result)
    }

    /// `InDomain`: whether a parameter lies in a (possibly wrapping) arc of a period.
    /// Example: `PrsDim::in_domain(0.0, 1.0, 0.5)`.
    pub fn in_domain(first: f64, last: f64, parameter: f64) -> bool {
        if first >= 0.0 {
            if last > first {
                return parameter >= first && parameter <= last;
            }
            let delta = 2.0 * PI - first;
            let mut lp = last + delta;
            let mut par = parameter + delta;
            while lp > 2.0 * PI {
                lp -= 2.0 * PI;
            }
            while par > 2.0 * PI {
                par -= 2.0 * PI;
            }
            return par >= 0.0 && par <= lp;
        }
        parameter >= first + 2.0 * PI || parameter <= last
    }

    /// `DistanceFromApex`: the parameter distance from an ellipse apex to a parameter.
    /// Example: `PrsDim::distance_from_apex(&ellipse, &apex, 1.0)?`.
    pub fn distance_from_apex(
        ellipse: &Ellipse3,
        apex: &Point3,
        parameter: f64,
    ) -> Result<f64, Failure> {
        let apex_parameter = elclib::parameter(ellipse, *apex)?;
        let par = parameter;
        if apex_parameter == 0.0 || apex_parameter == PI {
            if apex_parameter == 0.0 {
                return Ok(if par < PI { par } else { 2.0 * PI - par });
            }
            return Ok(if par < PI { PI - par } else { par - PI });
        }
        if apex_parameter == PI / 2.0 {
            if par <= apex_parameter + PI && par > apex_parameter {
                return Ok(par - apex_parameter);
            }
            if par > apex_parameter + PI {
                return Ok(2.0 * PI - par + apex_parameter);
            }
            return Ok(apex_parameter - par);
        }
        if par <= apex_parameter && par >= PI / 2.0 {
            return Ok(apex_parameter - par);
        }
        if par > apex_parameter {
            return Ok(par - apex_parameter);
        }
        Ok(par + PI / 2.0)
    }

    /// `NearestApex`: the apex of an ellipse arc (positive first) within the arc, else the
    /// one nearest to its ends, with whether it lies in the arc.
    /// Example: `PrsDim::nearest_apex(&ellipse, &p, &n, 0.0, 1.0)?`.
    pub fn nearest_apex(
        ellipse: &Ellipse3,
        positive: &Point3,
        negative: &Point3,
        first: f64,
        last: f64,
    ) -> Result<(Point3, bool), Failure> {
        let parameter = elclib::parameter(ellipse, *positive)?;
        if Self::in_domain(first, last, parameter) {
            return Ok((*positive, true));
        }
        let parameter = elclib::parameter(ellipse, *negative)?;
        if Self::in_domain(first, last, parameter) {
            return Ok((*negative, true));
        }
        // `std::min` keeps its first argument unless the second is smaller.
        let min = |a: f64, b: f64| if b < a { b } else { a };
        let positive_distance = min(
            Self::distance_from_apex(ellipse, positive, first)?,
            Self::distance_from_apex(ellipse, positive, last)?,
        );
        let negative_distance = min(
            Self::distance_from_apex(ellipse, negative, first)?,
            Self::distance_from_apex(ellipse, negative, last)?,
        );
        let end = if positive_distance < negative_distance {
            *positive
        } else {
            *negative
        };
        Ok((end, false))
    }
}
