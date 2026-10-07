//! Port-only tests of the TFunction mechanism: a chain of functions A -> B -> C whose
//! drivers read and write child labels, its dependency graph, the iterator in both modes,
//! the logbook and the driver table.
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Guid, Handle, StandardTransient};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::tdf::{TdfData, TdfLabel, TdfLabelMap};
use rustycad_tklcaf::tfunction::{
    TFunctionDriver, TFunctionDriverBase, TFunctionDriverTable, TFunctionExecutionStatus,
    TFunctionIFunction, TFunctionIterator, TFunctionLogbook, TFunctionScope,
};
use std::sync::{Mutex, MutexGuard};

/// A fake driver: the arguments are child 1 of the function label's father (`input`),
/// the result is child 2 of the function label (`output`), keyed by tags.
#[derive(Default)]
struct ChainDriver {
    transient: StandardTransient,
    base: TFunctionDriverBase,
}
impl_transient!(ChainDriver, "ChainDriver", dyn TFunctionDriver, transient, interfaces: [dyn TFunctionDriver]);

impl TFunctionDriver for ChainDriver {
    fn driver_base(&self) -> &TFunctionDriverBase {
        &self.base
    }
    fn execute(&self, log: &mut Handle<TFunctionLogbook>) -> Result<i32, Failure> {
        self.validate(log)?;
        Ok(0)
    }
    fn arguments(&self, arguments: &mut Vec<TdfLabel>) {
        let label = self.label();
        if label.tag() > 1 {
            let previous = label
                .father()
                .find_child(label.tag() - 1, false)
                .unwrap_or_default();
            arguments.push(previous.find_child(2, true).unwrap_or_default());
        }
    }
    fn results(&self, results: &mut Vec<TdfLabel>) {
        results.push(self.label().find_child(2, true).unwrap_or_default());
    }
}

/// The driver GUID of the test, unique to this file (the table is process-wide).
const CHAIN_DRIVER_ID: Guid = guid_from_text("0f2a9a10-1111-4000-8000-0000000000aa");

fn register_driver() {
    let table = TFunctionDriverTable::get();
    let driver: Handle<dyn TFunctionDriver> = Handle::new(ChainDriver::default()).cast();
    table.add_driver(&CHAIN_DRIVER_ID, &driver, 0);
}

/// Serializes the tests that use the process-wide driver: `GetDriver` re-initializes the
/// one shared driver with the function label, as natively.
static SHARED_DRIVER: Mutex<()> = Mutex::new(());

/// Three functions on labels 1, 2, 3 under one father, connected by their drivers; the
/// guard holds the shared driver for the test.
fn chain(data: &Handle<TdfData>) -> (MutexGuard<'static, ()>, Vec<TdfLabel>) {
    let guard = SHARED_DRIVER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    register_driver();
    let father = data.root().find_child(7, true).unwrap();
    let labels: Vec<TdfLabel> = (1..=3)
        .map(|tag| father.find_child(tag, true).unwrap())
        .collect();
    for label in &labels {
        assert!(TFunctionIFunction::new_function(label, &CHAIN_DRIVER_ID).unwrap());
    }
    assert!(TFunctionIFunction::update_all_dependencies(&labels[0]).unwrap());
    (guard, labels)
}

#[test]
fn update_dependencies_links_the_chain() {
    let data = TdfData::new();
    let (_guard, labels) = chain(&data);
    let mut next = Vec::new();
    TFunctionIFunction::new(&labels[0])
        .get_next(&mut next)
        .unwrap();
    assert_eq!(next, vec![labels[1].clone()]);
    let mut previous = Vec::new();
    TFunctionIFunction::new(&labels[2])
        .get_previous(&mut previous)
        .unwrap();
    assert_eq!(previous, vec![labels[1].clone()]);
}

