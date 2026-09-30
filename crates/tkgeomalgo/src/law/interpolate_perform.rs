// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_Interpolate.cxx (PerformPeriodic and
// PerformNonPeriodic) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The interpolation systems of `Law_Interpolate`.
use super::LawBSpline;
use super::interpolate::{LawInterpolate, build_periodic_tangent, build_tangents, shared};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkmath::bsplclib;

/// The arrays of one interpolation system (the native locals of `Perform*`).
struct System {
    parameters: Array1<'static, f64>,
    flatknots: Array1<'static, f64>,
    mults: Array1<'static, i32>,
    contact: Array1<'static, i32>,
    poles: Array1<'static, f64>,
}

impl System {
    /// Zero contact orders, unit interior multiplicities and `end_mult` at both ends.
    fn new(num_poles: i32, degree: i32, num_distinct: i32, end_mult: i32) -> Result<Self, Failure> {
        let mut mults: Array1<'static, i32> = one_based(num_distinct)?;
        for ii in 2..num_distinct {
            put(&mut mults, ii, 1)?;
        }
        put(&mut mults, 1, end_mult)?;
        put(&mut mults, num_distinct, end_mult)?;
        let mut contact: Array1<'static, i32> = one_based(num_poles)?;
        contact.init(0);
        Ok(Self {
            parameters: one_based(num_poles)?,
            flatknots: one_based(num_poles + degree + 1)?,
            mults,
            contact,
            poles: one_based(num_poles)?,
        })
    }

    /// `BSplCLib::Interpolate` in dimension 1 over `parameters`; returns the inversion
    /// problem flag.
    fn solve(&mut self, degree: i32, parameters: Option<&Array1<'_, f64>>) -> Result<i32, Failure> {
        let mut inversion_problem: i32 = 0;
        bsplclib::interpolate_flat(
            degree,
            &self.flatknots,
            parameters.unwrap_or(&self.parameters),
            &self.contact,
            1,
            self.poles.as_mut_slice(),
            &mut inversion_problem,
        )?;
        Ok(inversion_problem)
    }
}

/// The locked inputs of a `Perform`: values, parameters, tangents and flags.
struct Inputs {
    points: Array1<'static, f64>,
    params: Array1<'static, f64>,
    tangents: Array1<'static, f64>,
    flags: Array1<'static, bool>,
}

impl LawInterpolate {
    fn inputs(&self) -> Result<Inputs, Failure> {
        Ok(Inputs {
            points: shared(&self.points, "points")?.array1().clone(),
            params: shared(&self.parameters, "parameters")?.array1().clone(),
            tangents: shared(&self.tangents, "tangents")?.array1().clone(),
            flags: shared(&self.tangent_flags, "tangent flags")?
                .array1()
                .clone(),
        })
    }

    /// Writes back the tangents and flags that the tangent builders completed.
    fn store_tangents(&self, inputs: &Inputs) -> Result<(), Failure> {
        *shared(&self.tangents, "tangents")?.change_array1() = inputs.tangents.clone();
        *shared(&self.tangent_flags, "tangent flags")?.change_array1() = inputs.flags.clone();
        Ok(())
    }

    fn finish(&mut self, curve: LawBSpline) {
        self.curve = Handle::new(curve);
        self.is_done = true;
    }

