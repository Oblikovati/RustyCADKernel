//! Owned expectations precede the lifecycle change: unleased-fixtures.json at da38095c0.
use super::*;
use rustycad_tkernel::standard::Failure;
use std::sync::Mutex;

pub(super) struct RecordingDisplayCalls {
    pub open_result: usize,
    pub calls: Mutex<Vec<String>>,
}
impl RecordingDisplayCalls {
    pub fn new(open_result: usize) -> Arc<Self> {
        Arc::new(Self {
            open_result,
            calls: Mutex::default(),
        })
    }
    pub fn recorded(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}
impl NativeDisplayCalls for RecordingDisplayCalls {
    fn open(&self, name: &AsciiString) -> usize {
        self.calls.lock().unwrap().push(format!(
            "open({})",
            String::from_utf8_lossy(name.as_bytes())
        ));
        self.open_result
    }
    fn close(&self, display: usize) {
        self.calls.lock().unwrap().push(format!("close({display})"));
    }
    fn free_visual(&self, visual: usize) {
        self.calls.lock().unwrap().push(format!("free({visual})"));
    }
    fn delete_atom(&self, display: usize) -> Option<u64> {
        self.calls.lock().unwrap().push(format!("atom({display})"));
        Some(73)
    }
}
type UnleasedCase = (
    usize,
    bool,
    usize,
    usize,
    usize,
    bool,
    &'static [&'static str],
);
const UNLEASED_CASES: &[UnleasedCase] = &[
    (0, false, 0, 41, 41, true, &["open(:fixture)", "atom(41)"]),
    (
        41,
        true,
        0,
        42,
        42,
        true,
        &["close(41)", "open(:fixture)", "atom(42)"],
    ),
    (41, true, 55, 42, 55, false, &["close(41)", "atom(55)"]),
    (55, false, 0, 42, 42, true, &["open(:fixture)", "atom(42)"]),
    (41, true, 0, 0, 0, false, &["close(41)", "open(:fixture)"]),
    // This dangerous alias is observed only through the fake, never a freed native pointer.
    (41, true, 41, 42, 41, false, &["close(41)", "atom(41)"]),
];
pub(super) fn fake_connection(native: Arc<RecordingDisplayCalls>) -> AspectDisplayConnection {
    AspectDisplayConnection::with_native(AsciiString::from(":fixture"), native)
}
#[test]
fn unleased_reset_preserves_owned_calls_and_state() {
    for case in UNLEASED_CASES {
        assert_unleased_case(case);
    }
}
fn assert_unleased_case(case: &UnleasedCase) {
    let &(initial, owned, requested, opened, expected, expected_owned, calls) = case;
    let native = RecordingDisplayCalls::new(opened);
    let connection = fake_connection(native.clone());
    set_initial_connection(&connection, initial, owned);
    let result = connection.init(requested);
    assert_eq!(native.recorded(), calls);
    assert_eq!(connection.get_display_aspect(), expected);
    assert_eq!(connection.is_own_display(), expected_owned);
    assert_eq!(
        connection.get_atom(AspectXAtom::DeleteWindow).ok(),
        (expected != 0).then_some(73)
    );
    assert_fixture_error(result, expected);
}
fn set_initial_connection(connection: &AspectDisplayConnection, initial: usize, owned: bool) {
    let mut state = connection.state.write();
    state.display = initial;
    state.is_own_display = owned;
    state.atoms.insert(AspectXAtom::DeleteWindow, 19);
}
fn assert_fixture_error(result: Result<(), Failure>, expected: usize) {
    if expected != 0 {
        assert!(result.is_ok());
        return;
    }
    let failure = result.unwrap_err();
    assert_eq!(
        failure.exception_type(),
        "Aspect_DisplayConnectionDefinitionError"
    );
    assert_eq!(
        failure.message(),
        "Can not connect to the server \":fixture\""
    );
}
#[test]
fn unleased_visual_metadata_and_destructor_order_are_preserved() {
    let native = RecordingDisplayCalls::new(41);
    let connection = fake_connection(native.clone());
    connection.set_default_visual_info(17, 23);
    connection.set_default_visual_info(18, 24);
    connection.init(0).unwrap();
    assert_eq!(connection.get_default_visual_info(), 18);
    assert_eq!(connection.get_default_fb_config(), 24);
    drop(connection);
    assert_eq!(native.recorded(), EXPECTED_VISUAL_CALLS);
}

const EXPECTED_VISUAL_CALLS: &[&str] = &[
    "free(17)",
    "open(:fixture)",
    "atom(41)",
    "free(18)",
    "close(41)",
];
