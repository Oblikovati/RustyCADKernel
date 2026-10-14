//! Shared accepted-frame collection preserves each workload's warmup/index reset policy.
#![allow(dead_code)]
/// Discard warmup observations; e.g. collect_samples(20,100,|index| redraw(index)).
pub fn collect_samples(
    warmup: usize,
    samples: usize,
    mut frame: impl FnMut(usize) -> f64,
) -> Vec<f64> {
    for index in 0..warmup {
        frame(index);
    }
    (0..samples).map(frame).collect()
}
#[cfg(test)]
mod tests {
    #[test]
    fn warmup_is_discarded_and_measured_index_restarts() {
        let mut indices = Vec::new();
        let mut calls = 0;
        let samples = super::collect_samples(3, 2, |index| {
            indices.push(index);
            calls += 1;
            calls as f64
        });
        assert_eq!(indices, [0, 1, 2, 0, 1]);
        assert_eq!(samples, [4., 5.]);
    }
}
