// Rust translation of ModelingData/TKGeomBase/Approx/Approx_CurvlinFunc.cxx (EvalCase1, EvalCase2, EvalCase3, EvalCurOnSur)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The values and derivatives along the abscissa S: `dC/dS = dC/dU dU/dS` with
//! `dU/dS = L / |dC/dU|`.
use super::{CurvlinFunc, CurvlinState, null_handle};
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurveOnSurface;
use rustycad_tkg3d::geom::GeomSurfaceD2;
use rustycad_tkmath::gp::{Point3, Vector2, Vector3};

fn wrong_case(member: &str, case: i32) -> Failure {
    failure(
        "Standard_ConstructionError",
        &format!("Approx_CurvlinFunc::{member}: case {case}"),
    )
}

// Result(0..) = values, through the checked native accesses.
fn write(result: &mut Array1<'_, f64>, values: &[f64]) -> Result<(), Failure> {
    for (i, value) in (0..).zip(values) {
        result
            .set_value(i, *value)
            .map_err(|e| array_failure(e, "Approx_CurvlinFunc::Eval"))?;
    }
    Ok(())
}

/// `d2U/dS2 = -L (C' . C'') dU/dS / |C'|^3`.
fn second_parameter_derivative(length: f64, d1: Vector3, d2: Vector3, du_ds: f64, mag: f64) -> f64 {
    -length * d1.dot(d2) * du_ds / (mag * mag * mag)
}

impl CurvlinFunc {
    /// `EvalCase1(S, Order, Result)`: the point or derivative of order 0 to 2 at `s` in
    /// `Result(0..=2)`; false (and zeros) for another order; Standard_ConstructionError
    /// in another case; `f.eval_case1(0.5, 1, &mut result)?`.
    pub fn eval_case1(
        &self,
        s: f64,
        order: i32,
        result: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        let st: CurvlinState = self.snapshot();
        if st.case != 1 {
            return Err(wrong_case("EvalCase1", st.case));
        }
        let c3d = st.c3d.get().ok_or_else(|| null_handle("curve"))?;
        let u: f64 = self.get_u_parameter(c3d, s, 1)?;
        let values: [f64; 3] = match order {
            0 => {
                let c: Point3 = c3d.value(u)?;
                [c.x(), c.y(), c.z()]
            }
            1 => {
                let (_, dc_du) = c3d.d1(u)?;
                let du_ds: f64 = st.length / dc_du.magnitude();
                let dc_ds: Vector3 = dc_du * du_ds;
                [dc_ds.x(), dc_ds.y(), dc_ds.z()]
            }
            2 => {
                let (_, dc_du, d2c_du2) = c3d.d2(u)?;
                let mag: f64 = dc_du.magnitude();
                let du_ds: f64 = st.length / mag;
                let d2u_ds2: f64 =
                    second_parameter_derivative(st.length, dc_du, d2c_du2, du_ds, mag);
                let d2c_ds2: Vector3 = d2c_du2 * du_ds * du_ds + dc_du * d2u_ds2;
                [d2c_ds2.x(), d2c_ds2.y(), d2c_ds2.z()]
            }
            _ => {
                write(result, &[0.0; 3])?;
                return Ok(false);
            }
        };
        write(result, &values).map(|()| true)
    }

    /// `EvalCase2(S, Order, Result)`: `(u, v, x, y, z)` or their derivatives in
    /// `Result(0..=4)`; `f.eval_case2(0.5, 1, &mut result)?`.
    pub fn eval_case2(
        &self,
        s: f64,
        order: i32,
        result: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        let case: i32 = self.state.read().case;
        if case != 2 {
            return Err(wrong_case("EvalCase2", case));
        }
        self.eval_cur_on_sur(s, order, result, 1)
    }

