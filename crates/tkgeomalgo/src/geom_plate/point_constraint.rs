// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_PointConstraint.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomPlate_PointConstraint`: a point (and derivative) constraint of a plate.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::{GeomSurface, GeomSurfaceD2};
use rustycad_tkgeombase::geom_lprop::SLProps;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};
use std::sync::RwLockWriteGuard;

#[derive(Clone)]
struct PointConstraintState {
    order: i32,
    point: Point3,
    pt2d: Point2,
    // The native `mySurf` member is written and never read; the surface lives in
    // the local properties.
    d2: GeomSurfaceD2,
    u: f64,
    v: f64,
    tol_dist: f64,
    tol_ang: f64,
    tol_curv: f64,
    has_pnt2d_on_surf: bool,
}

/// A point constraint of a plate: a 3d point of order G0 (or none), or a point of a
/// surface at `(U, V)` with the derivatives there, of order up to G2
/// (`GeomPlate_PointConstraint`). The builder and the caller share it, so its fields
/// sit behind a lock (docs/geomalgo-design.md section 19).
///
/// ```
/// use rustycad_tkgeomalgo::geom_plate::GeomPlatePointConstraint;
/// use rustycad_tkmath::gp::Point3;
/// let c = GeomPlatePointConstraint::with_point(Point3::new(1.0, 2.0, 3.0), 0, 1e-4).unwrap();
/// assert_eq!(c.d0(), Point3::new(1.0, 2.0, 3.0));
/// ```
pub struct GeomPlatePointConstraint {
    parent: StandardTransient,
    state: SharedState<PointConstraintState>,
    lprop: SharedState<SLProps>,
}
impl_transient!(
    GeomPlatePointConstraint,
    "GeomPlate_PointConstraint",
    StandardTransient,
    parent
);

