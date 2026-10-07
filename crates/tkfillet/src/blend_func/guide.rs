//! The positioning on the guide of a two-surface blend: the guide point, its derivatives
//! and the section plane normal with its derivatives (the `switch (myTOrder)` block of
//! BlendFunc_ConstRad::ComputeValues and BlendFunc_EvolRad::ComputeValues).
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The guide values the native code keeps in `static thread_local` variables: they
/// survive between calls and are shared by every function object of the same class in a
/// thread, so a cached parameter may read values another object computed (kept as is).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GuideScratch {
    pub ptgui: Point3,
    pub d1gui: Vector3,
    pub d2gui: Vector3,
    pub d3gui: Vector3,
    pub invnormtg: f64,
    pub dinvnormtg: f64,
}

/// The section plane normal and its first two derivatives along the guide.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PlaneFrame {
    pub nplan: Vector3,
    pub dnplan: Vector3,
    pub d2nplan: Vector3,
}

fn live(curve: &Handle<dyn Adaptor3dCurve>) -> Result<&dyn Adaptor3dCurve, Failure> {
    curve
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "BlendFunc: null guide curve"))
}

/// Positions on `curve` at `t` to order `order` (0, 1 or 2), writing the guide values and
/// the plane frame; false for another order, as the native `default: return false`.
pub(crate) fn position_on_guide(
    curve: &Handle<dyn Adaptor3dCurve>,
    t: f64,
    order: i32,
    scratch: &mut GuideScratch,
    plane: &mut PlaneFrame,
) -> Result<bool, Failure> {
    let curve = live(curve)?;
    match order {
        0 => {
            (scratch.ptgui, scratch.d1gui) = curve.d1(t)?;
            plane.nplan = scratch.d1gui.normalized()?;
        }
        1 => {
            (scratch.ptgui, scratch.d1gui, scratch.d2gui) = curve.d2(t)?;
            first_order(scratch, plane)?;
        }
        2 => {
            (scratch.ptgui, scratch.d1gui, scratch.d2gui, scratch.d3gui) = curve.d3(t)?;
            first_order(scratch, plane)?;
            second_order(scratch, plane);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// `nplan`, `invnormtg` and `dnplan` from the first two guide derivatives.
fn first_order(scratch: &mut GuideScratch, plane: &mut PlaneFrame) -> Result<(), Failure> {
    plane.nplan = scratch.d1gui.normalized()?;
    scratch.invnormtg = 1. / scratch.d1gui.magnitude();
    let invnormtg = scratch.invnormtg;
    plane.dnplan.set_linear_form(
        invnormtg,
        scratch.d2gui,
        -invnormtg * plane.nplan.dot(scratch.d2gui),
        plane.nplan,
    );
    Ok(())
}

/// `dinvnormtg` and `d2nplan` from the third guide derivative.
fn second_order(scratch: &mut GuideScratch, plane: &mut PlaneFrame) {
    let (nplan, dnplan) = (plane.nplan, plane.dnplan);
    let (d2gui, d3gui, invnormtg) = (scratch.d2gui, scratch.d3gui, scratch.invnormtg);
    scratch.dinvnormtg = -nplan.dot(d2gui) * invnormtg * invnormtg;
    plane
        .d2nplan
        .set_linear_form(invnormtg, d3gui, scratch.dinvnormtg, d2gui);
    let aux =
        scratch.dinvnormtg * nplan.dot(d2gui) + invnormtg * (dnplan.dot(d2gui) + nplan.dot(d3gui));
    let previous = plane.d2nplan;
    plane.d2nplan.set_linear_form_offset(
        -invnormtg * nplan.dot(d2gui),
        dnplan,
        -aux,
        nplan,
        previous,
    );
}
