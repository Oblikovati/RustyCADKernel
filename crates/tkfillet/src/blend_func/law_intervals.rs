//! The intervals of the evolutive-radius functions: those of the guide (at the next
//! continuity) merged with those of the radius law (`FusionneIntervalles` of
//! BlendFunc_EvolRad and BRepBlend_SurfRstEvolRad / RstRstEvolRad).
use super::choice::live_law;
use super::package::{live_curve, next_shape};
use rustycad_tkernel::ncollection::{Array1, ArrayError};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::geom_abs::Shape;

/// The merged intervals, `None` when the law has a single interval.
fn merged_intervals(
    curv: &Handle<dyn Adaptor3dCurve>,
    fevol: &Handle<dyn LawFunction>,
    class: &str,
    shape: Shape,
) -> Result<Option<Vec<f64>>, Failure> {
    let curve = live_curve(curv)?;
    let nb_curve = curve.nb_intervals(next_shape(shape))?;
    let law = live_law(fevol, class)?;
    let nb_law = law.nb_intervals(shape)?;
    if nb_law == 1 {
        return Ok(None);
    }
    let mut int_c = Array1::with_bounds(1, nb_curve + 1).map_err(range)?;
    let mut int_l = Array1::with_bounds(1, nb_law + 1).map_err(range)?;
    curve.intervals(&mut int_c, next_shape(shape))?;
    law.intervals(&mut int_l, shape)?;
    fusionne_intervalles(&int_c, &int_l).map(Some)
}

/// `NbIntervals(S)` of an evolutive function: the merged count, or the guide count for
/// a single-interval law; `law_nb_intervals(&guide, &law, "BlendFunc_EvolRad", Shape::C1)?`.
pub(crate) fn law_nb_intervals(
    curv: &Handle<dyn Adaptor3dCurve>,
    fevol: &Handle<dyn LawFunction>,
    class: &str,
    shape: Shape,
) -> Result<i32, Failure> {
    match merged_intervals(curv, fevol, class, shape)? {
        Some(inter) => Ok(inter.len() as i32 - 1),
        None => live_curve(curv)?.nb_intervals(next_shape(shape)),
    }
}

/// `Intervals(T, S)` of an evolutive function.
pub(crate) fn law_intervals(
    curv: &Handle<dyn Adaptor3dCurve>,
    fevol: &Handle<dyn LawFunction>,
    class: &str,
    bounds: &mut Array1<'_, f64>,
    shape: Shape,
) -> Result<(), Failure> {
    let Some(inter) = merged_intervals(curv, fevol, class, shape)? else {
        return live_curve(curv)?.intervals(bounds, next_shape(shape));
    };
    for (ii, value) in (1..).zip(inter) {
        put(bounds, ii, value)?;
    }
    Ok(())
}

fn range(error: ArrayError) -> Failure {
    Failure::with_message("Standard_RangeError", &format!("{error:?}"))
}

/// `FusionneIntervalles(I1, I2, Seq)`: the sorted union of two sorted parameter tables,
/// values closer than `0.99 * PConfusion` merged into their middle.
fn fusionne_intervalles(i1: &Array1<'_, f64>, i2: &Array1<'_, f64>) -> Result<Vec<f64>, Failure> {
    let (mut ind1, mut ind2) = (1, 1);
    let epspar = precision::p_confusion() * 0.99;
    let mut seq = Vec::new();
    while ind1 <= i1.upper() && ind2 <= i2.upper() {
        let (v1, v2) = (at(i1, ind1)?, at(i2, ind2)?);
        if (v1 - v2).abs() <= epspar {
            seq.push((v1 + v2) / 2.);
            ind1 += 1;
            ind2 += 1;
        } else if v1 < v2 {
            seq.push(v1);
            ind1 += 1;
        } else {
            seq.push(v2);
            ind2 += 1;
        }
    }
    if ind1 > i1.upper() {
        for ind in ind2..=i2.upper() {
            seq.push(at(i2, ind)?);
        }
    }
    if ind2 > i2.upper() {
        for ind in ind1..=i1.upper() {
            seq.push(at(i1, ind)?);
        }
    }
    Ok(seq)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 1-based array of `values`, as the native intervals are.
    fn one_based(values: &[f64]) -> Array1<'static, f64> {
        let mut array = Array1::with_bounds(1, values.len() as i32).unwrap();
        for (index, value) in (1..).zip(values) {
            put(&mut array, index, *value).unwrap();
        }
        array
    }

    #[test]
    fn merged_intervals_fuse_close_values() {
        let (a, b) = (
            one_based(&[0., 1., 2.]),
            one_based(&[0., 1.0 + 1.0e-10, 1.5, 2.]),
        );
        let merged = fusionne_intervalles(&a, &b).unwrap();
        assert_eq!(merged, vec![0., (1. + 1.0 + 1.0e-10) / 2., 1.5, 2.]);
    }
}
