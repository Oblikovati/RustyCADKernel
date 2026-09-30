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
        return sequential_items(0..count, operation);
    }
    let result = SharedState::new(Ok(()));
    Parallel::for_range(
        0,
        count,
        |index| record_failure(&result, operation(index)),
        false,
    )?;
    std::mem::replace(&mut *result.write(), Ok(()))
}

pub(crate) fn model_items<T: Send>(
    items: Vec<T>,
    parallel: bool,
    operation: impl Fn(T) -> Result<(), Failure> + Sync,
) -> Result<(), Failure> {
    if !parallel || items.len() <= 1 {
        return sequential_items(items, operation);
    }
    let result = SharedState::new(Ok(()));
    let count = items.len() as i32;
    Parallel::for_each(
        items.into_iter(),
        |item| record_failure(&result, operation(item)),
        false,
        count,
    )?;
    std::mem::replace(&mut *result.write(), Ok(()))
}

fn sequential_items<T>(
    items: impl IntoIterator<Item = T>,
    operation: impl Fn(T) -> Result<(), Failure>,
) -> Result<(), Failure> {
    for item in items {
        operation(item)?;
    }
    Ok(())
}
fn record_failure(destination: &SharedState<Result<(), Failure>>, result: Result<(), Failure>) {
    if let Err(error) = result {
        *destination.write() = Err(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequential_failure_stops_before_later_items() {
        let visited = SharedState::new(Vec::new());
        let result = model_items(vec![0, 1, 2], false, |item| {
            visited.write().push(item);
            if item == 1 {
                return Err(Failure::out_of_range(
                    "item 1; expected successful callback",
                ));
            }
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(*visited.read(), vec![0, 1]);
    }
    #[test]
    fn parallel_failure_is_returned_after_all_callbacks_finish() {
        let visited = SharedState::new(Vec::new());
        let result = model_items(vec![0, 1, 2], true, |item| {
            visited.write().push(item);
            if item == 1 {
                return Err(Failure::out_of_range(
                    "item 1; expected successful callback",
                ));
            }
            Ok(())
        });
        assert!(result.is_err());
        visited.write().sort_unstable();
        assert_eq!(*visited.read(), vec![0, 1, 2]);
    }
}
