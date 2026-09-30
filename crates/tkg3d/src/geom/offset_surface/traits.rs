// Rust translation of ModelingData/TKG3d/Geom/Geom_OffsetSurface.cxx (the Geom_Geometry and Geom_Surface members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The virtual members of Geom_OffsetSurface: moves, copy, dump, bounds, closure,
//! periodicity, continuity, iso curves and evaluation.
use super::{GeomOffsetSurface, undefined_derivative, undefined_value};
use crate::geom::errors;
use crate::geom::offset_surface_utils::{self as utils, OsculatingQuery};
use crate::geom::{
    GeomCircle, GeomCurve, GeomElementarySurface, GeomEllipse, GeomGeometry,
    GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomSweptSurface, SurfaceBounds,
    check_dn_orders, dump_surface,
};
use crate::geom_eval::rep_utils;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_field};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{GeneralTransform2, Point3, Transform3, Vector3};
use std::io;

impl GeomGeometry for GeomOffsetSurface {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.basis.transform(transform)?;
        // The native code multiplies by the signed scale factor.
        state.offset_value *= transform.scale_factor();
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(Self::copy_of(self)?).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let equivalent: Handle<dyn GeomSurface> = self.direct_rep_surface();
        let state = self.state.read();
        dump_class_name(stream, "Geom_OffsetSurface")?;
        dump_base(stream, depth, dump_surface)?;
        dump_field(stream, depth, "basisSurf", |nested, level| {
            state.basis.dump_json(nested, level)
        })?;
        // DumpFieldToName drops the "an" prefix of anEquivSurface.
        if let Some(equivalent) = equivalent.get() {
            dump_field(stream, depth, "EquivSurface", |nested, level| {
                equivalent.dump_json(nested, level)
            })?;
        }
        dump::dump_real(stream, "offsetValue", state.offset_value)?;
        dump::dump_integer(stream, "BasisSurfContinuity", state.basis_continuity as i32)
    }
}

// The IsUClosed/IsVClosed classification of the (possibly trimmed) basis: whether
// the untrimmed basis is elementary, an extrusion of a circle or an ellipse, or a
// revolution.
fn swept_closure(basis: &Handle<dyn GeomSurface>) -> (bool, bool, bool) {
    let elementary: bool = basis.cast::<dyn GeomElementarySurface>().is_some();
    let closed_extrusion: bool = basis
        .cast::<GeomSurfaceOfLinearExtrusion>()
        .get()
        .is_some_and(|e| {
            let curve: Handle<dyn GeomCurve> = e.basis_curve();
            curve.cast::<GeomCircle>().is_some() || curve.cast::<GeomEllipse>().is_some()
        });
    (
        elementary,
        closed_extrusion,
        basis.cast::<GeomSurfaceOfRevolution>().is_some(),
    )
}

impl GeomOffsetSurface {
    fn osc_query(&self) -> Option<crate::geom::GeomOsculatingSurface> {
        self.state.read().osc.clone()
    }
    fn evaluation_inputs(&self) -> (Handle<dyn GeomSurface>, f64) {
        let state = self.state.read();
        (state.basis.clone(), state.offset_value)
    }
}

