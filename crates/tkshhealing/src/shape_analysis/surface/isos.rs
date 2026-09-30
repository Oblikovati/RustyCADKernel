use super::*;

impl ShapeAnalysisSurface {
    /// Computes all four boundary isos once; `analysis.compute_bound_isos();`.
    pub fn compute_bound_isos(&self) {
        if self.fields.read().bound_isos.is_some() {
            return;
        }
        let (uf, ul, vf, vl) = self.bounds();
        let surface = self.surface();
        let isos = [
            protected_iso(&surface, true, uf),
            protected_iso(&surface, true, ul),
            protected_iso(&surface, false, vf),
            protected_iso(&surface, false, vl),
        ];
        self.fields.write().bound_isos = Some(isos);
    }

    /// Returns a U iso, reusing cached boundaries; `let iso = analysis.u_iso(0.0);`.
    pub fn u_iso(&self, parameter: f64) -> Handle<dyn GeomCurve> {
        let (first, last, _, _) = self.bounds();
        self.cached_iso(true, parameter, first, last, 0)
    }

    /// Returns a V iso, reusing cached boundaries; `let iso = analysis.v_iso(0.0);`.
    pub fn v_iso(&self, parameter: f64) -> Handle<dyn GeomCurve> {
        let (_, _, first, last) = self.bounds();
        self.cached_iso(false, parameter, first, last, 2)
    }

    fn cached_iso(
        &self,
        u_iso: bool,
        parameter: f64,
        first: f64,
        last: f64,
        offset: usize,
    ) -> Handle<dyn GeomCurve> {
        if parameter != first && parameter != last {
            return protected_iso(&self.surface(), u_iso, parameter);
        }
        self.compute_bound_isos();
        let index = offset + usize::from(parameter != first);
        self.fields.read().bound_isos.as_ref().unwrap()[index].clone()
    }
}

fn protected_iso(
    surface: &Handle<dyn GeomSurface>,
    u_iso: bool,
    parameter: f64,
) -> Handle<dyn GeomCurve> {
    let Some(surface) = surface.get() else {
        return Handle::default();
    };
    let result = if u_iso {
        surface.u_iso(parameter)
    } else {
        surface.v_iso(parameter)
    };
    result.unwrap_or_default()
}