impl GeomPlatePointConstraint {
    fn from_parts(state: PointConstraintState, lprop: SLProps) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(state),
            lprop: SharedState::new(lprop),
        }
    }

    /// The point `pt` of order `order` with the distance tolerance `tol_dist`
    /// (`GeomPlate_PointConstraint(Pt, Order, TolDist)`; the native default of
    /// `TolDist` is 1e-4). Standard_Failure unless `order` is -1, 0 or 1 (the message
    /// says "0 or -1", but the native test accepts 1).
    /// `GeomPlatePointConstraint::with_point(p, 0, 1e-4)?`.
    pub fn with_point(pt: Point3, order: i32, tol_dist: f64) -> Result<Self, Failure> {
        let state = PointConstraintState {
            order,
            point: pt,
            pt2d: Point2::default(),
            d2: GeomSurfaceD2::default(),
            u: 0.0,
            v: 0.0,
            tol_dist,
            tol_ang: 0.0,
            tol_curv: 0.0,
            has_pnt2d_on_surf: false,
        };
        let constraint = Self::from_parts(state, SLProps::without_surface(2, tol_dist)?);
        if !(-1..=1).contains(&order) {
            return Err(Failure::new(&format!(
                "GeomPlate_PointConstraint : the constraint must 0 or -1 with a point; got order {order}"
            )));
        }
        Ok(constraint)
    }

    /// The point of `surf` at `(u, v)`, with its first and second derivatives, of
    /// order `order` (`GeomPlate_PointConstraint(U, V, Surf, Order, TolDist, TolAng,
    /// TolCurv)`; the native defaults are 1e-4, 0.01 and 0.1). The order is not
    /// checked, as natively. Standard_NullObject for a null surface (a null
    /// dereference natively). `GeomPlatePointConstraint::on_surface(0.5, 0.5, &s, 1, 1e-4, 0.01, 0.1)?`.
    pub fn on_surface(
        u: f64,
        v: f64,
        surf: &Handle<dyn GeomSurface>,
        order: i32,
        tol_dist: f64,
        tol_ang: f64,
        tol_curv: f64,
    ) -> Result<Self, Failure> {
        let mut lprop: SLProps = SLProps::without_surface(2, tol_dist)?;
        let geometry = surf.get().ok_or_else(|| {
            errors::null_object(
                "GeomPlate_PointConstraint: the surface is null; expected a Geom_Surface",
            )
        })?;
        let d2: GeomSurfaceD2 = geometry.d2(u, v)?;
        lprop.set_surface(surf.clone());
        let state = PointConstraintState {
            order,
            point: d2.point,
            pt2d: Point2::default(),
            d2,
            u,
            v,
            tol_dist,
            tol_ang,
            tol_curv,
            has_pnt2d_on_surf: false,
        };
        Ok(Self::from_parts(state, lprop))
    }

    /// Sets the order (`SetOrder`); `c.set_order(1)`.
    pub fn set_order(&self, order: i32) {
        self.state.write().order = order;
    }

    /// The order: -1 (none), 0, 1 or 2 (`Order`); `c.order()`.
    pub fn order(&self) -> i32 {
        self.state.read().order
    }

    /// Sets the distance tolerance (`SetG0Criterion`); `c.set_g0_criterion(1e-5)`.
    pub fn set_g0_criterion(&self, tol_dist: f64) {
        self.state.write().tol_dist = tol_dist;
    }

    /// Sets the angular tolerance (`SetG1Criterion`); `c.set_g1_criterion(0.01)`.
    pub fn set_g1_criterion(&self, tol_ang: f64) {
        self.state.write().tol_ang = tol_ang;
    }

    /// Sets the curvature tolerance (`SetG2Criterion`); `c.set_g2_criterion(0.1)`.
    pub fn set_g2_criterion(&self, tol_curv: f64) {
        self.state.write().tol_curv = tol_curv;
    }

    /// The distance tolerance (`G0Criterion`); `c.g0_criterion()`.
    pub fn g0_criterion(&self) -> f64 {
        self.state.read().tol_dist
    }

    /// The angular tolerance (`G1Criterion`); `c.g1_criterion()`.
    pub fn g1_criterion(&self) -> f64 {
        self.state.read().tol_ang
    }

    /// The curvature tolerance (`G2Criterion`); `c.g2_criterion()`.
    pub fn g2_criterion(&self) -> f64 {
        self.state.read().tol_curv
    }

    /// The point (`D0(P)`); `c.d0()`.
    pub fn d0(&self) -> Point3 {
        self.state.read().point
    }

    /// The point and the first derivatives of the surface, null vectors for a point
    /// constraint (`D1(P, V1, V2)`); `let (p, du, dv) = c.d1();`.
    pub fn d1(&self) -> (Point3, Vector3, Vector3) {
        let state = self.state.read();
        (state.point, state.d2.d1u, state.d2.d1v)
    }

    /// The point and the derivatives up to order 2 (`D2(P, V1, V2, V3, V4, V5)`);
    /// `c.d2().d2uv`.
    pub fn d2(&self) -> GeomSurfaceD2 {
        let state = self.state.read();
        GeomSurfaceD2 {
            point: state.point,
            ..state.d2
        }
    }

    /// Whether a parameter on the plate's initial surface is set (`HasPnt2dOnSurf`);
    /// `c.has_pnt2d_on_surf()`.
    pub fn has_pnt2d_on_surf(&self) -> bool {
        self.state.read().has_pnt2d_on_surf
    }

    /// Sets the parameter on the initial surface (`SetPnt2dOnSurf`);
    /// `c.set_pnt2d_on_surf(Point2::new(0.5, 0.5))`.
    pub fn set_pnt2d_on_surf(&self, pnt: Point2) {
        let mut state = self.state.write();
        state.pt2d = pnt;
        state.has_pnt2d_on_surf = true;
    }

    /// The parameter on the initial surface, the origin until set (`Pnt2dOnSurf`);
    /// `c.pnt2d_on_surf()`.
    pub fn pnt2d_on_surf(&self) -> Point2 {
        self.state.read().pt2d
    }

    /// The local properties of the surface at `(U, V)` (`LPropSurf`): they are set
    /// to the constraint parameters, then returned for the caller to update.
    /// Standard_NullObject for a point constraint without surface (a null dereference
    /// natively). `c.lprop_surf()?.value()`.
    pub fn lprop_surf(&self) -> Result<RwLockWriteGuard<'_, SLProps>, Failure> {
        let (u, v) = {
            let state = self.state.read();
            (state.u, state.v)
        };
        let mut lprop = self.lprop.write();
        lprop.set_parameters(u, v)?;
        Ok(lprop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkg3d::geom::GeomSphericalSurface;
    use rustycad_tkmath::gp::Axis3;

    fn sphere() -> Handle<dyn GeomSurface> {
        Handle::new(GeomSphericalSurface::new(Axis3::default(), 2.0).unwrap()).cast()
    }

    #[test]
    fn point_constraint_checks_order_and_keeps_tolerances() {
        assert!(GeomPlatePointConstraint::with_point(Point3::default(), 2, 1e-4).is_err());
        assert!(GeomPlatePointConstraint::with_point(Point3::default(), -2, 1e-4).is_err());
        let c = GeomPlatePointConstraint::with_point(Point3::new(1.0, 2.0, 3.0), 1, 1e-3).unwrap();
        assert_eq!(
            (c.order(), c.g0_criterion(), c.g1_criterion()),
            (1, 1e-3, 0.0)
        );
        c.set_g1_criterion(0.5);
        c.set_order(2);
        assert_eq!((c.order(), c.g1_criterion()), (2, 0.5));
        assert!(c.lprop_surf().is_err());
    }

    #[test]
    fn surface_constraint_reads_the_derivatives() {
        let s = sphere();
        let c = GeomPlatePointConstraint::on_surface(0.3, 0.2, &s, 2, 1e-4, 0.01, 0.1).unwrap();
        let d2 = s.get().unwrap().d2(0.3, 0.2).unwrap();
        assert_eq!(c.d2(), d2);
        assert_eq!(c.d1(), (d2.point, d2.d1u, d2.d1v));
        assert!(!c.has_pnt2d_on_surf());
        c.set_pnt2d_on_surf(Point2::new(0.1, 0.2));
        assert!(c.has_pnt2d_on_surf());
        assert_eq!(c.pnt2d_on_surf(), Point2::new(0.1, 0.2));
        assert_eq!(c.lprop_surf().unwrap().value(), d2.point);
    }
}
