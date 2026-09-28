//! LU_Decompose and math_Gauss with a cancelling user progress indicator: the twin of
//! `tools/oracle/tkmath_progress_probe.cxx`.
use rustycad_tkernel::message::{ProgressIndicator, ProgressIndicatorHooks, ProgressScope};
use rustycad_tkmath::math::recipes::{
    lu_decompose_with_progress, lu_decompose_with_scales_and_progress,
};
use rustycad_tkmath::math::{Gauss, IntegerVector, Matrix, Vector};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicI64, Ordering},
};

/// Records every show; breaks once polled more than `allowed` times (never if negative).
struct CancellingIndicator {
    allowed: i64,
    polls: AtomicI64,
    shows: Mutex<Vec<String>>,
}
/// Hooks sharing the record with the probe after the indicator took ownership.
struct CancellingHooks(Arc<CancellingIndicator>);
impl ProgressIndicatorHooks for CancellingHooks {
    fn user_break(&self) -> bool {
        let polls = self.0.polls.fetch_add(1, Ordering::SeqCst) + 1;
        self.0.allowed >= 0 && polls > self.0.allowed
    }
    fn show(&self, scope: &ProgressScope<'_>, position: f64, force: bool) {
        let name = scope.name().map_or_else(
            || "~".to_owned(),
            |bytes| String::from_utf8_lossy(bytes).replace(' ', "_"),
        );
        let entry = format!("{} {name} {}", position.to_bits(), i32::from(force));
        self.0.shows.lock().unwrap().push(entry);
    }
}
fn cancelling(allowed: f64) -> (ProgressIndicator, Arc<CancellingIndicator>) {
    let record = Arc::new(CancellingIndicator {
        allowed: allowed as i64,
        polls: AtomicI64::new(0),
        shows: Mutex::new(Vec::new()),
    });
    let hooks = Box::new(CancellingHooks(Arc::clone(&record)));
    (ProgressIndicator::new(hooks), record)
}
fn report(record: &CancellingIndicator, indicator: &ProgressIndicator) {
    let shows = record.shows.lock().unwrap();
    print!("{} {} ", record.polls.load(Ordering::SeqCst), shows.len());
    for show in shows.iter() {
        print!("{show} ");
    }
    println!("{}", indicator.get_position().to_bits());
}
fn read_square(v: &[f64], n: i32, start: usize) -> Matrix<'static> {
    let mut matrix = Matrix::with_bounds(1, n, 1, n).unwrap();
    for (offset, value) in v[start..start + (n * n) as usize].iter().enumerate() {
        let (row, col) = (offset / n as usize, offset % n as usize);
        *matrix.change_at(row, col).unwrap() = *value;
    }
    matrix
}
fn cells(matrix: &Matrix<'_>) {
    let (lower_row, upper_row) = (matrix.lower_row(), matrix.upper_row());
    let (lower_col, upper_col) = (matrix.lower_col(), matrix.upper_col());
    print!("{lower_row} {upper_row} {lower_col} {upper_col} ");
    for row in lower_row..=upper_row {
        for col in lower_col..=upper_col {
            print!("{} ", matrix[(row, col)].to_bits());
        }
    }
}
fn integers(vector: &IntegerVector<'_>) {
    print!("{} {} {} ", vector.lower(), vector.upper(), vector.length());
    for index in vector.lower()..=vector.upper() {
        print!("{} ", vector[index]);
    }
}
/// `progress_lu n allowed_polls with_scales tiny cells[n*n]`.
pub(super) fn lu_case(v: &[f64]) {
    let (n, with_scales, tiny) = (v[0] as i32, v[2] != 0.0, v[3]);
    let mut a = read_square(v, n, 4);
    let mut indx = IntegerVector::filled(1, n, 0).unwrap();
    let mut d = 91.0;
    let (indicator, record) = cancelling(v[1]);
    let status = {
        let range = indicator.start();
        if with_scales {
            let mut vv = Vector::filled(1, n, 91.0).unwrap();
            lu_decompose_with_scales_and_progress(&mut a, &mut indx, &mut d, &mut vv, tiny, &range)
        } else {
            lu_decompose_with_progress(&mut a, &mut indx, &mut d, tiny, &range)
        }
    };
    print!("{} {} ", status.unwrap(), d.to_bits());
    cells(&a);
    integers(&indx);
    report(&record, &indicator);
}
/// `progress_gauss n allowed_polls min_pivot cells[n*n]`.
pub(super) fn gauss_case(v: &[f64]) {
    let n = v[0] as i32;
    let a = read_square(v, n, 3);
    let (indicator, record) = cancelling(v[1]);
    let gauss = Gauss::with_progress(&a, v[2], &indicator.start()).unwrap();
    print!("{} ", i32::from(gauss.is_done()));
    if gauss.is_done() {
        print!("{} ", gauss.determinant().unwrap().to_bits());
    }
    report(&record, &indicator);
}
