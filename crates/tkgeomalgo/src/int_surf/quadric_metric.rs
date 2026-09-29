// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntSurf/IntSurf_Quadric.cxx (the implicit
// members Distance, Gradient, ValAndGrad and Normale) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The implicit-function members of `IntSurf_Quadric`.
use super::IntSurfQuadric;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{self, Direction3, Point3, Vector3, Xyz};

/// The anonymous `hasMagnitudeForNormalization`: a norm above `gp::Resolution`.
fn has_magnitude_for_normalization(vector: &Vector3) -> bool {
    let resolution: f64 = gp::resolution();
    vector.square_magnitude() > resolution * resolution
}

/// `grad / |grad|`, or the null vector when the norm is at most 1e-14 (the cylinder,
/// sphere and torus gradients).
fn unit_or_null(grad: Vector3) -> Vector3 {
    let n: f64 = grad.magnitude();
    if n > 1e-14 {
        return grad / n;
    }
    Vector3::new(0.0, 0.0, 0.0)
}

/// A direction from a vector whose norm the caller checked to be far above
/// `gp::Resolution`, so the native `gp_Dir(gp_Vec)` cannot raise.
fn checked_direction(vector: Vector3) -> Direction3 {
    Direction3::from_vector(vector)
        .unwrap_or_else(|_| unreachable!("the vector norm is checked above gp::Resolution"))
}

impl IntSurfQuadric {
    /// The point of the torus's core circle in the meridian plane of `p`: `p` is
    /// projected on the equatorial plane, and a point on the axis uses the X direction.
    fn torus_core_point(&self, p: Point3) -> Point3 {
        let o: Point3 = self.ax3.location();
        let oz: Vector3 = Vector3::from(self.ax3.direction());
        let along: f64 = Vector3::between(o, p).dot(Vector3::from(self.ax3.direction()));
        let pp: Point3 = p.translated(oz * -along);
        let dopp: Direction3 = if o.square_distance(&pp) < 1e-14 {
            self.ax3.x_direction()
        } else {
            checked_direction(Vector3::between(o, pp))
        };
        Point3::from(o.xyz() + dopp.xyz() * self.prm1)
    }

    /// The cone parameters of `p` and the cone's point and first derivatives there.
    fn cone_d1_at(&self, p: Point3) -> (Point3, Vector3, Vector3) {
        let (u, v) = elslib::cone_parameters(self.ax3, self.prm1, self.prm2, p);
        elslib::cone_d1(u, v, self.ax3, self.prm1, self.prm2)
    }

    /// The signed distance of the cone: the radial gap to the projection on the cone,
    /// divided by the cosine of the semi-angle.
    fn cone_distance(&self, p: Point3) -> f64 {
        let dist: f64 = self.lin.distance(p);
        let (u, v) = elslib::cone_parameters(self.ax3, self.prm1, self.prm2, p);
        let pp: Point3 = elslib::cone_value(u, v, self.ax3, self.prm1, self.prm2);
        let distp: f64 = self.lin.distance(pp);
        (dist - distp) / self.prm3
    }

    /// The unit cone normal `D1u ^ D1v` oriented by the frame, or the null vector on the
    /// axis, where the native code notes that any value may be returned.
    fn cone_unit_normal(&self, d1u: Vector3, d1v: Vector3) -> Vector3 {
        let mut grad: Vector3 = d1u.crossed(d1v);
        if !self.ax3direc {
            grad.reverse();
        }
        if !has_magnitude_for_normalization(&grad) {
            return Vector3::new(0.0, 0.0, 0.0);
        }
        grad.normalized()
            .unwrap_or_else(|_| unreachable!("the norm is checked above gp::Resolution"))
    }

    /// The cylinder gradient: from the axis projection of `p` towards `p`.
    fn cylinder_gradient(&self, p: Point3) -> Vector3 {
        let mut pp: Xyz = self.lin.location().xyz();
        pp += Xyz::from(self.lin.direction()) * elclib::line_parameter(self.lin.position(), p);
        unit_or_null(Vector3::from(p.xyz() - pp))
    }

    /// The signed distance of `p` to the surface (`Distance`); 0 for an undefined
    /// quadric. `quadric.distance(point)`.
    pub fn distance(&self, p: Point3) -> f64 {
        match self.typ {
            SurfaceType::Plane => {
                self.prm1 * p.x() + self.prm2 * p.y() + self.prm3 * p.z() + self.prm4
            }
            SurfaceType::Cylinder => self.lin.distance(p) - self.prm1,
            SurfaceType::Sphere => self.lin.location().distance(&p) - self.prm1,
            SurfaceType::Cone => self.cone_distance(p),
            SurfaceType::Torus => p.distance(&self.torus_core_point(p)) - self.prm2,
            _ => 0.0,
        }
    }

