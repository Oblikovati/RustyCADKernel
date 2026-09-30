//! Bounds selection and optional area metrics preserve native initialization order.
use super::*;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tktopalgo::{brep_gprop::BRepGProp, brep_lib::BRepLibMakeEdge};
impl SplitSurfaceData {
    fn reset(&self, surface: &Handle<dyn GeomSurface>) {
        self.with_mut(|fields| {
            fields.status = 0;
            fields.surface = surface.clone();
            fields.result = Handle::new(ShapeExtendCompositeSurface::new());
            fields.u_values = Handle::new(HSequence::default());
            fields.v_values = Handle::new(HSequence::default());
            fields.rows = 1;
            fields.columns = 1;
        });
    }
    pub(super) fn init_natural(&self, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
        self.reset(surface);
        let surface = surface.get().ok_or_else(null_surface)?;
        self.append_bounds(surface.bounds());
        Ok(())
    }
    pub(super) fn init_bounds(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
        area: f64,
    ) -> Result<(), Failure> {
        self.reset(surface);
        self.with_mut(|fields| fields.area = area);
        let surface_ref = surface.get().ok_or_else(null_surface)?;
        let bounds = clamp_bounds(surface_ref, bounds)?;
        if area != 0. {
            self.measure_isos(surface, bounds)?;
        }
        self.append_bounds(expand_bounds(bounds));
        Ok(())
    }
    fn append_bounds(&self, bounds: (f64, f64, f64, f64)) {
        let (u1, u2, v1, v2) = bounds;
        let u = self.values(true);
        let v = self.values(false);
        u.append(u1);
        u.append(u2);
        v.append(v1);
        v.append(v2);
    }
    fn measure_isos(
        &self,
        surface: &Handle<dyn GeomSurface>,
        bounds: (f64, f64, f64, f64),
    ) -> Result<(), Failure> {
        let trimmed = trimmed_surface(surface, bounds)?;
        let u_iso = trimmed.u_iso((bounds.0 + bounds.1) / 2.)?;
        let v_iso = trimmed.v_iso((bounds.2 + bounds.3) / 2.)?;
        let u_edge = BRepLibMakeEdge::from_curve(&u_iso)?.edge()?;
        let v_edge = BRepLibMakeEdge::from_curve(&v_iso)?.edge()?;
        let mut properties = GProps::default();
        BRepGProp::linear_properties(&v_edge, &mut properties, false, false)?;
        self.with_mut(|fields| fields.u_size = properties.mass());
        BRepGProp::linear_properties(&u_edge, &mut properties, false, false)?;
        self.with_mut(|fields| fields.v_size = properties.mass());
        Ok(())
    }
    pub(super) fn merge_values(&self, u: bool, given: &SurfaceSplitValues) -> Result<(), Failure> {
        if given.is_null() {
            return Ok(());
        }
        let own = self.values(u);
        let same = own.object_address() == given.object_address();
        let snapshot = if same {
            None
        } else {
            Some(split_snapshot(given)?)
        };
        let own = own.get().ok_or_else(null_surface)?;
        own.change_sequence(|values| {
            super::super::split_curve::insert_split_values(values, snapshot.as_deref())
        })
    }
}
fn null_surface() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_SplitSurface: null input; expected initialized surface and split values",
        "",
    )
}
fn clamp_bounds(
    surface: &dyn GeomSurface,
    bounds: (f64, f64, f64, f64),
) -> Result<(f64, f64, f64, f64), Failure> {
    let (mut u1, mut u2, mut v1, mut v2) = surface.bounds();
    if surface.is_u_periodic() && bounds.1 - bounds.0 <= u2 - u1 + precision::p_confusion() {
        u1 = bounds.0;
        u2 = u1 + surface.u_period()?;
    }
    if surface.is_v_periodic() && bounds.3 - bounds.2 <= v2 - v1 + precision::p_confusion() {
        v1 = bounds.2;
        v2 = v1 + surface.v_period()?;
    }
    let u = intersect_interval((bounds.0, bounds.1), (u1, u2));
    let v = intersect_interval((bounds.2, bounds.3), (v1, v2));
    Ok((u.0, u.1, v.0, v.1))
}
fn intersect_interval(given: (f64, f64), natural: (f64, f64)) -> (f64, f64) {
    if given.0 > natural.1 - precision::p_confusion()
        || given.1 < natural.0 - precision::p_confusion()
    {
        return natural;
    }
    (natural.0.max(given.0), natural.1.min(given.1))
}
fn expand_bounds(bounds: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let u = expand_interval((bounds.0, bounds.1));
    let v = expand_interval((bounds.2, bounds.3));
    (u.0, u.1, v.0, v.1)
}
fn expand_interval(mut bounds: (f64, f64)) -> (f64, f64) {
    if bounds.1 - bounds.0 < precision::p_confusion() {
        let half = precision::p_confusion() / 2.;
        bounds.0 -= half;
        bounds.1 += half;
    }
    bounds
}
