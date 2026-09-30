// Rust translation of the joint members of ShapeExtend_CompositeSurface.cxx (UJointValues,
// VJointValues, UJointValue, VJointValue, SetUJointValues, SetVJointValues, SetUFirstValue,
// SetVFirstValue, ComputeJointValues, CheckConnectivity and the static LimitValue and
// GetLimitedBounds) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The joint values of a composite surface and the connectivity of its patches.
use super::{JointValues, ShapeExtendCompositeSurface, joint};
use crate::shape_extend::ShapeExtendParametrisation;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;

/// The sample count along a joint in `CheckConnectivity`.
const NPOINTS: i32 = 23;

/// A new joint array `[1, n]` of zeros.
fn new_joints(n: i32, member: &str) -> Result<JointValues, Failure> {
    let array = HArray1::with_value(1, n, 0.0)
        .map_err(|e| e.into_failure(&format!("ShapeExtend_CompositeSurface::{member}")))?;
    Ok(Handle::new(array))
}

/// `joints->SetValue(i, value)`.
fn set_joint(joints: &JointValues, i: i32, value: f64) -> Result<(), Failure> {
    joints
        .change_array1()
        .set_value(i, value)
        .map_err(|e| e.into_failure("ShapeExtend_CompositeSurface::SetValue"))
}

impl ShapeExtendCompositeSurface {
    /// `UJointValues()`: the handle of the U joint values (null before `Init`), shared:
    /// `SetUFirstValue` shifts them in place; `surface.u_joint_values()`.
    pub fn u_joint_values(&self) -> JointValues {
        self.fields.read().u_joints.clone()
    }

    /// `VJointValues()`: the handle of the V joint values; `surface.v_joint_values()`.
    pub fn v_joint_values(&self) -> JointValues {
        self.fields.read().v_joints.clone()
    }

    /// `UJointValue(i)`: the U joint of rank `i`; `surface.u_joint_value(1)?`.
    pub fn u_joint_value(&self, i: i32) -> Result<f64, Failure> {
        joint(&self.joints(true, "UJointValue")?, i, "UJointValue")
    }

    /// `VJointValue(j)`: the V joint of rank `j`; `surface.v_joint_value(1)?`.
    pub fn v_joint_value(&self, j: i32) -> Result<f64, Failure> {
        joint(&self.joints(false, "VJointValue")?, j, "VJointValue")
    }

    /// `SetUJointValues(UJoints)`: takes `NbUPatches() + 1` increasing values (each step
    /// at least `Precision::PConfusion()`); false and no change otherwise;
    /// `surface.set_u_joint_values(&joints)?`.
    pub fn set_u_joint_values(&self, joints: &Array1<'_, f64>) -> Result<bool, Failure> {
        let nb = self.nb_u_patches()?;
        let Some(values) = checked_joints(joints, nb, "SetUJointValues")? else {
            return Ok(false);
        };
        self.fields.write().u_joints = values;
        Ok(true)
    }

    /// `SetVJointValues(VJoints)`: as [`Self::set_u_joint_values`] for V;
    /// `surface.set_v_joint_values(&joints)?`.
    pub fn set_v_joint_values(&self, joints: &Array1<'_, f64>) -> Result<bool, Failure> {
        let nb = self.nb_v_patches()?;
        let Some(values) = checked_joints(joints, nb, "SetVJointValues")? else {
            return Ok(false);
        };
        self.fields.write().v_joints = values;
        Ok(true)
    }

    /// `SetUFirstValue(UFirst)`: shifts the U joints so that the first one is `u_first`;
    /// nothing before they exist; `surface.set_u_first_value(0.0)?`.
    pub fn set_u_first_value(&self, u_first: f64) -> Result<(), Failure> {
        shift_joints(&self.u_joint_values(), u_first)
    }

    /// `SetVFirstValue(VFirst)`: as [`Self::set_u_first_value`] for V;
    /// `surface.set_v_first_value(0.0)?`.
    pub fn set_v_first_value(&self, v_first: f64) -> Result<(), Failure> {
        shift_joints(&self.v_joint_values(), v_first)
    }
}

