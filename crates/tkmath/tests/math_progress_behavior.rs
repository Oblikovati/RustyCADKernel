//! Progress and user-break behavior of LU_Decompose and math_Gauss: the
//! `Message_ProgressRange` argument drives a `"math_Gauss LU_Decompose"` scope with
//! one step per column and aborts with `math_Status_UserAborted` on a break.
use rustycad_tkernel::message::{ProgressIndicator, ProgressIndicatorHooks, ProgressScope};
use rustycad_tkmath::math::recipes::{
    STATUS_OK, STATUS_SINGULAR_MATRIX, STATUS_USER_ABORTED, lu_decompose,
    lu_decompose_with_progress, lu_decompose_with_scales_and_progress,
};
use rustycad_tkmath::math::{Gauss, IntegerVector, Matrix, Vector};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

/// Fake user indicator: records `show` positions and scope names, and reports a
/// break once it has been polled more than `allowed_polls` times.
struct CancellingIndicator {
    allowed_polls: usize,
    polls: AtomicUsize,
    shows: Mutex<Vec<(f64, Option<Vec<u8>>)>>,
}
/// Hooks sharing the fake so a test can read it after the indicator borrowed it.
struct CancellingHooks(Arc<CancellingIndicator>);
impl ProgressIndicatorHooks for CancellingHooks {
    fn show(&self, scope: &ProgressScope<'_>, position: f64, _force: bool) {
        let name = scope.name().map(<[u8]>::to_vec);
        self.0.shows.lock().unwrap().push((position, name));
    }
    fn user_break(&self) -> bool {
        self.0.polls.fetch_add(1, Ordering::SeqCst) >= self.0.allowed_polls
    }
}
fn cancelling(allowed_polls: usize) -> (ProgressIndicator, Arc<CancellingIndicator>) {
    let fake = Arc::new(CancellingIndicator {
        allowed_polls,
        polls: AtomicUsize::new(0),
        shows: Mutex::new(Vec::new()),
    });
    let hooks = Box::new(CancellingHooks(Arc::clone(&fake)));
    (ProgressIndicator::new(hooks), fake)
}
fn diagonal(n: i32) -> Matrix<'static> {
    let mut result = Matrix::with_bounds(1, n, 1, n).unwrap();
    for i in 1..=n {
        for j in 1..=n {
            result[(i, j)] = if i == j { 4.0 } else { 1.0 / f64::from(i + j) };
        }
    }
    result
}
fn show_positions(fake: &CancellingIndicator) -> Vec<f64> {
    fake.shows
        .lock()
        .unwrap()
        .iter()
        .map(|show| show.0)
        .collect()
}

#[test]
fn lu_without_break_advances_once_per_column() {
    let (progress, fake) = cancelling(usize::MAX);
    let mut a = diagonal(4);
    let mut indx = IntegerVector::with_bounds(1, 4).unwrap();
    let mut d = 0.0;
    let status = lu_decompose_with_progress(&mut a, &mut indx, &mut d, 1e-20, &progress.start());
    assert_eq!(status.unwrap(), STATUS_OK);
    assert_eq!(show_positions(&fake), [0.0, 0.25, 0.5, 0.75, 1.0]);
    assert_eq!(fake.polls.load(Ordering::SeqCst), 4);
    let name = fake.shows.lock().unwrap()[1].1.clone();
    assert_eq!(name.as_deref(), Some(&b"math_Gauss LU_Decompose"[..]));
    let mut reference = diagonal(4);
    lu_decompose(&mut reference, &mut indx, &mut d, 1e-20).unwrap();
    assert_eq!(reference[(4, 4)].to_bits(), a[(4, 4)].to_bits());
}
#[test]
fn lu_break_before_a_column_aborts_and_closes_the_scope() {
    let (progress, fake) = cancelling(2);
    let mut a = diagonal(5);
    let mut indx = IntegerVector::with_bounds(1, 5).unwrap();
    let mut scales = Vector::with_bounds(1, 5).unwrap();
    let mut d = 0.0;
    let range = progress.start();
    let status = lu_decompose_with_scales_and_progress(
        &mut a,
        &mut indx,
        &mut d,
        &mut scales,
        1e-30,
        &range,
    );
    assert_eq!(status.unwrap(), STATUS_USER_ABORTED);
    assert_eq!(fake.polls.load(Ordering::SeqCst), 3);
    assert_eq!(show_positions(&fake), [0.0, 0.2, 0.4, 1.0]);
    assert!(!range.is_active());
}
#[test]
fn lu_singular_exit_still_completes_the_range() {
    let (progress, fake) = cancelling(usize::MAX);
    let mut a = diagonal(3);
    for j in 1..=3 {
        a[(2, j)] = 0.0;
    }
    let mut indx = IntegerVector::with_bounds(1, 3).unwrap();
    let mut d = 0.0;
    let status = lu_decompose_with_progress(&mut a, &mut indx, &mut d, 1e-20, &progress.start());
    assert_eq!(status.unwrap(), STATUS_SINGULAR_MATRIX);
    assert_eq!(show_positions(&fake), [0.0, 1.0]);
    assert_eq!(fake.polls.load(Ordering::SeqCst), 0);
}
#[test]
fn gauss_with_a_cancelling_indicator_is_not_done() {
    let (progress, _fake) = cancelling(0);
    let solver = Gauss::with_progress(&diagonal(3), 1e-20, &progress.start()).unwrap();
    assert!(!solver.is_done());
    assert_eq!(
        solver.determinant().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    let (progress, fake) = cancelling(usize::MAX);
    let solver = Gauss::with_progress(&diagonal(3), 1e-20, &progress.start()).unwrap();
    assert!(solver.is_done());
    assert_eq!(show_positions(&fake).len(), 4);
}
#[test]
fn gauss_refuses_a_non_square_matrix_before_consuming_the_range() {
    let (progress, fake) = cancelling(usize::MAX);
    let range = progress.start();
    let rectangular = Matrix::with_bounds(1, 2, 1, 3).unwrap();
    let error = Gauss::with_progress(&rectangular, 1e-20, &range)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "math_NotSquare");
    assert!(range.is_active());
    drop(range);
    assert_eq!(show_positions(&fake), [0.0, 1.0]);
}
