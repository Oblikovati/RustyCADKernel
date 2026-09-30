// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert.cxx
// (ConcatG1, ConcatC1) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dConvert::ConcatG1` and `Geom2dConvert::ConcatC1`: the concatenation of an
//! array of B-splines into as few B-splines as the G1 junctions allow.
use super::comp_curve::CompCurveToBSplineCurve;
use super::concat_support::{
    JunctionTables, cloned_at, index_min, need_2deg_repara, need_to_be_treated, pretreatment,
    reorder_array_of_g1, store, zero_based,
};
use super::continuity::{Junction, continuity};
use super::reparametrize::{double_degree_reparametrization, mult_num_and_denom, new_curve_table};
use super::{HArrayOfBSplineCurves, as_curve, live_bspline};
use crate::hermit;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape;

/// `Geom2dConvert::ConcatG1(ArrayOfCurves, ArrayOfToler, ArrayOfConcatenated,
/// ClosedFlag, ClosedTolerance)`: joins each run of G1 junctions of
/// `array_of_curves` (indexed from 0) into one B-spline of `array_of_concatenated`.
///
/// `array_of_toler[i]` is the tolerance of the junction after curve `i`. The G1 test
/// itself uses Precision::Confusion() and Precision::Angular(). When `closed_flag`
/// is set, the array is rotated so that a group spans the closure, and a single
/// group becomes one periodic curve; `closed_flag` is cleared when the closing curve
/// cannot be reparametrized. The input array is rotated and the rational curves'
/// weights are normalized in place. Standard_ConstructionError when two curves do
/// not join, Standard_Failure ("Courbes non jointives") when their ends are apart;
/// `geom2d_convert::concat_g1(&mut curves, &tolerances, &mut concatenated, &mut closed, 1e-7)?`.
pub fn concat_g1(
    array_of_curves: &mut Array1<'_, Handle<Geom2dBSplineCurve>>,
    array_of_toler: &Array1<'_, f64>,
    array_of_concatenated: &mut HArrayOfBSplineCurves,
    closed_flag: &mut bool,
    closed_tolerance: f64,
) -> Result<(), Failure> {
    let tolerances = JunctionTolerances {
        fixed: None,
        angular: precision::angular(),
    };
    let mut plan: GroupPlan = GroupPlan::build(array_of_curves, array_of_toler, &tolerances)?;
    plan.rearrange(array_of_curves, *closed_flag, closed_tolerance)?;
    *array_of_concatenated = new_curve_table(plan.nb_group)?;
    let context = ConcatContext::prepare(array_of_curves, plan.nb_group, closed_flag)?;
    if plan.nb_group == 1 && *closed_flag {
        plan.reorder_at_min_degree(array_of_curves, closed_tolerance)?;
        let closed: Handle<Geom2dBSplineCurve> =
            close_g1(array_of_curves, &plan, context.need_double_deg_repara)?;
        return table_of(array_of_concatenated)?
            .change_array1()
            .set_value(0, closed)
            .map_err(table_error);
    }
    for_each_group(&plan, array_of_toler, |group, j| {
        let curve: Handle<Geom2dBSplineCurve> = cloned_at(array_of_curves, j)?;
        group.append(array_of_concatenated, curve)
    })
}

/// `Geom2dConvert::ConcatC1(ArrayOfCurves, ArrayOfToler, ArrayOfIndices,
/// ArrayOfConcatenated, ClosedFlag, ClosedTolerance)`: [`concat_c1_with_angular`]
/// with Precision::Angular();
/// `geom2d_convert::concat_c1(&mut curves, &tol, &mut indices, &mut result, &mut closed, 1e-7)?`.
pub fn concat_c1(
    array_of_curves: &mut Array1<'_, Handle<Geom2dBSplineCurve>>,
    array_of_toler: &Array1<'_, f64>,
    array_of_indices: &mut Handle<HArray1<i32>>,
    array_of_concatenated: &mut HArrayOfBSplineCurves,
    closed_flag: &mut bool,
    closed_tolerance: f64,
) -> Result<(), Failure> {
    let outputs = ConcatOutputs {
        indices: array_of_indices,
        concatenated: array_of_concatenated,
    };
    concat_c1_with_angular(
        array_of_curves,
        array_of_toler,
        outputs,
        closed_flag,
        closed_tolerance,
        precision::angular(),
    )
}

/// The two output arrays of ConcatC1: the index in the (rotated) input array of the
/// first curve of each group, then a closing index, and the concatenated curves.
pub struct ConcatOutputs<'o> {
    /// `ArrayOfIndices`, filled at `0..NbGroup`.
    pub indices: &'o mut Handle<HArray1<i32>>,
    /// `ArrayOfConcatenated`, filled at `0..NbGroup - 1`.
    pub concatenated: &'o mut HArrayOfBSplineCurves,
}