/// The body of `SetUJointValues`: a new `[1, nb + 1]` array of the values, or `None` when
/// the length differs or a step is below `Precision::PConfusion()`.
fn checked_joints(
    joints: &Array1<'_, f64>,
    nb: i32,
    member: &str,
) -> Result<Option<JointValues>, Failure> {
    if joints.length() != nb + 1 {
        return Ok(None);
    }
    let values = new_joints(nb + 1, member)?;
    let value = |j: i32| {
        joints
            .value(j)
            .copied()
            .map_err(|e| e.into_failure(&format!("ShapeExtend_CompositeSurface::{member}")))
    };
    for (i, j) in (1..=nb + 1).zip(joints.lower()..) {
        set_joint(&values, i, value(j)?)?;
        if i > 1 && value(j)? - value(j - 1)? < precision::p_confusion() {
            return Ok(None);
        }
    }
    Ok(Some(values))
}

/// The body of `SetUFirstValue`: adds `first - joints(1)` to every joint.
fn shift_joints(joints: &JointValues, first: f64) -> Result<(), Failure> {
    if joints.is_null() {
        return Ok(());
    }
    let shift = first - joint(joints, 1, "SetFirstValue")?;
    let nb = joints.array1().length();
    for i in 1..=nb {
        let value = joint(joints, i, "SetFirstValue")?;
        set_joint(joints, i, value + shift)?;
    }
    Ok(())
}

impl ShapeExtendCompositeSurface {
    /// `ComputeJointValues(param)`: new joint values: Natural adds the ranges of the patches
    /// of the first column (U) and of the first row (V), starting at their first bound;
    /// Uniform gives the steps 1 from 0, Unitary the steps `1 / NbU` and `1 / NbV`;
    /// `surface.compute_joint_values(ShapeExtendParametrisation::Unitary)?`.
    pub fn compute_joint_values(&self, param: ShapeExtendParametrisation) -> Result<(), Failure> {
        let (nb_u, nb_v) = (self.nb_u_patches()?, self.nb_v_patches()?);
        let u_joints = new_joints(nb_u + 1, "ComputeJointValues")?;
        let v_joints = new_joints(nb_v + 1, "ComputeJointValues")?;
        {
            let mut fields = self.fields.write();
            fields.u_joints = u_joints.clone();
            fields.v_joints = v_joints.clone();
        }
        if param == ShapeExtendParametrisation::Natural {
            self.natural_joints(&u_joints, nb_u, true)?;
            return self.natural_joints(&v_joints, nb_v, false);
        }
        let (mut step_u, mut step_v) = (1.0, 1.0);
        if param == ShapeExtendParametrisation::Unitary {
            step_u /= f64::from(nb_u);
            step_v /= f64::from(nb_v);
        }
        for i in 0..=nb_u {
            set_joint(&u_joints, i + 1, f64::from(i) * step_u)?;
        }
        for i in 0..=nb_v {
            set_joint(&v_joints, i + 1, f64::from(i) * step_v)?;
        }
        Ok(())
    }

    /// The natural joints of one direction, from the patches `(i, 1)` for U or `(1, i)` for V.
    fn natural_joints(
        &self,
        joints: &JointValues,
        nb: i32,
        u_direction: bool,
    ) -> Result<(), Failure> {
        let mut value = 0.0;
        for i in 1..=nb {
            let patch = if u_direction {
                self.live_patch(i, 1)?
            } else {
                self.live_patch(1, i)?
            };
            let (u1, u2, v1, v2) = patch.bounds();
            let (first, last) = if u_direction { (u1, u2) } else { (v1, v2) };
            if i == 1 {
                value = first;
                set_joint(joints, 1, value)?;
            }
            value += last - first;
            set_joint(joints, i + 1, value)?;
        }
        Ok(())
    }

    /// `CheckConnectivity(prec)`: samples 23 points along each joint (and along the joint
    /// of the last row or column with the first one, which only sets the closed flags) and
    /// returns whether the patches meet within `prec`; infinite bounds are limited to
    /// +/-10000; `surface.check_connectivity(1e-7)?`.
    pub fn check_connectivity(&self, prec: f64) -> Result<bool, Failure> {
        let (nb_u, nb_v) = (self.nb_u_patches()?, self.nb_v_patches()?);
        let mut ok = true;
        let mut j = nb_u;
        for i in 1..=nb_u {
            let gap = self.max_gap(
                |k| Ok((self.live_patch(j, k)?, self.live_patch(i, k)?)),
                nb_v,
                true,
            )?;
            ok &= self.record_gap(i, gap, prec, true);
            j = i;
        }
        let mut j = nb_v;
        for i in 1..=nb_v {
            let gap = self.max_gap(
                |k| Ok((self.live_patch(k, j)?, self.live_patch(k, i)?)),
                nb_u,
                false,
            )?;
            ok &= self.record_gap(i, gap, prec, false);
            j = i;
        }
        Ok(ok)
    }

