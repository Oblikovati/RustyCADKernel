//! Rust side of the Standard_Mutex / Standard_Condition scripts (tkernel_standard_services_sync.cxx).
use rustycad_tkernel::standard::mutex::Sentry;
use rustycad_tkernel::standard::{StandardCondition, StandardMutex};
use std::str::SplitWhitespace;
use std::thread;
use std::time::Duration;

/// A token answers 0/1, or nothing ("-").
fn answer(value: Option<bool>) -> String {
    match value {
        Some(true) => "1".into(),
        Some(false) => "0".into(),
        None => "-".into(),
    }
}

/// "services_mutex <token>...": see the native case for the token meanings.
pub fn mutex_case(input: &mut SplitWhitespace<'_>) -> String {
    let mutex = StandardMutex::new();
    let mut sentries: Vec<Sentry<'_>> = Vec::new();
    let answers: Vec<String> = input
        .map(|token| answer(mutex_step(&mutex, &mut sentries, token)))
        .collect();
    answers.join(" ")
}

fn mutex_step<'a>(
    mutex: &'a StandardMutex,
    sentries: &mut Vec<Sentry<'a>>,
    token: &str,
) -> Option<bool> {
    match token {
        "L" => mutex.lock(),
        "U" => mutex.unlock(),
        "T" => return Some(mutex.try_lock()),
        "t" => return Some(on_other_thread(|| other_try_lock(mutex))),
        "u" => on_other_thread(|| mutex.unlock()),
        "S" => sentries.push(Sentry::new(mutex)),
        "N" => sentries.push(Sentry::from_option(None)),
        _ => drop(sentries.pop()),
    }
    None
}

fn on_other_thread<T: Send>(work: impl FnOnce() -> T + Send) -> T {
    thread::scope(|scope| scope.spawn(work).join().expect("probe thread"))
}

fn other_try_lock(mutex: &StandardMutex) -> bool {
    let locked = mutex.try_lock();
    if locked {
        mutex.unlock();
    }
    locked
}

/// "services_condition <initial> <token>...": see the native case for the token meanings.
pub fn condition_case(input: &mut SplitWhitespace<'_>) -> String {
    let initial = input.next().expect("initial state") != "0";
    let condition = StandardCondition::new(initial);
    let answers: Vec<String> = input
        .map(|token| answer(condition_step(&condition, token)))
        .collect();
    answers.join(" ")
}

fn condition_step(condition: &StandardCondition, token: &str) -> Option<bool> {
    let (operation, arguments) = token.split_at(1);
    match operation {
        "S" => condition.set(),
        "R" => condition.reset(),
        "C" => return Some(condition.check()),
        "X" => return Some(condition.check_reset()),
        "W" => return Some(condition.wait_timeout(arguments.parse().expect("milliseconds"))),
        _ => return threaded_wait(condition, operation, arguments),
    }
    None
}

fn threaded_wait(condition: &StandardCondition, operation: &str, arguments: &str) -> Option<bool> {
    let (delay, limit) = arguments.split_once(',').unwrap_or((arguments, "0"));
    let delay: u64 = delay.parse().expect("delay");
    thread::scope(|scope| {
        scope.spawn(|| {
            thread::sleep(Duration::from_millis(delay));
            condition.set();
        });
        if operation == "D" {
            return Some(condition.wait_timeout(limit.parse().expect("milliseconds")));
        }
        condition.wait();
        None
    })
}