    /// `EvalCase3(S, Order, Result)`: `(u1, v1, u2, v2)` and the mean of both 3D
    /// points (or derivatives) in `Result(0..=6)`; true only when both evaluations
    /// are; `f.eval_case3(0.5, 1, &mut result)?`.
    pub fn eval_case3(
        &self,
        s: f64,
        order: i32,
        result: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure> {
        let case: i32 = self.state.read().case;
        if case != 3 {
            return Err(wrong_case("EvalCase3", case));
        }
        let alloc = || {
            Array1::with_bounds(0, 4).map_err(|e| array_failure(e, "Approx_CurvlinFunc::EvalCase3"))
        };
        let (mut res1, mut res2): (Array1<'static, f64>, Array1<'static, f64>) =
            (alloc()?, alloc()?);
        let mut done: bool = self.eval_cur_on_sur(s, order, &mut res1, 1)?;
        done = self.eval_cur_on_sur(s, order, &mut res2, 2)? && done;
        let mean = |i: i32| 0.5 * (res1[i] + res2[i]);
        write(
            result,
            &[
                res1[0],
                res1[1],
                res2[0],
                res2[1],
                mean(2),
                mean(3),
                mean(4),
            ],
        )?;
        Ok(done)
    }

    /// The private `EvalCurOnSur(S, Order, Result, NumberOfCurve)`.
    fn eval_cur_on_sur(
        &self,
        s: f64,
        order: i32,
        result: &mut Array1<'_, f64>,
        number: i32,
    ) -> Result<bool, Failure> {
        let st: CurvlinState = self.snapshot();
        let (c2d, surf, length) = match number {
            1 => (
                &st.c2d1,
                &st.surf1,
                if st.case == 3 { st.length1 } else { st.length },
            ),
            2 => (&st.c2d2, &st.surf2, st.length2),
            _ => {
                return Err(failure(
                    "Standard_ConstructionError",
                    &format!("Approx_CurvlinFunc::EvalCurOnSur: curve {number}; expected 1 or 2"),
                ));
            }
        };
        // Adaptor3d_CurveOnSurface(C2D, Surf) loads the surface first, unlike the
        // Load(C2D), Load(Surf) sequence of the other members.
        let cons = Adaptor3dCurveOnSurface::with_curve(c2d, surf)?;
        let u: f64 = self.get_u_parameter(&cons, s, number)?;
        let cur2d = c2d.get().ok_or_else(|| null_handle("2D curve"))?;
        let surface = surf.get().ok_or_else(|| null_handle("surface"))?;
        let values: [f64; 5] = match order {
            0 => {
                let c2 = cur2d.value(u)?;
                let c = surface.value(c2.x(), c2.y())?;
                [c2.x(), c2.y(), c.x(), c.y(), c.z()]
            }
            1 => {
                let (c2, dc2_du) = cur2d.d1(u)?;
                let d = surface.d1(c2.x(), c2.y())?;
                first_order(dc2_du, [d.d1u, d.d1v], length)
            }
            2 => {
                let (c2, dc2_du, d2c2_du2) = cur2d.d2(u)?;
                let d: GeomSurfaceD2 = surface.d2(c2.x(), c2.y())?;
                second_order([dc2_du, d2c2_du2], &d, length)
            }
            _ => {
                write(result, &[0.0; 5])?;
                return Ok(false);
            }
        };
        write(result, &values).map(|()| true)
    }
}

// Order 1: dV/dS, dW/dS and dC/dS of the curve (V(U), W(U)) on the surface.
fn first_order(dc2_du: Vector2, [ds_dv, ds_dw]: [Vector3; 2], length: f64) -> [f64; 5] {
    let (dv_du, dw_du) = (dc2_du.x(), dc2_du.y());
    let dc_du: Vector3 = ds_dv * dv_du + ds_dw * dw_du;
    let du_ds: f64 = length / dc_du.magnitude();
    let (dv_ds, dw_ds) = (dv_du * du_ds, dw_du * du_ds);
    let dc_ds: Vector3 = dc_du * du_ds;
    [dv_ds, dw_ds, dc_ds.x(), dc_ds.y(), dc_ds.z()]
}

// Order 2: the second derivatives, with the native expression order.
fn second_order([dc2_du, d2c2_du2]: [Vector2; 2], d: &GeomSurfaceD2, length: f64) -> [f64; 5] {
    let (dv_du, dw_du) = (dc2_du.x(), dc2_du.y());
    let (d2v_du2, d2w_du2) = (d2c2_du2.x(), d2c2_du2.y());
    let (ds_dv, ds_dw) = (d.d1u, d.d1v);
    let (d2s_dv2, d2s_dw2, d2s_dvdw) = (d.d2u, d.d2v, d.d2uv);
    let dc_du: Vector3 = ds_dv * dv_du + ds_dw * dw_du;
    let d2c_du2: Vector3 = (d2s_dv2 * dv_du + d2s_dvdw * dw_du) * dv_du
        + ds_dv * d2v_du2
        + (d2s_dvdw * dv_du + d2s_dw2 * dw_du) * dw_du
        + ds_dw * d2w_du2;
    let mag: f64 = dc_du.magnitude();
    let du_ds: f64 = length / mag;
    let d2u_ds2: f64 = second_parameter_derivative(length, dc_du, d2c_du2, du_ds, mag);
    let (dv_ds, dw_ds) = (dv_du * du_ds, dw_du * du_ds);
    let d2v_ds2: f64 = d2v_du2 * du_ds * du_ds + dv_du * d2u_ds2;
    let d2w_ds2: f64 = d2w_du2 * du_ds * du_ds + dw_du * d2u_ds2;
    // The native code then recomputes d2U/dS2 without the length and never reads it.
    let d2c_ds2: Vector3 = (d2s_dv2 * dv_ds + d2s_dvdw * dw_ds) * dv_ds
        + ds_dv * d2v_ds2
        + (d2s_dw2 * dw_ds + d2s_dvdw * dv_ds) * dw_ds
        + ds_dw * d2w_ds2;
    [d2v_ds2, d2w_ds2, d2c_ds2.x(), d2c_ds2.y(), d2c_ds2.z()]
}