#[test]
fn iterator_walks_level_by_level() {
    let data = TdfData::new();
    let (_guard, labels) = chain(&data);
    let mut iterator = TFunctionIterator::new(&labels[0]).unwrap();
    let mut levels = Vec::new();
    while iterator.more().unwrap() {
        levels.push(iterator.current().to_vec());
        iterator.next().unwrap();
    }
    assert_eq!(
        levels,
        vec![
            vec![labels[0].clone()],
            vec![labels[1].clone()],
            vec![labels[2].clone()]
        ]
    );
    let error = TFunctionIterator::new(&labels[0])
        .unwrap()
        .get_max_nb_threads()
        .unwrap_err();
    assert!(
        error.is_kind("Standard_NullObject"),
        "the native walk has no scope"
    );
}

#[test]
fn execution_mode_waits_for_succeeded_previous_functions() {
    let data = TdfData::new();
    let (_guard, labels) = chain(&data);
    for label in &labels {
        TFunctionIFunction::new(label)
            .set_status(TFunctionExecutionStatus::NotExecuted)
            .unwrap();
    }
    let mut iterator = TFunctionIterator::empty();
    iterator.set_usage_of_execution_status(true);
    iterator.init(&labels[0]).unwrap();
    assert_eq!(iterator.current(), &labels[..1]);
    iterator.next().unwrap();
    assert_eq!(
        iterator.current(),
        &labels[..1],
        "not executed yet: stays current"
    );
    iterator
        .set_status(&labels[0], TFunctionExecutionStatus::Succeeded)
        .unwrap();
    iterator.next().unwrap();
    assert_eq!(iterator.current(), &labels[1..2]);
    let mut text = Vec::new();
    iterator.dump(&mut text).unwrap();
    assert!(String::from_utf8(text).unwrap().starts_with("Functions:\n"));
    assert_eq!(
        iterator.get_status(&labels[0]).unwrap(),
        TFunctionExecutionStatus::Succeeded
    );
}

#[test]
fn delete_function_disconnects_its_neighbours() {
    let data = TdfData::new();
    let (_guard, labels) = chain(&data);
    assert!(TFunctionIFunction::delete_function(&labels[1]).unwrap());
    let scope = TFunctionScope::set(&labels[0]).unwrap();
    assert!(!scope.has_function_label(&labels[1]));
    let mut next = Vec::new();
    TFunctionIFunction::new(&labels[0])
        .get_next(&mut next)
        .unwrap();
    assert!(next.is_empty());
    assert!(
        TFunctionIFunction::new(&labels[1])
            .get_graph_node()
            .is_err()
    );
}

#[test]
fn driver_executes_and_validates_through_the_logbook() {
    let data = TdfData::new();
    let (_guard, labels) = chain(&data);
    let function = TFunctionIFunction::new(&labels[1]);
    let driver = function.get_driver(0).unwrap();
    let mut logbook = TFunctionLogbook::set(&labels[0]).unwrap();
    logbook
        .set_touched(&labels[0].find_child(2, true).unwrap())
        .unwrap();
    assert!(driver.must_execute(&logbook).unwrap());
    assert_eq!(driver.execute(&mut logbook).unwrap(), 0);
    let mut valid = TdfLabelMap::default();
    logbook.get_valid_into(&mut valid);
    assert!(valid.contains(&labels[1].find_child(2, true).unwrap()));
    logbook.done(true).unwrap();
    assert!(logbook.is_done());
    logbook.clear().unwrap();
    assert!(logbook.is_empty());
}

#[test]
fn thread_tables_grow_and_remove_returns_false() {
    let table = TFunctionDriverTable::new();
    let driver: Handle<dyn TFunctionDriver> = Handle::new(ChainDriver::default()).cast();
    let id = guid_from_text("0f2a9a10-1111-4000-8000-0000000000bb");
    assert!(table.add_driver(&id, &driver, 3));
    assert!(table.has_driver(&id, 3) && !table.has_driver(&id, 1) && !table.has_driver(&id, 0));
    assert!(!table.remove_driver(&id, 3));
    assert!(!table.has_driver(&id, 3));
    assert!(!table.add_driver(&id, &driver, -1));
    table.clear();
    assert!(!table.has_driver(&id, 2));
}
