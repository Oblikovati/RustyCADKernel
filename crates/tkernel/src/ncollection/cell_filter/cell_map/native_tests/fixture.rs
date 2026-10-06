use super::*;
pub(super) struct Probe {
    pub(super) arena: BaseAllocator,
    pub(super) cell: Option<*mut CellRecord<Target>>,
    labels: BTreeMap<usize, i32>,
    pub(super) events: String,
    pub(super) fail_clone: Option<i32>,
    pub(super) fail_drop: Option<i32>,
}
impl Probe {
    fn edges(&self) -> String {
        let Some(cell) = self.cell else {
            return String::new();
        };
        let mut edges = String::new();
        // Fixture lifetime pins the Cell allocation through every observed Object Drop.
        // Incremental storage keeps Next initialized after prior Object destruction during Clear.
        let mut node = unsafe { (*cell).objects };
        while !node.is_null() {
            let next = unsafe { (*node).next };
            let next_label = self.labels.get(&next.addr()).copied().unwrap_or(0);
            write!(edges, "{}>{next_label},", self.labels[&node.addr()]).unwrap();
            node = next;
        }
        edges
    }
    fn record_drop(&mut self, number: i32) {
        let edges = self.edges();
        write!(self.events, "D{number}[{edges}];").unwrap();
        if self.fail_drop == Some(number) {
            self.fail_drop = None;
            panic!("requested destructor failure for target {number}");
        }
    }
    pub(super) fn live_leases(&self) -> usize {
        self.arena
            .reset(false)
            .unwrap_err()
            .message()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap()
    }
    fn record_clone(&mut self, number: i32) {
        let live = self.live_leases();
        write!(self.events, "C{number}:{live};").unwrap();
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
        if self.stored && self.probe.borrow().cell.is_some() {
            self.probe.borrow_mut().record_drop(self.number);
        }
    }
}
pub(super) fn filled(mode: usize) -> (CellFilter<Inspector>, Rc<RefCell<Probe>>) {
    let arena = BaseAllocator::incremental(1024);
    let mut filter = CellFilter::with_allocator(1., Some(arena.clone()));
    let probe = Rc::new(RefCell::new(Probe {
        arena,
        cell: None,
        labels: BTreeMap::new(),
        events: String::new(),
        fail_clone: None,
        fail_drop: None,
    }));
    for number in 1..=3 {
        filter.add(&Target::source(number, &probe), &[0.]);
    }
    native_row(&format!("copy{mode} {}", probe.borrow().events));
    bind_probe(&filter, &probe);
    probe.borrow_mut().events.clear();
    (filter, probe)
}
fn bind_probe(filter: &CellFilter<Inspector>, probe: &Rc<RefCell<Probe>>) {
    let index = filter.indices(&[0.]);
    let (_, map_node) = filter.cells.lookup_index(&index).unwrap();
    let cell = unsafe { CellMapNode::cell_ptr(map_node.cast()) };
    let mut observed = probe.borrow_mut();
    observed.cell = Some(cell);
    let mut node = unsafe { (*cell).objects };
    while !node.is_null() {
        observed
            .labels
            .insert(node.addr(), unsafe { object_number(node) });
        node = unsafe { (*node).next };
    }
}
pub(super) fn operation(
    mode: usize,
    filter: &mut CellFilter<Inspector>,
    probe: &Rc<RefCell<Probe>>,
) {
    let arena = probe.borrow().arena.clone();
    match mode {
        0..=2 => filter.remove(&Target::source(3 - mode as i32, probe), &[0.]),
        3..=4 => filter.inspect(&[0.], &mut Inspector { all: mode == 3 }),
        5 => filter.reset(2., Some(arena)).unwrap(),
        _ => filter.reset(2., None).unwrap(),
    }
}
pub(super) fn remaining(row: &mut String, filter: &CellFilter<Inspector>) {
    row.push_str(" R");
    for (_, cell) in filter.cells.items() {
        let mut node = cell.objects;
        while !node.is_null() {
            write!(row, "{},", unsafe { object_number(node) }).unwrap();
            node = unsafe { (*node).next };
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
    assert_eq!(actual, expected);
}

unsafe fn object_number(node: *const CellObjectNode<Target>) -> i32 {
    // Test labels are read only while Object is live; no whole-node reference is created.
    unsafe { (*std::ptr::addr_of!((*node).object).cast::<Target>()).number }
}