    /// `PerformPeriodic`: a cubic with double knots at the period ends, the closing
    /// tangent from `BuildPeriodicTangent`.
    pub(super) fn perform_periodic(&mut self) -> Result<(), Failure> {
        let mut inputs: Inputs = self.inputs()?;
        let (degree, half_order): (i32, i32) = (3, 2);
        let num_points: i32 = inputs.points.len() as i32;
        let mut num_poles: i32 = num_points + 1 + 2;
        if self.tangent_request {
            num_poles += extra_tangents(&inputs.flags, inputs.flags.upper())?;
        }
        let mut sys = System::new(num_poles, degree, num_points + 1, half_order)?;
        periodic_end_knots(&mut sys, &inputs.params, num_poles, half_order)?;
        build_periodic_tangent(
            &inputs.points,
            &mut inputs.tangents,
            &mut inputs.flags,
            &inputs.params,
        )?;
        self.store_tangents(&inputs)?;
        start_conditions(&mut sys, &inputs)?;
        if self.tangent_request {
            fill_with_tangents(&mut sys, &inputs, inputs.flags.upper(), degree + 2)?;
        } else {
            periodic_fill(&mut sys, &inputs, degree)?;
        }
        let last_param: f64 = at(&inputs.params, inputs.params.upper())?;
        put(&mut sys.contact, num_poles - 1, 1)?;
        put(&mut sys.parameters, num_poles - 1, last_param)?;
        put(&mut sys.poles, num_poles - 1, at(&inputs.tangents, 1)?)?;
        put(&mut sys.parameters, num_poles, last_param)?;
        put(&mut sys.poles, num_poles, at(&inputs.points, 1)?)?;
        if sys.solve(degree, None)? != 0 {
            return Ok(());
        }
        let new_poles: Array1<'static, f64> =
            rustycad_tkg2d::geom2d::spline_data::slice(&sys.poles, 1, num_poles - 2)?;
        let curve = LawBSpline::new(
            &new_poles,
            &inputs.params,
            &sys.mults,
            degree,
            self.periodic,
        )?;
        self.finish(curve);
        Ok(())
    }

    /// `PerformNonPeriodic`: degree 1 for two values, 2 for three, else a cubic with
    /// the end tangents (built when missing) and the imposed interior tangents.
    pub(super) fn perform_non_periodic(&mut self) -> Result<(), Failure> {
        let mut inputs: Inputs = self.inputs()?;
        let num_points: i32 = inputs.points.len() as i32;
        let (degree, num_distinct, num_poles) =
            self.non_periodic_sizes(num_points, &inputs.flags)?;
        let mut sys = System::new(num_poles, degree, num_distinct, degree + 1)?;
        for ii in 1..=degree + 1 {
            put(&mut sys.flatknots, ii, at(&inputs.params, 1)?)?;
            put(
                &mut sys.flatknots,
                ii + num_poles,
                at(&inputs.params, num_points)?,
            )?;
        }
        match degree {
            1 => self.linear_law(&inputs, &mut sys, num_poles),
            2 => self.quadratic_law(&inputs, &mut sys, num_poles),
            _ => self.cubic_law(&mut inputs, &mut sys, num_points, num_poles),
        }
    }

    /// `(degree, distinct knots, poles)` of the non-periodic system.
    fn non_periodic_sizes(
        &self,
        num_points: i32,
        flags: &Array1<'_, bool>,
    ) -> Result<(i32, i32, i32), Failure> {
        if num_points == 2 && !self.tangent_request {
            return Ok((1, num_points, num_points));
        }
        if num_points == 3 && !self.tangent_request {
            return Ok((2, 2, num_points));
        }
        let mut num_poles: i32 = num_points + 2;
        if self.tangent_request {
            num_poles += extra_tangents(flags, flags.upper() - 1)?;
        }
        Ok((3, num_points, num_poles))
    }

    fn linear_law(
        &mut self,
        inputs: &Inputs,
        sys: &mut System,
        num_poles: i32,
    ) -> Result<(), Failure> {
        for ii in 1..=num_poles {
            put(&mut sys.poles, ii, at(&inputs.points, ii)?)?;
        }
        let curve = LawBSpline::new(&sys.poles, &inputs.params, &sys.mults, 1, false)?;
        self.finish(curve);
        Ok(())
    }

