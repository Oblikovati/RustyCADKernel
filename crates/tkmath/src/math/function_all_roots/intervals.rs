use super::{Failure, FunctionAllRoots, FunctionSampling, FunctionWithDerivative, RootState, Scan};
impl FunctionAllRoots {
    pub(super) fn scan(
        &mut self,
        f: &mut impl FunctionWithDerivative,
        s: &impl FunctionSampling,
        eps: [f64; 3],
    ) -> Result<Scan, Failure> {
        let mut value: f64 = 0.0;
        let mut scratch: f64 = 0.0;
        f.value(s.parameter(1), &mut value);
        let mut scan: Scan = Scan {
            previous_null: value.abs() <= eps[2],
            inside: false,
            starts_null: false,
            ends_null: false,
            saved: 0.0,
            start: RootState {
                parameter: 0.0,
                state: 0,
            },
        };
        if !scan.previous_null {
            scan.saved = value;
        }
        for index in 2..=s.number_of_points() {
            f.value(s.parameter(index), &mut value);
            self.scan_point(f, s, eps, &mut scan, index, value, &mut scratch)?;
        }
        if scan.inside {
            let end: RootState = Self::capture(f, s.parameter(s.number_of_points()), &mut scratch);
            self.intervals.push([scan.start, end]);
            scan.ends_null = true;
        }
        Ok(scan)
    }
    #[allow(clippy::too_many_arguments)]
    fn scan_point(
        &mut self,
        f: &mut impl FunctionWithDerivative,
        s: &impl FunctionSampling,
        eps: [f64; 3],
        scan: &mut Scan,
        index: i32,
        value: f64,
        scratch: &mut f64,
    ) -> Result<(), Failure> {
        let null: bool = value.abs() <= eps[2];
        if !null {
            scan.saved = value;
        }
        if scan.inside && !null {
            scan.inside = false;
            let end: RootState = Self::boundary(
                f,
                [s.parameter(index - 1), s.parameter(index)],
                eps,
                value,
                false,
            )?;
            self.intervals.push([scan.start, end]);
        } else if !scan.inside && scan.previous_null && null {
            scan.inside = true;
            Self::start_interval(f, s, eps, scan, index, scratch)?;
        }
        scan.previous_null = null;
        Ok(())
    }
    fn start_interval(
        f: &mut impl FunctionWithDerivative,
        s: &impl FunctionSampling,
        eps: [f64; 3],
        scan: &mut Scan,
        index: i32,
        scratch: &mut f64,
    ) -> Result<(), Failure> {
        if index == 2 {
            scan.start = Self::capture(f, s.parameter(1), scratch);
            scan.starts_null = true;
        } else {
            scan.start = Self::boundary(
                f,
                [s.parameter(index - 2), s.parameter(index - 1)],
                eps,
                scan.saved,
                true,
            )?;
        }
        Ok(())
    }
    fn boundary(
        f: &mut impl FunctionWithDerivative,
        bounds: [f64; 2],
        eps: [f64; 3],
        value: f64,
        last: bool,
    ) -> Result<RootState, Failure> {
        let target: f64 = if value > 0.0 { eps[2] } else { -eps[2] };
        let first: super::FunctionRoots = Self::roots(f, bounds, 10, eps, target, true)?;
        let index: i32 = if last {
            first.number_of_solutions()?
        } else {
            1
        };
        let mut chosen: RootState = Self::root_state(&first, index)?;
        let second: super::FunctionRoots = Self::roots(f, bounds, 10, eps, -target, false)?;
        let count: i32 = second.number_of_solutions()?;
        if count == 0 {
            return Ok(chosen);
        }
        let index: i32 = if last { count } else { 1 };
        let candidate: RootState = Self::root_state(&second, index)?;
        if (last && candidate.parameter > chosen.parameter)
            || (!last && candidate.parameter < chosen.parameter)
        {
            chosen = candidate;
        }
        Ok(chosen)
    }
}
