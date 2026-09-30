// Rust translation of the inherited members of ShapeExtend_CompositeSurface.cxx (from
// Geom_Geometry and Geom_Surface) and its Value(pnt) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Geom_Surface` members of a composite surface: each evaluation goes to the patch
//! that holds the global parameters, at the local parameters.
use super::{ShapeExtendCompositeSurface, SurfaceGrid};
use crate::shape_extend::ShapeExtendParametrisation;
use rustycad_tkernel::ncollection::HArray2;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomCurve, GeomGeometry, GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
    SurfaceBounds, dump_surface,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Transform3, Vector3};
use std::io;

impl ShapeExtendCompositeSurface {
    /// The patch that holds the global `(u, v)` and the local parameters on it.
    fn located(&self, u: f64, v: f64) -> Result<(Handle<dyn GeomSurface>, Point2), Failure> {
        let (i, j) = (self.locate_u_parameter(u)?, self.locate_v_parameter(v)?);
        let local = self.global_to_local(i, j, Point2::new(u, v))?;
        Ok((self.live_patch(i, j)?, local))
    }

    /// `Value(pnt)`: the point of the global parameters `pnt`; `surface.value_at(p)?`.
    pub fn value_at(&self, pnt: Point2) -> Result<Point3, Failure> {
        let (patch, local) = self.located(pnt.x(), pnt.y())?;
        patch.d0(local.x(), local.y())
    }

    /// The bounds as `Bounds` computes them: the first and last joints.
    fn joint_bounds(&self) -> Result<SurfaceBounds, Failure> {
        let (u1, v1) = (self.u_joint_value(1)?, self.v_joint_value(1)?);
        let u2 = self.u_joint_value(self.nb_u_patches()? + 1)?;
        let v2 = self.v_joint_value(self.nb_v_patches()? + 1)?;
        Ok((u1, u2, v1, v2))
    }

    /// The copied grid of `Copy`: `[1, NbU] x [1, NbV]`, each patch copied.
    fn copied_grid(&self) -> Result<SurfaceGrid, Failure> {
        let (nb_u, nb_v) = (self.nb_u_patches()?, self.nb_v_patches()?);
        let grid = HArray2::with_value(1, nb_u, 1, nb_v, Handle::<dyn GeomSurface>::default())
            .map_err(|e| e.into_failure("ShapeExtend_CompositeSurface::Copy"))?;
        for i in 1..=nb_u {
            for j in 1..=nb_v {
                let copy = self.live_patch(i, j)?.copy()?.cast::<dyn GeomSurface>();
                grid.change_array2()
                    .set_value(i, j, copy)
                    .map_err(|e| e.into_failure("ShapeExtend_CompositeSurface::Copy"))?;
            }
        }
        Ok(Handle::new(grid))
    }
}

impl GeomGeometry for ShapeExtendCompositeSurface {
    /// `Transform(T)`: transforms each patch; nothing without a grid.
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        if self.patches().is_null() {
            return Ok(());
        }
        for i in 1..=self.nb_u_patches()? {
            for j in 1..=self.nb_v_patches()? {
                self.live_patch(i, j)?.transform(transform)?;
            }
        }
        Ok(())
    }

    /// `Copy()`: a composite surface of copies of the patches, initialized with the natural
    /// parametrisation (the joint values are not copied, as natively); an empty surface
    /// without a grid.
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let surface = ShapeExtendCompositeSurface::new();
        if !self.patches().is_null() {
            surface.init(&self.copied_grid()?, ShapeExtendParametrisation::Natural)?;
        }
        Ok(Handle::new(surface).cast())
    }

    /// `Geom_Surface::DumpJson`, which the class does not override.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_surface(stream, depth)
    }
}

impl GeomSurface for ShapeExtendCompositeSurface {
    /// Does nothing, as natively (`UReverse`).
    fn u_reverse(&self) -> Result<(), Failure> {
        Ok(())
    }
    /// `U` itself (`UReversedParameter`).
    fn u_reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(u)
    }
    /// Does nothing, as natively (`VReverse`).
    fn v_reverse(&self) -> Result<(), Failure> {
        Ok(())
    }
    /// `V` itself (`VReversedParameter`).
    fn v_reversed_parameter(&self, v: f64) -> Result<f64, Failure> {
        Ok(v)
    }
    /// The first and last joint values (`Bounds`). Without a grid or joint values (a null
    /// dereference natively) the bounds are all zero.
    fn bounds(&self) -> SurfaceBounds {
        self.joint_bounds().unwrap_or((0.0, 0.0, 0.0, 0.0))
    }
    /// The flag `CheckConnectivity` sets (`IsUClosed`); false before any check (the
    /// native field is uninitialized then).
    fn is_u_closed(&self) -> Result<bool, Failure> {
        Ok(self.fields.read().u_closed)
    }
    /// The flag `CheckConnectivity` sets (`IsVClosed`).
    fn is_v_closed(&self) -> Result<bool, Failure> {
        Ok(self.fields.read().v_closed)
    }
    fn is_u_periodic(&self) -> bool {
        false
    }
    fn is_v_periodic(&self) -> bool {
        false
    }
    /// A null curve (`UIso`).
    fn u_iso(&self, _u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(Handle::default())
    }
    /// A null curve (`VIso`).
    fn v_iso(&self, _v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(Handle::default())
    }
    fn continuity(&self) -> Shape {
        Shape::C0
    }
    /// `N <= 0` (`IsCNu`).
    fn is_cn_u(&self, n: i32) -> Result<bool, Failure> {
        Ok(n <= 0)
    }
    /// `N <= 0` (`IsCNv`).
    fn is_cn_v(&self, n: i32) -> Result<bool, Failure> {
        Ok(n <= 0)
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        let (patch, local) = self.located(u, v)?;
        patch.eval_d0(local.x(), local.y())
    }
    /// The derivatives of the patch, not scaled to the global parameters, as natively.
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        let (patch, local) = self.located(u, v)?;
        patch.eval_d1(local.x(), local.y())
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        let (patch, local) = self.located(u, v)?;
        patch.eval_d2(local.x(), local.y())
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        let (patch, local) = self.located(u, v)?;
        patch.eval_d3(local.x(), local.y())
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        let (patch, local) = self.located(u, v)?;
        patch.eval_dn(local.x(), local.y(), nu, nv)
    }
}