    fn quadratic_law(
        &mut self,
        inputs: &Inputs,
        sys: &mut System,
        num_poles: i32,
    ) -> Result<(), Failure> {
        let mut knots: Array1<'static, f64> = one_based(2)?;
        put(&mut knots, 1, at(&inputs.params, 1)?)?;
        put(&mut knots, 2, at(&inputs.params, num_poles)?)?;
        for ii in 1..=num_poles {
            put(&mut sys.poles, ii, at(&inputs.points, ii)?)?;
        }
        if sys.solve(2, Some(&inputs.params))? != 0 {
            return Ok(());
        }
        let curve = LawBSpline::new(&sys.poles, &knots, &sys.mults, 2, false)?;
        self.finish(curve);
        Ok(())
    }

    fn cubic_law(
        &mut self,
        inputs: &mut Inputs,
        sys: &mut System,
        num_points: i32,
        num_poles: i32,
    ) -> Result<(), Failure> {
        if num_points >= 3 {
            // Degree 3 cannot build the tangents from 2 points unless they were given.
            build_tangents(
                &inputs.points,
                &mut inputs.tangents,
                &mut inputs.flags,
                &inputs.params,
            )?;
            self.store_tangents(inputs)?;
        }
        start_conditions(sys, inputs)?;
        if self.tangent_request {
            fill_interior_tangents(sys, inputs, 3 + 2)?;
        } else {
            cubic_fill(sys, inputs)?;
        }
        let last_param: f64 = at(&inputs.params, inputs.params.upper())?;
        put(
            &mut sys.poles,
            num_poles - 1,
            at(&inputs.tangents, num_points)?,
        )?;
        put(&mut sys.contact, num_poles - 1, 1)?;
        put(&mut sys.parameters, num_poles, last_param)?;
        put(&mut sys.parameters, num_poles - 1, last_param)?;
        put(&mut sys.poles, num_poles, at(&inputs.points, num_points)?)?;
        if sys.solve(3, None)? != 0 {
            return Ok(());
        }
        let curve = LawBSpline::new(&sys.poles, &inputs.params, &sys.mults, 3, false)?;
        self.finish(curve);
        Ok(())
    }
}

/// The number of set flags from `Lower + 1` to `last`.
fn extra_tangents(flags: &Array1<'_, bool>, last: i32) -> Result<i32, Failure> {
    let mut extra: i32 = 0;
    for ii in flags.lower() + 1..=last {
        extra += i32::from(at(flags, ii)?);
    }
    Ok(extra)
}

/// The flat knots of the periodic system outside its interior knots.
fn periodic_end_knots(
    sys: &mut System,
    params: &Array1<'_, f64>,
    num_poles: i32,
    half: i32,
) -> Result<(), Failure> {
    let period: f64 = at(params, params.upper())? - at(params, params.lower())?;
    for ii in 1..=half {
        put(
            &mut sys.flatknots,
            ii,
            at(params, params.upper() - 1)? - period,
        )?;
        put(&mut sys.flatknots, ii + half, at(params, params.lower())?)?;
        put(
            &mut sys.flatknots,
            num_poles + ii,
            at(params, params.upper())?,
        )?;
        put(
            &mut sys.flatknots,
            num_poles + half + ii,
            at(params, half)? + period,
        )?;
    }
    Ok(())
}

/// The first value and the first tangent: poles 1 and 2 at the first parameter.
fn start_conditions(sys: &mut System, inputs: &Inputs) -> Result<(), Failure> {
    let first: f64 = at(&inputs.params, 1)?;
    put(&mut sys.contact, 2, 1)?;
    put(&mut sys.parameters, 1, first)?;
    put(&mut sys.parameters, 2, first)?;
    put(&mut sys.poles, 1, at(&inputs.points, 1)?)?;
    put(&mut sys.poles, 2, at(&inputs.tangents, 1)?)
}