/// `Geom2dConvert::ConcatC1(..., ClosedTolerance, AngularTolerance)`: [`concat_g1`]
/// where the G1 test uses `array_of_toler` and `angular_tolerance`, and each rational
/// curve whose end weights are not 1 is first multiplied by its Hermit law so that
/// the junctions become C1;
/// `geom2d_convert::concat_c1_with_angular(&mut curves, &tol, outputs, &mut closed, 1e-7, 1e-9)?`.
pub fn concat_c1_with_angular(
    array_of_curves: &mut Array1<'_, Handle<Geom2dBSplineCurve>>,
    array_of_toler: &Array1<'_, f64>,
    outputs: ConcatOutputs<'_>,
    closed_flag: &mut bool,
    closed_tolerance: f64,
    angular_tolerance: f64,
) -> Result<(), Failure> {
    let tolerances = JunctionTolerances {
        fixed: Some(array_of_toler),
        angular: angular_tolerance,
    };
    let mut plan: GroupPlan = GroupPlan::build(array_of_curves, array_of_toler, &tolerances)?;
    plan.rearrange(array_of_curves, *closed_flag, closed_tolerance)?;
    *outputs.indices = Handle::new(HArray1::with_bounds(0, plan.nb_group).map_err(table_error)?);
    *outputs.concatenated = new_curve_table(plan.nb_group)?;
    let context = ConcatContext::prepare(array_of_curves, plan.nb_group, closed_flag)?;
    if plan.nb_group == 1 && *closed_flag {
        set_index(outputs.indices, 0, 0)?;
        set_index(outputs.indices, 1, 0)?;
        plan.reorder_at_min_degree(array_of_curves, closed_tolerance)?;
        let closed: Handle<Geom2dBSplineCurve> =
            close_c1(array_of_curves, &plan, context.need_double_deg_repara)?;
        return table_of(outputs.concatenated)?
            .change_array1()
            .set_value(0, closed)
            .map_err(table_error);
    }
    let concatenated: &mut HArrayOfBSplineCurves = outputs.concatenated;
    let indices: &mut Handle<HArray1<i32>> = outputs.indices;
    for_each_group(&plan, array_of_toler, |group, j| {
        if j == group.index {
            record_group_indices(indices, &plan, group, *closed_flag)?;
        }
        let curve: Handle<Geom2dBSplineCurve> = hermit_treated(&cloned_at(array_of_curves, j)?)?;
        group.append(concatenated, curve)
    })
}

// The tolerances of the junction test: ConcatG1 uses the default Continuity
// (Precision::Confusion(), Precision::Angular()), ConcatC1 the junction tolerances.
struct JunctionTolerances<'t> {
    fixed: Option<&'t Array1<'t, f64>>,
    angular: f64,
}

// The G1 flags of the junctions, their tolerances and the groups they make.
struct GroupPlan {
    nb_curve: i32,
    nb_group: i32,
    nb_vertex_group0: i32,
    tab_g1: Array1<'static, bool>,
    local_tolerance: Array1<'static, f64>,
}

