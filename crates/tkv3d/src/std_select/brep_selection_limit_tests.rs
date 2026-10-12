//! Native-TU private FindLimits replay covers finite endpoints, NaN exits and the exact 100,000 expansion ceiling.
use super::brep_edge_sampling::brep_selection_curve_limits;
use rustycad_tkernel::{
    precision,
    standard::{Failure, SharedState, StandardTransient},
};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;
#[derive(Default)]
struct SelectionLimitEvaluations {
    count: usize,
    first: f64,
    last: f64,
}
struct SelectionLimitCurveFake {
    base: StandardTransient,
    scenario: usize,
    evaluations: SharedState<SelectionLimitEvaluations>,
}
rustycad_tkernel::impl_transient!(SelectionLimitCurveFake,"SelectionLimitCurveProbe",dyn Adaptor3dCurve,base,interfaces:[dyn Adaptor3dCurve]);
impl SelectionLimitCurveFake {
    fn new(scenario: usize) -> Self {
        Self {
            base: Default::default(),
            scenario,
            evaluations: Default::default(),
        }
    }
}
impl Adaptor3dCurve for SelectionLimitCurveFake {
    fn first_parameter(&self) -> Result<f64, Failure> {
        let finite = self.scenario == 0 || self.scenario == 3;
        Ok(if finite { 1. } else { -precision::infinite() })
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        let finite = self.scenario == 0 || self.scenario == 2;
        Ok(if finite { 4. } else { precision::infinite() })
    }
    fn eval_d0(&self, parameter: f64) -> Result<Point3, Failure> {
        let mut evaluations = self.evaluations.write();
        if evaluations.count == 0 {
            evaluations.first = parameter;
        }
        evaluations.count += 1;
        evaluations.last = parameter;
        let x = match self.scenario {
            4 => 0.,
            5 => f64::NAN,
            _ => parameter,
        };
        Ok(Point3::new(x, 0., 0.))
    }
}
#[test]
fn pinned_native_limit_expansion_preserves_nan_and_callback_order() {
    let rows = include_str!("../../tests/fixtures/colored-shape-compute/native-limits.txt");
    for row in rows.lines() {
        replay_selection_limit_row(row);
    }
}
fn replay_selection_limit_row(row: &str) {
    let fields: Vec<_> = row.split_whitespace().collect();
    let scenario = fields[1].parse::<usize>().unwrap();
    let curve = SelectionLimitCurveFake::new(scenario);
    let limit = if scenario == 6 { f64::NAN } else { 20. };
    let (first, last, limited) = brep_selection_curve_limits(&curve, limit).unwrap();
    assert_eq!(limited, fields[2] == "1");
    assert_eq!(first, fields[3].parse::<f64>().unwrap());
    assert_eq!(last, fields[4].parse::<f64>().unwrap());
    let evaluations = curve.evaluations.read();
    assert_eq!(evaluations.count, fields[5].parse::<usize>().unwrap());
    assert_eq!(evaluations.first, fields[6].parse::<f64>().unwrap());
    assert_eq!(evaluations.last, fields[7].parse::<f64>().unwrap());
}
