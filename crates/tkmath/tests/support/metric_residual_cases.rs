// SPDX-License-Identifier: GPL-2.0-only
//! Authored accepted metric requests are shared by functional and performance gates.
use super::metric_api::{RootIsolationOptions, metric_bounds::*};
pub struct AcceptedMetricWorkload {
    pub name: &'static str,
    pub curve: Vec<[f64; 3]>,
    pub pcurve: Vec<[f64; 2]>,
    pub weights: Option<Vec<f64>>,
    pub limit: f64,
    pub interval: [f64; 2],
    pub expected_within: bool,
    pub degree: u16,
}
impl AcceptedMetricWorkload {
    /// Construct the original complete trimmed request; example: `workload.query()`.
    pub fn query(&self) -> PlanarResidualQuery<'_> {
        PlanarResidualQuery {
            curve: fixture_curve(&self.curve, self.weights.as_deref()),
            curve_embedding: identity(),
            pcurve: fixture_curve(&self.pcurve, self.weights.as_deref()),
            plane_embedding: plane_frame(),
            plane_placement: identity(),
            interval: self.interval,
            spatial_limit: self.limit,
            options: RootIsolationOptions {
                max_degree: self.degree,
                ..RootIsolationOptions::new(0.001)
            },
        }
    }
    /// Verify the independently authored outcome; example: `workload.verify()`.
    pub fn verify(&self) {
        let result = certify_planar_residual(self.query(), || false).unwrap();
        assert_eq!(
            matches!(result, ResidualDecision::Within { .. }),
            self.expected_within
        );
        std::hint::black_box(result);
    }
}
fn identity() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
/// Own accepted request coefficients; example: `accepted_metric_workloads()[0].verify()`.
pub fn accepted_metric_workloads() -> Vec<AcceptedMetricWorkload> {
    vec![
        closed_cubic_case(),
        quadratic_case("interior-peak-below", 1.99, false),
        quadratic_case("interior-peak-at", 2., true),
        rational_case(),
        narrow_peak_case(),
        subnormal_case(),
    ]
}
fn closed_cubic_case() -> AcceptedMetricWorkload {
    let cubic = vec![[0., 0., 0.], [4., 0., 0.], [0., 4., 0.], [0., 0., 0.]];
    let planar = vec![[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    case("closed-cubic-zero", cubic, planar, 0., true)
}
fn quadratic_case(name: &'static str, limit: f64, within: bool) -> AcceptedMetricWorkload {
    case(
        name,
        vec![[0., 0., 0.], [0., 0., 4.], [0., 0., 0.]],
        vec![[0.; 2]; 2],
        limit,
        within,
    )
}
fn subnormal_case() -> AcceptedMetricWorkload {
    let height = f64::from_bits(1);
    case(
        "subnormal-constant",
        vec![[0., 0., height]; 2],
        vec![[0.; 2]; 2],
        height,
        true,
    )
}
fn case(
    name: &'static str,
    curve: Vec<[f64; 3]>,
    pcurve: Vec<[f64; 2]>,
    limit: f64,
    expected_within: bool,
) -> AcceptedMetricWorkload {
    AcceptedMetricWorkload {
        name,
        curve,
        pcurve,
        weights: None,
        limit,
        interval: [0., 1.],
        expected_within,
        degree: 32,
    }
}
fn rational_case() -> AcceptedMetricWorkload {
    let mut request = case(
        "nonunit-rational-offset",
        vec![[0., 0., 0.5], [1., 0., 0.5], [2., 0., 0.5]],
        vec![[0., 0.], [1., 0.], [2., 0.]],
        0.5,
        true,
    );
    request.weights = Some(vec![2., 7., 3.]);
    request
}
fn narrow_peak_case() -> AcceptedMetricWorkload {
    let mut curve = vec![[0.; 3]; 24];
    curve[1][2] = 1.;
    let mut request = case(
        "degree23-independent-peak",
        curve,
        vec![[0.; 2]; 2],
        0.3758881921123227,
        false,
    );
    request.degree = 64;
    request
}

fn fixture_curve<'a, const DIM: usize>(
    poles: &'a [[f64; DIM]],
    weights: Option<&'a [f64]>,
) -> MetricCurve<'a, DIM> {
    MetricCurve::Bezier(MetricBezier { poles, weights })
}
fn plane_frame() -> MetricEmbedding<2> {
    MetricEmbedding {
        linear: [[1., 0.], [0., 1.], [0., 0.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
