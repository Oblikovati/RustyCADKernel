use super::*;
/// Records target copies (C), inspections (I) and destructors (D) in call order. The native
/// rows also print, inside each callback, the arena's live node count and the cell's list
/// links as the destructor sees them; both are protected state (plan D1 class B6) reached only
/// by aliasing the cell during the callback, so they are stripped before comparing.
#[derive(Default)]
pub(super) struct Probe {
    pub(super) events: String,
    pub(super) recording: bool,
    pub(super) fail_clone: Option<i32>,
    pub(super) fail_drop: Option<i32>,
}
impl Probe {
    fn record_drop(&mut self, number: i32) {
        write!(self.events, "D{number};").unwrap();
        if self.fail_drop == Some(number) {
            self.fail_drop = None;
            panic!("requested destructor failure for target {number}");
        }
    }
    fn record_clone(&mut self, number: i32) {
        write!(self.events, "C{number};").unwrap();
        if self.fail_clone == Some(number) {
            panic!("requested clone failure for target {number}");
        }
    }
}
pub(super) struct Target {
    pub(super) number: i32,
    stored: bool,
    pub(super) probe: Rc<RefCell<Probe>>,
}
impl Target {
    pub(super) fn source(number: i32, probe: &Rc<RefCell<Probe>>) -> Self {
        Self {
            number,
            stored: false,
            probe: probe.clone(),
        }
    }
}
impl Clone for Target {
    fn clone(&self) -> Self {
        self.probe.borrow_mut().record_clone(self.number);
        Self {
            number: self.number,
            stored: true,
            probe: self.probe.clone(),
        }
    }
}
impl Drop for Target {
    fn drop(&mut self) {
        if self.stored && self.probe.borrow().recording {
            self.probe.borrow_mut().record_drop(self.number);
        }
    }
}
pub(super) fn filled(mode: usize) -> (CellFilter<Inspector>, Rc<RefCell<Probe>>) {
    let arena = BaseAllocator::incremental(1024);
    let mut filter = CellFilter::with_allocator(1., Some(arena));
    let probe = Rc::new(RefCell::new(Probe::default()));
    for number in 1..=3 {
        filter.add(&Target::source(number, &probe), &[0.]);
    }
    native_row(&format!("copy{mode} {}", probe.borrow().events));
    let mut observed = probe.borrow_mut();
    observed.events.clear();
    observed.recording = true;
    drop(observed);
    (filter, probe)
}
pub(super) fn operation(
    mode: usize,
    filter: &mut CellFilter<Inspector>,
    probe: &Rc<RefCell<Probe>>,
) {
    match mode {
        0..=2 => filter.remove(&Target::source(3 - mode as i32, probe), &[0.]),
        3..=4 => filter.inspect(&[0.], &mut Inspector { all: mode == 3 }),
        5 => {
            let arena = filter.resource.clone();
            filter.reset(2., Some(arena)).unwrap()
        }
        _ => filter.reset(2., None).unwrap(),
    }
}
pub(super) fn remaining(row: &mut String, filter: &CellFilter<Inspector>) {
    row.push_str(" R");
    for (_, cell) in filter.cells.items() {
        for target in cell.targets() {
            write!(row, "{},", target.number).unwrap();
        }
    }
    write!(row, " cells{}", filter.cells.len()).unwrap();
}
pub(super) fn native_row(actual: &str) {
    let label = actual.split_once(' ').unwrap().0;
    let expected = include_str!(
        "../../../../../../../tools/oracle/tkernel_cell_filter_native_engine.expected"
    )
    .lines()
    .find(|row| row.split_once(' ').unwrap().0 == label)
    .unwrap();
    assert_eq!(actual, without_callback_state(expected));
}
// Drops the `[links]` of each destructor event and the `:count` of each copy event.
fn without_callback_state(row: &str) -> String {
    let mut kept = String::with_capacity(row.len());
    let mut skipping = None;
    for character in row.chars() {
        match (skipping, character) {
            (Some(']'), ']') | (Some(';'), ';') => {
                skipping = None;
                if character == ';' {
                    kept.push(';');
                }
            }
            (Some(_), _) => {}
            (None, '[') => skipping = Some(']'),
            (None, ':') if row.starts_with("copy") => skipping = Some(';'),
            (None, other) => kept.push(other),
        }
    }
    kept
}
