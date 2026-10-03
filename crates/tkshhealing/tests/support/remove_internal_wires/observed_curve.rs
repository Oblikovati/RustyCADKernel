//! A named geometry observer verifies callback-visible progress without mocking geometry.
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkg3d::geom::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomGeometry};
use rustycad_tkmath::{
    geom_abs::Shape,
    gp::{Point3, Transform3, Vector3},
};
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeRemoveInternalWires;
use std::sync::{
    Arc, Weak,
    atomic::{AtomicUsize, Ordering},
};

pub struct RemovalObservingCurve {
    parent: StandardTransient,
    basis: Handle<dyn GeomCurve>,
    remover: Weak<ShapeUpgradeRemoveInternalWires>,
    observed: Arc<AtomicUsize>,
}
impl_transient!(RemovalObservingCurve, "RemovalObservingCurve", StandardTransient, parent, interfaces: [dyn GeomGeometry, dyn GeomCurve]);
impl RemovalObservingCurve {
    pub fn new(
        basis: Handle<dyn GeomCurve>,
        remover: &Arc<ShapeUpgradeRemoveInternalWires>,
        observed: &Arc<AtomicUsize>,
    ) -> Self {
        Self {
            parent: StandardTransient::default(),
            basis,
            remover: Arc::downgrade(remover),
            observed: observed.clone(),
        }
    }
    fn observe_progress(&self) {
        let remover = self.remover.upgrade().expect("live removal tool");
        assert_eq!(remover.min_area(), 2.0);
        let removed = remover.removed_wires().sequence(|seq| seq.len());
        self.observed.fetch_max(removed + 1, Ordering::SeqCst);
    }
}
impl GeomGeometry for RemovalObservingCurve {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        self.basis.transform(transform)
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        self.basis.copy()
    }
}
impl GeomCurve for RemovalObservingCurve {
    fn reverse(&self) -> Result<(), Failure> {
        self.basis.reverse()
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        self.basis.reversed_parameter(u)
    }
    fn first_parameter(&self) -> f64 {
        self.basis.first_parameter()
    }
    fn last_parameter(&self) -> f64 {
        self.basis.last_parameter()
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        self.basis.is_closed()
    }
    fn is_periodic(&self) -> bool {
        self.basis.is_periodic()
    }
    fn continuity(&self) -> Shape {
        self.basis.continuity()
    }
    fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        self.basis.is_cn(n)
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        self.observe_progress();
        self.basis.eval_d0(u)
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        self.basis.eval_d1(u)
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        self.basis.eval_d2(u)
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        self.basis.eval_d3(u)
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        self.basis.eval_dn(u, n)
    }
}
