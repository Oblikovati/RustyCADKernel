//! Named real curve implementation injects one defined topology mutation from its native EvalD0 callback.
use rustycad_tkbrep::{
    topexp,
    topods::{self, Builder, Shape, TShape},
};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkg3d::geom::*;
use rustycad_tkmath::{
    geom_abs::Shape as Continuity,
    gp::{Point3, Transform3, Vector3},
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
pub(super) struct BrepContourMutationCurveFake {
    transient: StandardTransient,
    edge: SharedState<WeakHandle<dyn TShape>>,
    pub mutate: AtomicBool,
    pub calls: AtomicUsize,
    pub slot_callback:
        SharedState<Option<Box<dyn super::brep_owner_slot_curve_callback::BrepCurveSlotMutation>>>,
}
rustycad_tkernel::impl_transient!(BrepContourMutationCurveFake,"Martensite_ContourMutationCurve",dyn GeomCurve,transient,interfaces:[dyn GeomGeometry,dyn GeomCurve]);
impl BrepContourMutationCurveFake {
    pub fn new() -> Handle<Self> {
        let edge = SharedState::new(WeakHandle::default());
        Handle::new(Self {
            transient: Default::default(),
            edge,
            mutate: AtomicBool::new(false),
            calls: AtomicUsize::new(0),
            slot_callback: SharedState::new(None),
        })
    }
    /// Build the native finite linear-parameter curve fixture; `BrepContourMutationCurveFake::edge_fixture(&curve)`.
    pub fn edge_fixture(curve: &Handle<Self>) -> rustycad_tkbrep::topods::Edge {
        let builder =
            rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge::from_curve_params(
                &curve.cast(),
                0.,
                1.,
            )
            .unwrap();
        builder.edge().unwrap()
    }
    pub fn activate(&self, edge: &Shape) {
        *self.edge.write() = edge.tshape().downgrade();
        self.calls.store(0, Ordering::Relaxed);
        self.mutate.store(true, Ordering::Relaxed);
    }
    fn remove_current_edge_vertex(&self) -> Result<(), Failure> {
        let topology = self.edge.read().observe();
        Shape::with_reference_parts(
            &*topology,
            &Default::default(),
            rustycad_tkg3d::top_abs::Orientation::Forward,
            |shape| {
                let mut edge = topods::edge(shape)?;
                edge.set_free(true)?;
                let vertex = topexp::first_vertex(&edge, false);
                Builder.remove(&mut edge, &vertex)
            },
        )
    }
}
impl GeomGeometry for BrepContourMutationCurveFake {
    fn transform(&self, _: Transform3) -> Result<(), Failure> {
        // The native named fixture uses the identity placement throughout this scenario.
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        let curve = Self::new();
        Ok(curve.into_cast())
    }
}
impl GeomCurve for BrepContourMutationCurveFake {
    fn reverse(&self) -> Result<(), Failure> {
        // The callback trace never changes the native curve parameterization.
        Ok(())
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        let parameter = 1. - u;
        Ok(parameter)
    }
    fn first_parameter(&self) -> f64 {
        // Native fixture curve starts at zero.
        0.
    }
    fn last_parameter(&self) -> f64 {
        // Native fixture curve ends at one.
        1.
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let closed = false;
        Ok(closed)
    }
    fn is_periodic(&self) -> bool {
        // The callback fixture has no periodic wrap.
        false
    }
    fn continuity(&self) -> Continuity {
        // The analytic fixture is smooth across the sampled interval.
        Continuity::CN
    }
    fn is_cn(&self, _: i32) -> Result<bool, Failure> {
        let continuous = true;
        Ok(continuous)
    }
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        let callback = self.slot_callback.write().take();
        if let Some(callback) = callback {
            callback.replace();
        }
        if self.mutate.swap(false, Ordering::Relaxed) {
            self.remove_current_edge_vertex()?;
        }
        Ok(Point3::new(u, 0., 0.))
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let point = self.eval_d0(u)?;
        Ok(GeomCurveD1 {
            point,
            d1: Vector3::new(1., 0., 0.),
        })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let first = self.eval_d1(u)?;
        Ok(GeomCurveD2 {
            point: first.point,
            d1: first.d1,
            d2: Vector3::default(),
        })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let first = self.eval_d1(u)?;
        Ok(GeomCurveD3 {
            point: first.point,
            d1: first.d1,
            d2: Vector3::default(),
            d3: Vector3::default(),
        })
    }
    fn eval_dn(&self, _: f64, n: i32) -> Result<Vector3, Failure> {
        let derivative = if n == 1 {
            Vector3::new(1., 0., 0.)
        } else {
            Vector3::default()
        };
        Ok(derivative)
    }
}
