// SPDX-License-Identifier: GPL-2.0-only
use super::{operation_tests::accepted, *};
#[test]
fn complete_operation_replay_across_processes() {
    let (result, observer) = accepted();
    let Some(path) = std::env::var_os("MARTENSITE_COMPLETE_OWNED_RECEIPT") else {
        return;
    };
    std::fs::write(path, result.replay_bytes()).unwrap();
    write_trace(&observer.points);
    println!("COMPLETE_OWNED_OUTER_CHECKPOINTS={}", observer.points.len());
}
#[test]
fn complete_independent_threads_and_shared_readers_match_serial_owned_result() {
    let (result, observer) = accepted();
    let expected = result.replay_bytes();
    let workers: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                let (result, observer) = accepted();
                (result.replay_bytes(), observer.points)
            })
        })
        .collect();
    for worker in workers {
        let (bytes, points) = worker.join().unwrap();
        assert_eq!(bytes, expected);
        assert_eq!(points, observer.points);
    }
    assert_shared_reads(&result, &expected);
}
#[test]
fn complete_failure_and_success_do_not_depend_on_earlier_thread_work() {
    let expected = accepted().0.replay_bytes();
    let mut request = super::super::test_fixture::request();
    request.sources.clear();
    with_owned_prism_operation(
        request,
        &mut super::operation_tests::CompleteObserver::default(),
        |result| {
            assert!(matches!(
                result.err().unwrap().cause(),
                ScopedPrismCause::Preparation(_)
            ));
        },
    );
    assert_eq!(accepted().0.replay_bytes(), expected);
}

fn write_trace(points: &[CompletePrismCheckpoint]) {
    let Some(path) = std::env::var_os("MARTENSITE_COMPLETE_OWNED_TRACE") else {
        return;
    };
    // This is an exact test observation, not a new persistent diagnostic encoding.
    std::fs::write(path, format!("{points:?}\n")).unwrap();
}

fn assert_shared_reads(result: &OwnedCompletePrism, expected: &[u8]) {
    std::thread::scope(|scope| {
        let readers: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| result.replay_bytes()))
            .collect();
        for reader in readers {
            assert_eq!(reader.join().unwrap(), expected);
        }
    });
}