impl GroupPlan {
    fn build(
        curves: &Array1<'_, Handle<Geom2dBSplineCurve>>,
        toler: &Array1<'_, f64>,
        tolerances: &JunctionTolerances<'_>,
    ) -> Result<Self, Failure> {
        let nb_curve: i32 = curves.length();
        let mut local_tolerance: Array1<'static, f64> = zero_based(toler.length())?;
        for i in 0..toler.length() {
            store(&mut local_tolerance, i, cloned_at(toler, i)?)?;
        }
        let tab_g1: Array1<'static, bool> = junction_flags(curves, tolerances)?;
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

    // Determination of the wire characteristics: the groups of G1 junctions.
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
    fn vertices_g1(&self, index: i32) -> Result<i32, Failure> {
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
    fn rearrange(
        &mut self,
        curves: &mut Array1<'_, Handle<Geom2dBSplineCurve>>,
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
    fn reorder_at_min_degree(
        &mut self,
        curves: &mut Array1<'_, Handle<Geom2dBSplineCurve>>,
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
        curves: &mut Array1<'_, Handle<Geom2dBSplineCurve>>,
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

// The G1 flag of each junction (tabG1, indexed from 0): true from G1 on. The native
// loop calls Continuity twice with the same arguments (a `< GeomAbs_C0` test that
// cannot hold, then the G1 test); the call has no side effect, so it runs once.
fn junction_flags(
    curves: &Array1<'_, Handle<Geom2dBSplineCurve>>,
    tolerances: &JunctionTolerances<'_>,
) -> Result<Array1<'static, bool>, Failure> {
    let nb_curve: i32 = curves.length();
    let mut tab_g1: Array1<'static, bool> = zero_based(nb_curve - 1)?;
    let mut pre_last: f64 = 0.0;
    for i in 0..nb_curve {
        let current: Handle<dyn Geom2dCurve> = as_curve(&cloned_at(curves, i)?);
        if i >= 1 {
            let previous: Handle<dyn Geom2dCurve> = as_curve(&cloned_at(curves, i - 1)?);
            let junction = Junction {
                c1: &previous,
                c2: &current,
                u1: pre_last,
                u2: live(&current)?.first_parameter(),
                r1: true,
                r2: true,
            };
            let (tl, ta): (f64, f64) = match tolerances.fixed {
                Some(toler) => (cloned_at(toler, i - 1)?, tolerances.angular),
                None => (precision::confusion(), tolerances.angular),
            };
            store(
                &mut tab_g1,
                i - 1,
                continuity(&junction, tl, ta)? >= Shape::G1,
            )?;
        }
        pre_last = live(&current)?.last_parameter();
    }
    Ok(tab_g1)
}

fn live(curve: &Handle<dyn Geom2dCurve>) -> Result<&dyn Geom2dCurve, Failure> {
    curve.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "Geom2dConvert: null curve in the array; expected a B-spline",
        )
    })
}

// What both members compute after the rearrangement: the weight normalization and
// the degree-doubling test of the closing curve (which clears `closed_flag` when
// the doubled degree would exceed Geom2d_BSplineCurve::MaxDegree()).
struct ConcatContext {
    need_double_deg_repara: bool,
}

impl ConcatContext {
    fn prepare(
        curves: &Array1<'_, Handle<Geom2dBSplineCurve>>,
        nb_group: i32,
        closed_flag: &mut bool,
    ) -> Result<Self, Failure> {
        pretreatment(curves)?;
        let need_double_deg_repara: bool = need_2deg_repara(curves)?;
        if nb_group == 1 && *closed_flag && need_double_deg_repara {
            let closing: Handle<Geom2dBSplineCurve> = cloned_at(curves, curves.length() - 1)?;
            if live_bspline(&closing)?.degree() > Geom2dBSplineCurve::max_degree() / 2 {
                *closed_flag = false;
            }
        }
        Ok(Self {
            need_double_deg_repara,
        })
    }
}

// The single closed group of ConcatG1: every curve appended to the first one, the
// closing curve reparametrized first when needed, then made periodic.
fn close_g1(
    curves: &Array1<'_, Handle<Geom2dBSplineCurve>>,
    plan: &GroupPlan,
    need_double_deg_repara: bool,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let mut curve2: Handle<Geom2dBSplineCurve> = cloned_at(curves, 0)?;
    for j in 1..=(plan.nb_curve - 1) {
        let mut curve1: Handle<Geom2dBSplineCurve> = cloned_at(curves, j)?;
        if j == plan.nb_curve - 1 && need_double_deg_repara {
            curve1 = double_degree_reparametrization(&curve1, &curve2, &cloned_at(curves, 0)?)?;
        }
        curve2 = merged(&curve2, &curve1, cloned_at(&plan.local_tolerance, j - 1)?)?;
    }
    make_periodic(&curve2)?;
    Ok(curve2)
}

// The single closed group of ConcatC1: as close_g1, each curve Hermit-treated first.
fn close_c1(
    curves: &Array1<'_, Handle<Geom2dBSplineCurve>>,
    plan: &GroupPlan,
    need_double_deg_repara: bool,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let mut curve2: Handle<Geom2dBSplineCurve> = Handle::default();
    for j in 0..=(plan.nb_curve - 1) {
        let mut curve1: Handle<Geom2dBSplineCurve> = hermit_treated(&cloned_at(curves, j)?)?;
        if j == 0 {
            curve2 = curve1;
            continue;
        }
        if j == plan.nb_curve - 1 && need_double_deg_repara {
            curve1 = double_degree_reparametrization(&curve1, &curve2, &cloned_at(curves, 0)?)?;
        }
        curve2 = merged(&curve2, &curve1, cloned_at(&plan.local_tolerance, j - 1)?)?;
    }
    make_periodic(&curve2)?;
    Ok(curve2)
}

// A single C1 curve: periodic, the closure knot reduced to multiplicity 1 as far as
// Precision::Confusion() allows.
fn make_periodic(curve: &Handle<Geom2dBSplineCurve>) -> Result<(), Failure> {
    let curve: &Geom2dBSplineCurve = live_bspline(curve)?;
    curve.set_periodic()?;
    let last: i32 = curve.last_u_knot_index()?;
    curve.remove_knot(last, curve.multiplicity(last)? - 1, precision::confusion())?;
    Ok(())
}

// CompCurveToBSplineCurve(first).Add(second, tolerance): Standard_ConstructionError
// when the curves do not join.
fn merged(
    first: &Handle<Geom2dBSplineCurve>,
    second: &Handle<Geom2dBSplineCurve>,
    tolerance: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let mut comp = CompCurveToBSplineCurve::with_curve(
        &first.cast::<dyn Geom2dBoundedCurve>(),
        ParameterisationType::TgtThetaOver2,
    )?;
    if !comp.add(&second.cast::<dyn Geom2dBoundedCurve>(), tolerance, false)? {
        return Err(failure(
            "Standard_ConstructionError",
            "Geom2dConvert Concatenation Error",
        ));
    }
    Ok(comp.bspline_curve())
}

// A rational curve whose end weights are not 1 is multiplied by its Hermit law.
fn hermit_treated(
    curve: &Handle<Geom2dBSplineCurve>,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    if need_to_be_treated(live_bspline(curve)?)? {
        return mult_num_and_denom(&hermit::solution_2d(curve, 1e-6, 1e-6)?, curve);
    }
    Ok(curve.clone())
}

// One step of the open-array loop: curve `current` of group `number`, which starts
// at curve `index`.
struct GroupStep<'t> {
    number: i32,
    index: i32,
    current: i32,
    toler: &'t Array1<'t, f64>,
}

