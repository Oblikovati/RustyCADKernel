//! Message progress differential scripts, the twin of
//! `tools/oracle/tkernel_message_progress_probe.cxx` (see its header for the grammar).
use rustycad_tkernel::message::{
    LazyProgressScope, ProgressIndicator, ProgressIndicatorHooks, ProgressRange, ProgressScope,
    ProgressSentry,
};
use rustycad_tkernel::tcollection::AsciiString;
use std::str::SplitWhitespace;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicI64, Ordering},
};

/// Records every show; breaks once polled more than `allowed` times (never if negative).
struct RecordingIndicator {
    allowed: i64,
    polls: AtomicI64,
    shows: Mutex<Vec<String>>,
}
/// Hooks sharing the record with the probe after the indicator took ownership.
struct RecordingHooks(Arc<RecordingIndicator>);
impl ProgressIndicatorHooks for RecordingHooks {
    fn user_break(&self) -> bool {
        let polls = self.0.polls.fetch_add(1, Ordering::SeqCst) + 1;
        self.0.allowed >= 0 && polls > self.0.allowed
    }
    fn show(&self, scope: &ProgressScope<'_>, position: f64, force: bool) {
        let entry = format!(
            "{} {} {}",
            position.to_bits(),
            quoted(scope.name()),
            u8::from(force)
        );
        self.0.shows.lock().unwrap().push(entry);
    }
}
fn quoted(name: Option<&[u8]>) -> String {
    name.map_or_else(
        || "~".to_owned(),
        |bytes| format!("'{}'", String::from_utf8_lossy(bytes)),
    )
}
/// The token stream and the text printed so far.
struct Script<'t, 'i> {
    tokens: &'t mut SplitWhitespace<'i>,
    out: String,
}
impl<'i> Script<'_, 'i> {
    fn token(&mut self) -> Option<&'i str> {
        self.tokens.next()
    }
    fn real(&mut self) -> f64 {
        self.tokens.next().expect("number").parse().expect("real")
    }
    fn flag(&mut self) -> bool {
        self.real() != 0.0
    }
    fn open_block(&mut self) {
        assert_eq!(self.token(), Some("["), "block must open with [");
    }
    fn skip_block(&mut self) {
        self.open_block();
        let mut depth = 1;
        while depth > 0 {
            match self.token() {
                Some("[") => depth += 1,
                Some("]") => depth -= 1,
                Some(_) => {}
                None => return,
            }
        }
    }
    fn print(&mut self, text: &str) {
        self.out.push_str(text);
        self.out.push(' ');
    }
}
fn bit(value: bool) -> u8 {
    u8::from(value)
}

fn run_range(s: &mut Script<'_, '_>, range: &mut ProgressRange<'_>) {
    while let Some(token) = s.token() {
        match token {
            "]" => return,
            "S" => scope_case(s, range),
            "k" => {
                s.open_block();
                let mut copy = range.clone();
                run_range(s, &mut copy);
            }
            "x" => range.close(),
            "r" => {
                let text = format!(
                    "R {} {} {}",
                    bit(range.is_active()),
                    bit(range.user_break()),
                    bit(range.more())
                );
                s.print(&text);
            }
            "y" => sentry_case(s, range),
            "z" => lazy_case(s, range),
            other => panic!("unknown range token {other}"),
        }
    }
}
fn scope_case(s: &mut Script<'_, '_>, range: &ProgressRange<'_>) {
    let kind = s.token().expect("scope kind").to_owned();
    let (max, infinite) = (s.real(), s.flag());
    s.open_block();
    let scope = match kind.as_str() {
        "L" => ProgressScope::with_literal(range, "Lit", max, infinite),
        "Z" => ProgressScope::with_literal(range, "", max, infinite),
        "N" => ProgressScope::unnamed(range, max, infinite),
        "S" => ProgressScope::new(range, &AsciiString::new("Str"), max, infinite),
        _ => ProgressScope::new(range, &AsciiString::new(""), max, infinite),
    };
    run_scope(s, &scope);
}
fn sentry_case(s: &mut Script<'_, '_>, range: &ProgressRange<'_>) {
    let (min, max, step) = (s.real(), s.real(), s.real());
    let (infinite, span) = (s.flag(), s.real());
    match ProgressSentry::new(range, "Sen", min, max, step, infinite, span) {
        Ok(sentry) => {
            s.open_block();
            run_scope(s, &sentry);
        }
        Err(error) => {
            s.print(&format!("E {}", error.exception_type()));
            s.skip_block();
        }
    }
}
fn lazy_case(s: &mut Script<'_, '_>, range: &ProgressRange<'_>) {
    let (max, patch, infinite, count) = (s.real(), s.real(), s.flag(), s.real());
    let mut lazy = LazyProgressScope::new(range, "Lazy", max, patch, infinite);
    for _ in 0..count as i32 {
        lazy.next();
    }
    let more = lazy.more();
    let aborted = lazy.is_aborted();
    let text = format!("Z {} {} {}", bit(more), bit(aborted), bit(lazy.more()));
    s.print(&text);
}
fn scope_value(s: &mut Script<'_, '_>, scope: &ProgressScope<'_>) {
    let parent = scope
        .parent()
        .map_or_else(|| "-".to_owned(), |parent| quoted(parent.name()));
    let text = format!(
        "V {} {} {} {} {} {} {parent}",
        scope.value().to_bits(),
        bit(scope.is_active()),
        scope.max_value().to_bits(),
        scope.get_portion().to_bits(),
        bit(scope.is_infinite()),
        quoted(scope.name())
    );
    s.print(&text);
}
fn run_scope(s: &mut Script<'_, '_>, scope: &ProgressScope<'_>) {
    while let Some(token) = s.token() {
        match token {
            "]" => return,
            "n" => {
                let step = s.real();
                s.open_block();
                let mut range = scope.next(step);
                run_range(s, &mut range);
            }
            "c" => scope.close(),
            "w" => scope.show(),
            "v" => scope_value(s, scope),
            "b" => {
                let text = format!("B {} {}", bit(scope.user_break()), bit(scope.more()));
                s.print(&text);
            }
            "a" => assign_case(s, scope),
            other => panic!("unknown scope token {other}"),
        }
    }
}
fn assign_case(s: &mut Script<'_, '_>, scope: &ProgressScope<'_>) {
    let (first, second) = (s.real(), s.real());
    let mut target = scope.next(first);
    let source = scope.next(second);
    target.clone_from(&source);
    let text = format!("A {} {}", bit(target.is_active()), bit(source.is_active()));
    s.print(&text);
}

/// Answer `msgprog`; `None` for any other operation.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    if op != "msgprog" {
        return None;
    }
    let allowed: i64 = input.next()?.parse().expect("allowed polls");
    let null = input.next()? == "null";
    let record = Arc::new(RecordingIndicator {
        allowed,
        polls: AtomicI64::new(0),
        shows: Mutex::new(Vec::new()),
    });
    let indicator = ProgressIndicator::new(Box::new(RecordingHooks(Arc::clone(&record))));
    let mut script = Script {
        tokens: input,
        out: String::new(),
    };
    {
        let mut range = ProgressIndicator::start_optional((!null).then_some(&indicator));
        run_range(&mut script, &mut range);
    }
    let shows = record.shows.lock().unwrap();
    let mut out = script.out;
    out.push_str(&format!(
        "P {} {} ",
        record.polls.load(Ordering::SeqCst),
        shows.len()
    ));
    for show in shows.iter() {
        out.push_str(show);
        out.push(' ');
    }
    out.push_str(&indicator.get_position().to_bits().to_string());
    Some(out)
}