    /// For the first joint, sets the closed flag of the direction; for the others,
    /// whether the gap is within `prec` (compared squared, as natively).
    fn record_gap(&self, i: i32, max_dist2: f64, prec: f64, u_direction: bool) -> bool {
        if i != 1 {
            // `else if (maxdist2 > Prec * Prec) ok = false;`: a NaN gap does not fail.
            let too_far = max_dist2 > prec * prec;
            return !too_far;
        }
        let within = max_dist2 <= prec * prec;
        let mut fields = self.fields.write();
        if u_direction {
            fields.u_closed = within;
        } else {
            fields.v_closed = within;
        }
        true
    }

    /// The largest squared distance over the `nb` pairs of patches `(before, after)` that
    /// `pair(k)` gives, between the last iso of `before` and the first iso of `after`.
    fn max_gap(
        &self,
        pair: impl Fn(i32) -> Result<(Handle<dyn GeomSurface>, Handle<dyn GeomSurface>), Failure>,
        nb: i32,
        u_direction: bool,
    ) -> Result<f64, Failure> {
        let mut max_dist2: f64 = 0.0;
        for k in 1..=nb {
            let (before, after) = pair(k)?;
            let (bj, bi) = (limited_bounds(&*before), limited_bounds(&*after));
            let (step_j, step_i) = if u_direction {
                (
                    (bj.3 - bj.2) / f64::from(NPOINTS - 1),
                    (bi.3 - bi.2) / f64::from(NPOINTS - 1),
                )
            } else {
                (
                    (bj.1 - bj.0) / f64::from(NPOINTS - 1),
                    (bi.1 - bi.0) / f64::from(NPOINTS - 1),
                )
            };
            for sample in 0..NPOINTS {
                let dist2 = sample_gap(
                    &*before,
                    &*after,
                    bj,
                    bi,
                    (step_j, step_i),
                    sample,
                    u_direction,
                )?;
                if max_dist2 < dist2 {
                    max_dist2 = dist2;
                }
            }
        }
        Ok(max_dist2)
    }
}

/// One sample of `CheckConnectivity`: along U, `sj(Uj2, Vj1 + stepj * k)` against
/// `si(Ui1, Vi1 + stepi * k)`; along V, `sj(Uj1 + stepj * k, Vj2)` against
/// `si(Ui1 + stepi * k, Vi1)`.
fn sample_gap(
    before: &dyn GeomSurface,
    after: &dyn GeomSurface,
    bj: (f64, f64, f64, f64),
    bi: (f64, f64, f64, f64),
    (step_j, step_i): (f64, f64),
    sample: i32,
    u_direction: bool,
) -> Result<f64, Failure> {
    let k = f64::from(sample);
    let (pj, pi) = if u_direction {
        (
            before.value(bj.1, bj.2 + step_j * k)?,
            after.value(bi.0, bi.2 + step_i * k)?,
        )
    } else {
        (
            before.value(bj.0 + step_j * k, bj.3)?,
            after.value(bi.0 + step_i * k, bi.2)?,
        )
    };
    Ok(pj.square_distance(&pi))
}

/// The static `LimitValue`: an infinite parameter becomes -10000 or 10000.
fn limit_value(par: f64) -> f64 {
    if precision::is_infinite(par) {
        return if par < 0.0 { -10000.0 } else { 10000.0 };
    }
    par
}

/// The static `GetLimitedBounds`: the bounds of the surface, each limited.
fn limited_bounds(surface: &dyn GeomSurface) -> (f64, f64, f64, f64) {
    let (u1, u2, v1, v2) = surface.bounds();
    (
        limit_value(u1),
        limit_value(u2),
        limit_value(v1),
        limit_value(v2),
    )
}
