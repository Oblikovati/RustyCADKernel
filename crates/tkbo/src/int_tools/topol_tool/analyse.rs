use super::*;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkmath::gp::Vector3;
fn second_difference(a: Point3, b: Point3, c: Point3) -> Vector3 {
    Vector3::new(
        c.x() - b.x() - b.x() + a.x(),
        c.y() - b.y() - b.y() + a.y(),
        c.z() - b.z() - b.z() + a.z(),
    )
}
fn along(
    count: i32,
    sign: &mut i32,
    pole: impl Fn(i32) -> Result<Point3, Failure>,
) -> Result<i32, Failure> {
    let mut previous = second_difference(pole(1)?, pole(2)?, pole(3)?);
    let mut changes = 0;
    for k in 3..count {
        let next = second_difference(pole(k - 1)?, pole(k)?, pole(k + 1)?);
        let product = previous.dot(next);
        previous = next;
        changes += sign_change(product, sign);
    }
    Ok(changes)
}
fn sign_change(product: f64, sign: &mut i32) -> i32 {
    if !(product > 1e-7 || product < -1e-7) {
        return 0;
    }
    let wanted = if product > 0.0 { 1 } else { -1 };
    let changed = i32::from(*sign != wanted);
    *sign = wanted;
    changed
}
fn pole_at(poles: &Array2<'_, Point3>, i: i32, j: i32) -> Result<Point3, Failure> {
    poles
        .value(i, j)
        .copied()
        .map_err(|e| e.into_failure("IntTools_TopolTool::Analyse"))
}
pub(super) fn pole_counts(poles: &Array2<'_, Point3>) -> Result<[i32; 2], Failure> {
    let (nu, nv) = (poles.nb_rows() as i32, poles.nb_columns() as i32);
    let (mut sign, mut u, mut v) = (1, 0, 0);
    if nv > 2 {
        for i in 1..=nu {
            v = v.max(along(nv, &mut sign, |j| pole_at(poles, i, j))?);
        }
    }
    if nu > 2 {
        for j in 1..=nv {
            u = u.max(along(nu, &mut sign, |i| pole_at(poles, i, j))?);
        }
    }
    Ok([u + 5, v + 5])
}
