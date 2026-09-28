//! `gp3d_lerp`: NCollection_Lerp over doubles and gp_XYZ, and gp_TrsfNLerp, mirroring
//! `gpc_lerp_case`.
use crate::elementary_probe::{Fields, attempt, end, real, xyz};
use crate::gp_transform_probe::{read_transform, trsf_state};
use rustycad_tkernel::ncollection::Lerp;
use rustycad_tkmath::gp::{TransformNLerp, Xyz};

pub fn gp3_lerp_case(values: &[f64]) {
    let mut f = Fields::new(values);
    let (s0, s1, t) = (f.real(), f.real(), f.real());
    let (x0, x1) = (f.xyz(), f.xyz());
    let (a, b) = (read_transform(&mut f), read_transform(&mut f));
    real(Lerp::interpolated(s0, s1, t));
    let mut scalars = Lerp::<f64>::default();
    real(scalars.interpolate(t));
    scalars.init(s0, s1);
    real(scalars.interpolate(t));
    real(Lerp::new(s1, s0).interpolate(t));
    xyz(Lerp::interpolated(x0, x1, t));
    xyz(Lerp::<Xyz>::new(x0, x1).interpolate(t));
    let interpolate = |lerp: &TransformNLerp| {
        attempt(|| {
            trsf_state(lerp.interpolate(t)?);
            Ok(())
        })
    };
    interpolate(&TransformNLerp::new(a, b));
    let mut empty = TransformNLerp::default();
    interpolate(&empty);
    empty.init(b, a);
    interpolate(&empty);
    end();
}
