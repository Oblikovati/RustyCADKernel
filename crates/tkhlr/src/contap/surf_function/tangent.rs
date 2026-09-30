use super::super::ContapSurfProps;
use super::ContapSurfFunction;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool as hst;
use rustycad_tkmath::gp::{Direction2, Vector3};
impl ContapSurfFunction {
    /// IsTangent: the gradient (computed at the last point when needed) is null, or the
    /// 3D tangent of the contour is shorter than the tolerance; also computes the 2D
    /// and 3D tangents; `f.is_tangent()?`.
    pub fn is_tangent(&mut self) -> Result<bool, Failure> {
        if self.computed {
            return Ok(self.tangent);
        }
        self.computed = true;
        self.ensure_gradient()?;
        self.tangent = false;
        let d = (self.fpu * self.fpu + self.fpv * self.fpv).sqrt();
        if d <= rustycad_tkmath::gp::resolution() {
            self.tangent = true;
            return Ok(true);
        }
        self.update_tangent_direction()?;
        Ok(self.tangent)
    }

    fn update_tangent_direction(&mut self) -> Result<(), Failure> {
        self.d2d = Direction2::new(-self.fpv, self.fpu)?;
        let d1 = hst::d1(&self.my_surf, self.usol, self.vsol)?;
        self.solpt = d1.point;
        let mut d3dxyz = -self.fpv * *d1.d1u.xyz();
        d3dxyz += self.fpu * *d1.d1v.xyz();
        self.d3d.set_xyz(d3dxyz);
        if self.d3d.magnitude() <= self.tol {
            self.tangent = true;
        }
        Ok(())
    }
    fn ensure_gradient(&mut self) -> Result<(), Failure> {
        if self.derived {
            return Ok(());
        }
        let (point, normal, du, dv) =
            ContapSurfProps::norm_and_dn(&self.my_surf, self.usol, self.vsol)?;
        self.solpt = point;
        if let Some((gu, gv)) = self.gradient_of(normal, du, dv)? {
            (self.fpu, self.fpv) = (gu, gv);
        }
        self.derived = true;
        Ok(())
    }
    /// Direction3d: the 3D tangent of the contour; StdFail_UndefinedDerivative when
    /// tangent; `f.direction3d()?`.
    pub fn direction3d(&mut self) -> Result<Vector3, Failure> {
        if self.is_tangent()? {
            return Err(undefined_derivative("Direction3d"));
        }
        Ok(self.d3d)
    }

    /// Direction2d: the 2D tangent of the contour; StdFail_UndefinedDerivative when
    /// tangent; `f.direction2d()?`.
    pub fn direction2d(&mut self) -> Result<Direction2, Failure> {
        if self.is_tangent()? {
            return Err(undefined_derivative("Direction2d"));
        }
        Ok(self.d2d)
    }
}

/// StdFail_UndefinedDerivative of a tangent point.
fn undefined_derivative(member: &str) -> Failure {
    Failure::with_message(
        "StdFail_UndefinedDerivative",
        &format!("Contap_SurfFunction::{member}: the point is tangent, expected a regular point"),
    )
}