impl GeomSurface for GeomOffsetSurface {
    fn u_reverse(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.basis.u_reverse()?;
        state.offset_value = -state.offset_value;
        Ok(())
    }
    fn u_reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        self.basis_surface().u_reversed_parameter(u)
    }
    fn v_reverse(&self) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.eval_rep = Handle::default();
        state.basis.v_reverse()?;
        state.offset_value = -state.offset_value;
        Ok(())
    }
    fn v_reversed_parameter(&self, v: f64) -> Result<f64, Failure> {
        self.basis_surface().v_reversed_parameter(v)
    }
    /// The basis transforms the parameters, then the equivalent surface, if any,
    /// transforms them again, as natively.
    fn transform_parameters(&self, u: f64, v: f64, transform: Transform3) -> (f64, f64) {
        let (u, v) = self.basis_surface().transform_parameters(u, v, transform);
        let equivalent = self.direct_rep_surface();
        match equivalent.get() {
            Some(equivalent) => equivalent.transform_parameters(u, v, transform),
            None => (u, v),
        }
    }
    fn parametric_transformation(
        &self,
        transform: Transform3,
    ) -> Result<GeneralTransform2, Failure> {
        self.basis_surface().parametric_transformation(transform)
    }
    fn bounds(&self) -> SurfaceBounds {
        self.basis_surface().bounds()
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        let basis: Handle<dyn GeomSurface> = self.basis_surface();
        let trimmed: Handle<GeomRectangularTrimmedSurface> = basis.cast();
        let inner = match trimmed.get() {
            Some(trimmed) => trimmed.basis_surface(),
            None => basis.clone(),
        };
        let (elementary, closed_extrusion, revolution) = swept_closure(&inner);
        if trimmed.is_some() {
            return if elementary || closed_extrusion || revolution {
                basis.is_u_closed()
            } else {
                Ok(false)
            };
        }
        if elementary {
            return basis.is_u_closed();
        }
        Ok(closed_extrusion || revolution)
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        let basis: Handle<dyn GeomSurface> = self.basis_surface();
        let trimmed: Handle<GeomRectangularTrimmedSurface> = basis.cast();
        let inner = match trimmed.get() {
            Some(trimmed) => trimmed.basis_surface(),
            None => basis.clone(),
        };
        if inner.cast::<dyn GeomElementarySurface>().is_some() {
            return basis.is_v_closed();
        }
        Ok(false)
    }
    fn is_u_periodic(&self) -> bool {
        self.basis_surface().is_u_periodic()
    }
    fn u_period(&self) -> Result<f64, Failure> {
        self.basis_surface().u_period()
    }
    fn is_v_periodic(&self) -> bool {
        self.basis_surface().is_v_periodic()
    }
    fn v_period(&self) -> Result<f64, Failure> {
        self.basis_surface().v_period()
    }
    fn u_iso(&self, u: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.offset_u_iso(u)
    }
    fn v_iso(&self, v: f64) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.offset_v_iso(v)
    }
    fn continuity(&self) -> Shape {
        match self.basis_surf_continuity() {
            Shape::C2 => Shape::C1,
            Shape::C3 => Shape::C2,
            Shape::CN => Shape::CN,
            _ => Shape::C0,
        }
    }
    fn is_cn_u(&self, n: i32) -> Result<bool, Failure> {
        check_order("IsCNu", n)?;
        self.basis_surface().is_cn_u(n + 1)
    }
    fn is_cn_v(&self, n: i32) -> Result<bool, Failure> {
        check_order("IsCNv", n)?;
        self.basis_surface().is_cn_v(n + 1)
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        if let Some(point) = rep_utils::try_eval_surface_d0(&self.eval_representation(), u, v)? {
            return Ok(point);
        }
        let (basis, offset) = self.evaluation_inputs();
        let osc = self.osc_query();
        let query: Option<&dyn OsculatingQuery> = osc.as_ref().map(|o| o as &dyn OsculatingQuery);
        utils::evaluate_d0((u, v), &*basis, offset, query)?.ok_or_else(|| undefined_value("EvalD0"))
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        if let Some(result) = rep_utils::try_eval_surface_d1(&self.eval_representation(), u, v)? {
            return Ok(result);
        }
        let (basis, offset) = self.evaluation_inputs();
        let osc = self.osc_query();
        let query: Option<&dyn OsculatingQuery> = osc.as_ref().map(|o| o as &dyn OsculatingQuery);
        utils::evaluate_d1((u, v), &*basis, offset, query)?
            .ok_or_else(|| undefined_derivative("EvalD1"))
    }
    fn eval_d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        if let Some(result) = rep_utils::try_eval_surface_d2(&self.eval_representation(), u, v)? {
            return Ok(result);
        }
        let (basis, offset) = self.evaluation_inputs();
        let osc = self.osc_query();
        let query: Option<&dyn OsculatingQuery> = osc.as_ref().map(|o| o as &dyn OsculatingQuery);
        utils::evaluate_d2((u, v), &*basis, offset, query)?
            .ok_or_else(|| undefined_derivative("EvalD2"))
    }
    fn eval_d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        if let Some(result) = rep_utils::try_eval_surface_d3(&self.eval_representation(), u, v)? {
            return Ok(result);
        }
        let (basis, offset) = self.evaluation_inputs();
        let osc = self.osc_query();
        let query: Option<&dyn OsculatingQuery> = osc.as_ref().map(|o| o as &dyn OsculatingQuery);
        utils::evaluate_d3((u, v), &*basis, offset, query)?
            .ok_or_else(|| undefined_derivative("EvalD3"))
    }
    fn eval_dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        check_dn_orders("Geom_OffsetSurface", nu, nv)?;
        if let Some(result) =
            rep_utils::try_eval_surface_dn(&self.eval_representation(), u, v, nu, nv)?
        {
            return Ok(result);
        }
        let (basis, offset) = self.evaluation_inputs();
        let osc = self.osc_query();
        let query: Option<&dyn OsculatingQuery> = osc.as_ref().map(|o| o as &dyn OsculatingQuery);
        utils::evaluate_dn((u, v), (nu, nv), &*basis, offset, query)?
            .ok_or_else(|| undefined_derivative("EvalDN"))
    }
}

fn check_order(member: &str, n: i32) -> Result<(), Failure> {
    if n < 0 {
        return Err(errors::range(&format!(
            "Geom_OffsetSurface::{member}: N = {n}; expected N >= 0"
        )));
    }
    Ok(())
}