impl GroupStep<'_> {
    // The first curve of a group fills its slot; the next ones are appended with
    // the input tolerance of their junction (not the rotated local copy), as natively.
    fn append(
        &self,
        table: &HArrayOfBSplineCurves,
        curve: Handle<Geom2dBSplineCurve>,
    ) -> Result<(), Failure> {
        let slot: &HArray1<Handle<Geom2dBSplineCurve>> = table_of(table)?;
        if self.current == self.index {
            return slot
                .change_array1()
                .set_value(self.number, curve)
                .map_err(table_error);
        }
        let first: Handle<Geom2dBSplineCurve> = slot
            .array1()
            .value(self.number)
            .map_err(table_error)?
            .clone();
        let joined: Handle<Geom2dBSplineCurve> =
            merged(&first, &curve, cloned_at(self.toler, self.current - 1)?)?;
        slot.change_array1()
            .set_value(self.number, joined)
            .map_err(table_error)
    }
}

// The main loop on each group and the secondary loop inside it.
fn for_each_group<'t>(
    plan: &GroupPlan,
    toler: &'t Array1<'t, f64>,
    mut visit: impl FnMut(&GroupStep<'t>, i32) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let mut index: i32 = 0;
    for number in 0..=(plan.nb_group - 1) {
        let nb_vertex_g1: i32 = plan.vertices_g1(index)?;
        for current in index..=(index + nb_vertex_g1) {
            let step = GroupStep {
                number,
                index,
                current,
                toler,
            };
            visit(&step, current)?;
        }
        index = index + 1 + nb_vertex_g1;
    }
    Ok(())
}

// Filling the array of preserved indices: the first curve of each group, then the
// closing index, shifted by the rotation of a closed array.
fn record_group_indices(
    indices: &Handle<HArray1<i32>>,
    plan: &GroupPlan,
    step: &GroupStep<'_>,
    closed_flag: bool,
) -> Result<(), Failure> {
    let k: i32 = step.number + 1;
    let (first, closing): (i32, i32) = if !closed_flag || plan.nb_group == 1 {
        (step.index, 0)
    } else {
        (
            step.index + plan.nb_vertex_group0 + 1,
            plan.nb_vertex_group0 + 1,
        )
    };
    set_index(indices, k - 1, first)?;
    if k == plan.nb_group {
        set_index(indices, k, closing)?;
    }
    Ok(())
}

fn set_index(indices: &Handle<HArray1<i32>>, index: i32, value: i32) -> Result<(), Failure> {
    table_of(indices)?
        .change_array1()
        .set_value(index, value)
        .map_err(table_error)
}

// The object of an output table handle (always allocated before it is written).
fn table_of<T: Send + Sync + 'static>(table: &Handle<HArray1<T>>) -> Result<&HArray1<T>, Failure> {
    table.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "Geom2dConvert: null output table; expected an array",
        )
    })
}

fn table_error(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    crate::native_failure::array_failure(error, "Geom2dConvert")
}