    /// The unit gradient of the distance at `p` (`Gradient`), null where it is
    /// undefined (on the axis, at the centre, at the cone apex); the plane gradient is
    /// its equation normal. `quadric.gradient(point)`.
    pub fn gradient(&self, p: Point3) -> Vector3 {
        match self.typ {
            SurfaceType::Plane => Vector3::new(self.prm1, self.prm2, self.prm3),
            SurfaceType::Cylinder => self.cylinder_gradient(p),
            SurfaceType::Sphere => unit_or_null(Vector3::from(p.xyz() - self.lin.location().xyz())),
            SurfaceType::Cone => {
                let (_, d1u, d1v) = self.cone_d1_at(p);
                self.cone_unit_normal(d1u, d1v)
            }
            SurfaceType::Torus => {
                unit_or_null(Vector3::from(p.xyz() - self.torus_core_point(p).xyz()))
            }
            _ => Vector3::new(0.0, 0.0, 0.0),
        }
    }

    /// The distance and the gradient at `p` (`ValAndGrad`); `None` for an undefined
    /// quadric, whose native output arguments are left unchanged.
    /// `let (dist, grad) = quadric.val_and_grad(point).unwrap();`
    pub fn val_and_grad(&self, p: Point3) -> Option<(f64, Vector3)> {
        match self.typ {
            SurfaceType::Plane | SurfaceType::Cylinder | SurfaceType::Sphere => {
                Some((self.distance(p), self.gradient(p)))
            }
            SurfaceType::Cone => {
                // One parameter inversion serves the distance and the gradient here,
                // unlike Distance, which evaluates ConeValue.
                let dist: f64 = self.lin.distance(p);
                let (pp, d1u, d1v) = self.cone_d1_at(p);
                let distp: f64 = self.lin.distance(pp);
                Some(((dist - distp) / self.prm3, self.cone_unit_normal(d1u, d1v)))
            }
            SurfaceType::Torus => {
                let pt: Point3 = self.torus_core_point(p);
                let grad: Vector3 = unit_or_null(Vector3::from(p.xyz() - pt.xyz()));
                Some((p.distance(&pt) - self.prm2, grad))
            }
            _ => None,
        }
    }

    /// The normal at the parameters `(u, v)` (`Normale(U, V)`): unit except on the cone,
    /// whose normal is the unnormalized `D1u ^ D1v` (null near the apex) and ignores the
    /// frame handedness, as natively. Fails as [`Self::normale_at`] does.
    /// `quadric.normale(0.5, 0.25)?`.
    pub fn normale(&self, u: f64, v: f64) -> Result<Vector3, Failure> {
        match self.typ {
            SurfaceType::Plane => Ok(self.plane_normal()),
            SurfaceType::Cylinder | SurfaceType::Sphere | SurfaceType::Torus => {
                self.normale_at(self.value(u, v))
            }
            SurfaceType::Cone => {
                let (_, d1u, d1v) = elslib::cone_d1(u, v, self.ax3, self.prm1, self.prm2);
                if d1u.magnitude() < 0.0000001 {
                    return Ok(Vector3::new(0.0, 0.0, 0.0));
                }
                Ok(d1u.crossed(d1v))
            }
            _ => Ok(Vector3::new(0.0, 0.0, 0.0)),
        }
    }

    fn plane_normal(&self) -> Vector3 {
        let direction: Direction3 = self.ax3.direction();
        Vector3::from(if self.ax3direc {
            direction
        } else {
            direction.reversed()
        })
    }

    /// The unit normal at the point `p` of the surface (`Normale(P)`), oriented by the
    /// frame. Standard_ConstructionError for a point on a cylinder axis or at a sphere
    /// centre (the native `gp_Dir` of a null vector). `quadric.normale_at(point)?`.
    pub fn normale_at(&self, p: Point3) -> Result<Vector3, Failure> {
        match self.typ {
            SurfaceType::Plane => Ok(self.plane_normal()),
            SurfaceType::Cylinder => {
                let d: Direction3 = self.lin.normal(p)?.direction();
                Ok(Vector3::from(if self.ax3direc { d } else { d.reversed() }))
            }
            SurfaceType::Sphere => {
                let o: Point3 = self.ax3.location();
                let v = if self.ax3direc {
                    Vector3::between(o, p)
                } else {
                    Vector3::between(p, o)
                };
                Ok(Vector3::from(Direction3::from_vector(v)?))
            }
            SurfaceType::Cone => {
                let (u, v) = elslib::cone_parameters(self.ax3, self.prm1, self.prm2, p);
                self.normale(u, v)
            }
            SurfaceType::Torus => Ok(self.torus_normal(p)),
            _ => Ok(Vector3::new(0.0, 0.0, 0.0)),
        }
    }

    /// The torus normal at `p`: the axis direction on the core circle, otherwise from
    /// the core point towards `p` (reversed for an indirect frame).
    fn torus_normal(&self, p: Point3) -> Vector3 {
        let pt: Point3 = self.torus_core_point(p);
        if pt.square_distance(&p) < 1e-14 {
            return Vector3::from(self.ax3.direction());
        }
        let v = if self.ax3direc {
            Vector3::between(pt, p)
        } else {
            Vector3::between(p, pt)
        };
        Vector3::from(checked_direction(v))
    }
}
