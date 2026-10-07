//! The guide plane of the chamfer functions (`BlendFunc_Corde`, `BlendFunc_ConstThroat`
//! and its inverse): the guide point and derivatives at a parameter and the plane normal
//! to the guide through the point.
use super::package::live_curve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The plane `nplan.P + theD = 0` normal to the guide at `ptgui`, with the guide
/// derivatives and speed `normtg`.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ChamferPlane {
    pub(crate) ptgui: Point3,
    pub(crate) d1gui: Vector3,
    pub(crate) d2gui: Vector3,
    pub(crate) normtg: f64,
    pub(crate) nplan: Vector3,
    pub(crate) the_d: f64,
}

impl ChamferPlane {
    /// Moves the plane to `param` of `curve` (`curve->D2(Param, ...)`), field by field
    /// as natively: a null tangent (Standard_ConstructionError) leaves the guide values
    /// updated and the normal of the previous parameter; `plane.set(&guide, 1.0)?`.
    pub(crate) fn set(
        &mut self,
        curve: &Handle<dyn Adaptor3dCurve>,
        param: f64,
    ) -> Result<(), Failure> {
        (self.ptgui, self.d1gui, self.d2gui) = live_curve(curve)?.d2(param)?;
        self.normtg = self.d1gui.magnitude();
        self.nplan = self.d1gui.normalized()?;
        self.the_d = -self.nplan.xyz().dot(self.ptgui.xyz());
        Ok(())
    }

    /// The derivative of the plane normal along the guide:
    /// `dnplan.SetLinearForm(1 / normtg, d2gui, -1 / normtg * (nplan.d2gui), nplan)`.
    pub(crate) fn dnplan(&self) -> Vector3 {
        let mut dnplan = Vector3::default();
        dnplan.set_linear_form(
            1. / self.normtg,
            self.d2gui,
            -1. / self.normtg * (self.nplan.dot(self.d2gui)),
            self.nplan,
        );
        dnplan
    }

    /// The signed distance of `point` to the plane: `nplan.P + theD`.
    pub(crate) fn offset(&self, point: Point3) -> f64 {
        self.nplan.xyz().dot(point.xyz()) + self.the_d
    }
}
