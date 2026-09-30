use super::*;
use rustycad_tkgeomalgo::geom_int::adjust_periodic;
struct PeriodicScan {
    uv: [f64; 2],
    adjusted: [f64; 2],
    period: [f64; 2],
    periodic: [bool; 2],
    reframed: [bool; 2],
}
impl PeriodicScan {
    fn new(
        surface: &BRepAdaptorSurface,
        p: Point2,
        b: [f64; 4],
        recadre: bool,
    ) -> Result<Self, Failure> {
        let periodic = [surface.is_u_periodic()?, surface.is_v_periodic()?];
        let period = [
            if periodic[0] { surface.u_period()? } else { 0. },
            if periodic[1] { surface.v_period()? } else { 0. },
        ];
        let uv = [p.x(), p.y()];
        Ok(Self {
            uv,
            adjusted: adjusted_point(uv, b, period, periodic, recadre),
            period,
            periodic,
            reframed: [false; 2],
        })
    }
    fn finished(&self, state: State, recadre: bool) -> bool {
        !recadre
            || (!self.periodic[0] && !self.periodic[1])
            || matches!(state, State::In | State::On)
    }
    fn next(&mut self, b: [f64; 4]) -> bool {
        self.advance_axis(0);
        if self.uv[0] > b[1] || !self.periodic[0] {
            self.advance_axis(1);
            self.uv[0] = self.adjusted[0];
            if self.uv[1] > b[3] || !self.periodic[1] {
                return false;
            }
        }
        true
    }
    fn advance_axis(&mut self, axis: usize) {
        if !self.reframed[axis] {
            self.uv[axis] = self.adjusted[axis];
            self.reframed[axis] = true;
        } else if self.periodic[axis] {
            self.uv[axis] += self.period[axis];
        }
    }
}
impl FClass2d {
    pub(super) fn classify(
        &self,
        p: Point2,
        recadre: bool,
        on: Option<f64>,
    ) -> Result<State, Failure> {
        if self.classifiers.is_empty() {
            return Ok(State::In);
        }
        let surface = BRepAdaptorSurface::with_face(&self.face, false)?;
        let scan = PeriodicScan::new(&surface, p, self.bounds, recadre)?;
        self.scan_copies(&surface, scan, on, recadre)
    }
    fn scan_copies(
        &self,
        surface: &BRepAdaptorSurface,
        mut scan: PeriodicScan,
        on: Option<f64>,
        recadre: bool,
    ) -> Result<State, Failure> {
        loop {
            let p = Point2::new(scan.uv[0], scan.uv[1]);
            let state = self.state_at(p, surface, on)?;
            if scan.finished(state, recadre) {
                return Ok(state);
            }
            if !scan.next(self.bounds) {
                return Ok(state);
            }
        }
    }
    fn state_at(
        &self,
        p: Point2,
        s: &BRepAdaptorSurface,
        on: Option<f64>,
    ) -> Result<State, Failure> {
        if self.orientations[0] != -1 {
            let state = self.polygon_state(p, on);
            if state != State::On || on.is_some() {
                return Ok(state);
            }
        }
        let tolerance = match on {
            Some(tolerance) => tolerance,
            None => self.exact_tolerance(p, s)?,
        };
        self.exact_state(p, tolerance)
    }
    fn exact_state(&self, p: Point2, tolerance: f64) -> Result<State, Failure> {
        let mut guard = self.explorer.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_none() {
            *guard = Some(BRepClassFaceExplorer::new(&self.face)?);
        }
        let classifier =
            BRepClassFaceClassifier::with_explorer(guard.as_mut().unwrap(), &p, tolerance)?;
        Ok(classifier.state())
    }
    fn polygon_state(&self, p: Point2, on: Option<f64>) -> State {
        for (classifier, orientation) in self.classifiers.iter().zip(&self.orientations) {
            let state = match on {
                Some(tol) => classifier.si_dans_on_mode(p, tol),
                None => classifier.si_dans(p),
            };
            if state == Class2dResult::Uncertain {
                return State::On;
            }
            if (state == Class2dResult::Inside && *orientation == 0)
                || (state == Class2dResult::Outside && *orientation == 1)
            {
                return State::Out;
            }
        }
        State::In
    }
    fn exact_tolerance(&self, p: Point2, s: &BRepAdaptorSurface) -> Result<f64, Failure> {
        let u = s.u_resolution(self.tolerance)?;
        let v = s.v_resolution(self.tolerance)?;
        let ui = p.x() >= self.bounds[0] && p.x() <= self.bounds[1];
        let vi = p.y() >= self.bounds[2] && p.y() <= self.bounds[3];
        Ok(if ui == vi {
            std_min(u, v)
        } else if !ui {
            u
        } else {
            v
        })
    }
}

fn adjusted_point(
    uv: [f64; 2],
    b: [f64; 4],
    period: [f64; 2],
    periodic: [bool; 2],
    recadre: bool,
) -> [f64; 2] {
    let mut adjusted = uv;
    for axis in 0..2 {
        if recadre && periodic[axis] {
            adjusted[axis] =
                adjust_periodic(uv[axis], b[axis * 2], b[axis * 2 + 1], period[axis], 0.).1;
        }
    }
    adjusted
}
