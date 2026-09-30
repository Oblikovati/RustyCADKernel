//! Fallible model loops on the native OSD thread pool.
use rustycad_tkernel::{
    osd::Parallel,
    standard::{Failure, SharedState},
};
pub(crate) fn model_indices(
    count: i32,
    parallel: bool,
    operation: impl Fn(i32) -> Result<(), Failure> + Sync,
) -> Result<(), Failure> {
    if !parallel || count == 1 {
        for index in 0..count {
            operation(index)?;
        }
        return Ok(());
    }
    let result = SharedState::new(Ok(()));
    Parallel::for_range(
        0,
        count,
        |index| {
            if let Err(error) = operation(index) {
                *result.write() = Err(error);
            }
        },
        false,
    )?;
    std::mem::replace(&mut *result.write(), Ok(()))
}
