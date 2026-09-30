// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert.cxx
// (the junction groups of ConcatG1 and ConcatC1) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The junction groups of `GeomConvert::ConcatG1` and `GeomConvert::ConcatC1`:
//! the G1 flag and tolerance of each junction and the groups of G1 junctions.
use super::as_curve;
use super::concat_support::{JunctionTables, cloned_at, index_min, reorder_array_of_g1, stored};
use crate::array_access::zero_based;
use crate::geom_lprop;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkmath::geom_abs::Shape;

// The G1 flags of the junctions, their tolerances and the groups they make.
pub(super) struct GroupPlan {
    pub(super) nb_curve: i32,
    pub(super) nb_group: i32,
    pub(super) nb_vertex_group0: i32,
    tab_g1: Array1<'static, bool>,
    pub(super) local_tolerance: Array1<'static, f64>,
}

impl GroupPlan {
    // `angular` is None for ConcatG1 (the default Continuity tolerances), and the
    // angular tolerance of ConcatC1, whose junction tolerances are ArrayOfToler.
    pub(super) fn build(
        curves: &Array1<'_, Handle<GeomBSplineCurve>>,
        toler: &Array1<'_, f64>,
        angular: Option<f64>,
    ) -> Result<Self, Failure> {
        let nb_curve: i32 = curves.length();
        let mut local_tolerance: Array1<'static, f64> = zero_based(toler.length())?;
        for i in 0..toler.length() {
            stored(&mut local_tolerance, i, cloned_at(toler, i)?)?;
        }
        let tab_g1: Array1<'static, bool> = junction_flags(curves, &local_tolerance, angular)?;
        let mut plan = Self {
            nb_curve,
            nb_group: 0,
            nb_vertex_group0: 0,
            tab_g1,
            local_tolerance,
        };
        plan.count_groups()?;
        Ok(plan)
    }

    // Determination of the wire features: the groups of G1 junctions.
    fn count_groups(&mut self) -> Result<(), Failure> {
        let mut index: i32 = 0;
        while index < self.nb_curve {
            let nb_vertex_g1: i32 = self.vertices_g1(index)?;
            self.nb_group += 1;
            if index == 0 {
                self.nb_vertex_group0 = nb_vertex_g1;
            }
            index = index + 1 + nb_vertex_g1;
        }
        Ok(())
    }

    // The number of consecutive G1 junctions from `index`.
    pub(super) fn vertices_g1(&self, index: i32) -> Result<i32, Failure> {
        let mut nb_vertex_g1: i32 = 0;
        while (index + nb_vertex_g1) <= self.nb_curve - 2
            && cloned_at(&self.tab_g1, index + nb_vertex_g1)?
        {
            nb_vertex_g1 += 1;
        }
        Ok(nb_vertex_g1)
    }

    // A closed array of several groups is rotated so that the first group follows
    // the last one across the closure.
    pub(super) fn rearrange(
        &mut self,
        curves: &mut Array1<'_, Handle<GeomBSplineCurve>>,
        closed_flag: bool,
        closed_tolerance: f64,
    ) -> Result<(), Failure> {
        if closed_flag && self.nb_group != 1 {
            self.nb_group -= 1;
            let start: i32 = self.nb_vertex_group0;
            self.reorder(curves, start, closed_tolerance)?;
        }
        Ok(())
    }

    // A closed array of one group starts after its (last) lowest-degree curve.
    pub(super) fn reorder_at_min_degree(
        &mut self,
        curves: &mut Array1<'_, Handle<GeomBSplineCurve>>,
        closed_tolerance: f64,
    ) -> Result<(), Failure> {
        let indexmin: i32 = index_min(curves)?;
        if indexmin != curves.length() - 1 {
            self.reorder(curves, indexmin, closed_tolerance)?;
        }
        Ok(())
    }

    fn reorder(
        &mut self,
        curves: &mut Array1<'_, Handle<GeomBSplineCurve>>,
        start: i32,
        closed_tolerance: f64,
    ) -> Result<(), Failure> {
        let mut tables = JunctionTables {
            curves,
            tolerances: &mut self.local_tolerance,
            g1: &mut self.tab_g1,
        };
        reorder_array_of_g1(&mut tables, start, closed_tolerance)
    }
}

// The G1 flag of each junction (tabG1, indexed from 0): true from G1 on. A
// Continuity below C0 (which cannot occur) would be Standard_ConstructionError
// ("GeomConvert curves not C0").
fn junction_flags(
    curves: &Array1<'_, Handle<GeomBSplineCurve>>,
    local_tolerance: &Array1<'static, f64>,
    angular: Option<f64>,
) -> Result<Array1<'static, bool>, Failure> {
    let nb_curve: i32 = curves.length();
    let mut tab_g1: Array1<'static, bool> = zero_based(nb_curve - 1)?;
    let mut pre_last: f64 = 0.0;
    for i in 0..nb_curve {
        let current: Handle<dyn GeomCurve> = as_curve(&cloned_at(curves, i)?);
        if i >= 1 {
            let previous: Handle<dyn GeomCurve> = as_curve(&cloned_at(curves, i - 1)?);
            let first: f64 = live(&current)?.first_parameter();
            let cont: Shape = match angular {
                Some(ta) => geom_lprop::continuity_with_tolerances(
                    &previous,
                    &current,
                    pre_last,
                    first,
                    true,
                    true,
                    cloned_at(local_tolerance, i - 1)?,
                    ta,
                )?,
                None => geom_lprop::continuity(&previous, &current, pre_last, first, true, true)?,
            };
            stored(&mut tab_g1, i - 1, cont >= Shape::G1)?;
        }
        pre_last = live(&current)?.last_parameter();
    }
    Ok(tab_g1)
}

fn live(curve: &Handle<dyn GeomCurve>) -> Result<&dyn GeomCurve, Failure> {
    curve.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "GeomConvert: null curve in the array; expected a B-spline",
        )
    })
}
