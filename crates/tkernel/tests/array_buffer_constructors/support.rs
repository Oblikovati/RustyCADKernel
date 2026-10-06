use std::{cell::Cell, rc::Rc};

#[derive(Default)]
pub struct ConstructionCounters {
    pub defaults: usize,
    pub drops: Cell<usize>,
}

pub struct BufferItem {
    pub value: i32,
    pub counters: Rc<ConstructionCounters>,
}

impl BufferItem {
    pub fn supplied(value: i32, counters: &Rc<ConstructionCounters>) -> Self {
        Self {
            value,
            counters: Rc::clone(counters),
        }
    }
}

impl Default for BufferItem {
    fn default() -> Self {
        Self {
            value: 37,
            counters: Rc::new(ConstructionCounters {
                defaults: 1,
                drops: Cell::new(0),
            }),
        }
    }
}

impl Drop for BufferItem {
    fn drop(&mut self) {
        self.counters.drops.set(self.counters.drops.get() + 1);
    }
}

pub fn capture_counters(values: &[BufferItem]) -> Vec<Rc<ConstructionCounters>> {
    let mut counters = Vec::new();
    for item in values {
        if !counters
            .iter()
            .any(|entry| Rc::ptr_eq(entry, &item.counters))
        {
            counters.push(Rc::clone(&item.counters));
        }
    }
    counters
}

pub fn counter_row(counters: &[Rc<ConstructionCounters>]) -> String {
    let defaults = counters.iter().map(|entry| entry.defaults).sum::<usize>();
    let drops = counters
        .iter()
        .map(|entry| entry.drops.get())
        .sum::<usize>();
    // BufferItem deliberately has no Clone implementation, proving zero element copies.
    format!("{defaults},0,{drops}")
}

pub fn probe_buffer<const N: usize>() -> [BufferItem; N] {
    let counters = Rc::new(ConstructionCounters::default());
    std::array::from_fn(|index| BufferItem::supplied(index as i32 + 1, &counters))
}

pub fn values_row(values: &[BufferItem]) -> String {
    values
        .iter()
        .map(|item| item.value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn assert_native_row(label: &str, value: impl std::fmt::Display) {
    let expected =
        include_str!("../../../../tools/oracle/tkernel_array_buffer_constructors.expected");
    let row = format!("{label}={value}");
    assert!(
        expected.lines().any(|line| line == row),
        "missing native row: {row}"
    );
}