/// The periodic loop with imposed tangents: each value, then its tangent when flagged,
/// from the flag after the first to `last`.
fn fill_with_tangents(
    sys: &mut System,
    inputs: &Inputs,
    last: i32,
    first_flat: i32,
) -> Result<(), Failure> {
    let (mut mult_index, mut index, mut index1): (i32, i32, i32) = (2, 3, first_flat);
    for ii in inputs.flags.lower() + 1..=last {
        let parameter: f64 = at(&inputs.params, ii)?;
        put(&mut sys.parameters, index, parameter)?;
        put(&mut sys.flatknots, index1, parameter)?;
        put(&mut sys.poles, index, at(&inputs.points, ii)?)?;
        index += 1;
        index1 += 1;
        if at(&inputs.flags, ii)? {
            let mult: i32 = at(&sys.mults, mult_index)?;
            put(&mut sys.mults, mult_index, mult + 1)?;
            put(&mut sys.contact, index, 1)?;
            put(&mut sys.parameters, index, parameter)?;
            put(&mut sys.flatknots, index1, parameter)?;
            put(&mut sys.poles, index, at(&inputs.tangents, ii)?)?;
            index += 1;
            index1 += 1;
        }
        mult_index += 1;
    }
    Ok(())
}

/// The non-periodic loop with imposed tangents over the interior parameters; the
/// value comes from the point counter `index2` and the flag from `index1`, as natively.
fn fill_interior_tangents(
    sys: &mut System,
    inputs: &Inputs,
    first_flat: i32,
) -> Result<(), Failure> {
    let (mut mult_index, mut index, mut index1) = (2, 3, 2);
    let (mut index2, mut index3) = (inputs.points.lower() + 1, first_flat);
    for ii in inputs.params.lower() + 1..inputs.params.upper() {
        let parameter: f64 = at(&inputs.params, ii)?;
        put(&mut sys.parameters, index, parameter)?;
        put(&mut sys.poles, index, at(&inputs.points, index2)?)?;
        put(&mut sys.flatknots, index3, parameter)?;
        index += 1;
        index3 += 1;
        if at(&inputs.flags, index1)? {
            let mult: i32 = at(&sys.mults, mult_index)?;
            put(&mut sys.mults, mult_index, mult + 1)?;
            put(&mut sys.contact, index, 1)?;
            put(&mut sys.flatknots, index3, parameter)?;
            put(&mut sys.parameters, index, parameter)?;
            put(&mut sys.poles, index, at(&inputs.tangents, ii)?)?;
            index += 1;
            index3 += 1;
        }
        mult_index += 1;
        index1 += 1;
        index2 += 1;
    }
    Ok(())
}

/// Copies `source(ii)` for `ii` in `from..=to` into consecutive slots of `target`
/// from `first_slot`.
fn copy_run(
    target: &mut Array1<'_, f64>,
    first_slot: i32,
    source: &Array1<'_, f64>,
    (from, to): (i32, i32),
) -> Result<(), Failure> {
    for (slot, ii) in (first_slot..).zip(from..=to) {
        put(target, slot, at(source, ii)?)?;
    }
    Ok(())
}

/// The periodic system without imposed tangents: every parameter and the values
/// after the first.
fn periodic_fill(sys: &mut System, inputs: &Inputs, degree: i32) -> Result<(), Failure> {
    let params: (i32, i32) = (inputs.params.lower(), inputs.params.upper());
    copy_run(&mut sys.parameters, 2, &inputs.params, params)?;
    copy_run(&mut sys.flatknots, degree + 1, &inputs.params, params)?;
    // Every given value is copied: the last pole is set from the first value later.
    let values: (i32, i32) = (inputs.points.lower() + 1, inputs.points.upper());
    copy_run(&mut sys.poles, 3, &inputs.points, values)
}

/// The cubic system without imposed tangents: every parameter, the interior values
/// and the interior flat knots.
fn cubic_fill(sys: &mut System, inputs: &Inputs) -> Result<(), Failure> {
    let params: (i32, i32) = (inputs.params.lower(), inputs.params.upper());
    copy_run(&mut sys.parameters, 2, &inputs.params, params)?;
    let values: (i32, i32) = (inputs.points.lower() + 1, inputs.points.upper() - 1);
    copy_run(&mut sys.poles, 3, &inputs.points, values)?;
    copy_run(&mut sys.flatknots, 3 + 1, &inputs.params, params)
}
