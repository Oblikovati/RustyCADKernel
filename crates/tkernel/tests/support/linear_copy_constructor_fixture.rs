//! Shared native observations; this module contains no allocation recovery or kernel policy.
use super::LinearVector;
#[path = "linear_constructor_item.rs"]
mod item;
pub use item::{CopyItem, CopyTrace};

pub struct CopySpec {
    pub name: &'static str,
    pub capacity: usize,
    pub count: usize,
    pub fail: usize,
    pub assigning: bool,
    pub receiver_count: usize,
}

pub const COPY_CASES: [CopySpec; 14] = [
    CopySpec {
        name: "clone-empty",
        capacity: 0,
        count: 0,
        fail: 0,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-reserved-empty",
        capacity: 8,
        count: 0,
        fail: 0,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-single",
        capacity: 8,
        count: 1,
        fail: 0,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-retained",
        capacity: 8,
        count: 3,
        fail: 0,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-first",
        capacity: 8,
        count: 3,
        fail: 1,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-middle",
        capacity: 8,
        count: 3,
        fail: 2,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-last",
        capacity: 8,
        count: 3,
        fail: 3,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "clone-long-final",
        capacity: 8,
        count: 5,
        fail: 5,
        assigning: false,
        receiver_count: 0,
    },
    CopySpec {
        name: "assign-empty-target",
        capacity: 8,
        count: 3,
        fail: 0,
        assigning: true,
        receiver_count: 0,
    },
    CopySpec {
        name: "assign-occupied",
        capacity: 8,
        count: 3,
        fail: 0,
        assigning: true,
        receiver_count: 1,
    },
    CopySpec {
        name: "assign-first",
        capacity: 8,
        count: 3,
        fail: 1,
        assigning: true,
        receiver_count: 1,
    },
    CopySpec {
        name: "assign-middle",
        capacity: 8,
        count: 3,
        fail: 2,
        assigning: true,
        receiver_count: 1,
    },
    CopySpec {
        name: "assign-last",
        capacity: 8,
        count: 3,
        fail: 3,
        assigning: true,
        receiver_count: 1,
    },
    CopySpec {
        name: "assign-empty-first",
        capacity: 8,
        count: 1,
        fail: 1,
        assigning: true,
        receiver_count: 0,
    },
];

pub fn filled(
    capacity: usize,
    count: usize,
    start: i32,
    trace: &CopyTrace,
) -> LinearVector<CopyItem<'_>> {
    let mut vector = LinearVector::with_capacity(capacity);
    for index in 0..count {
        vector.append(CopyItem::new(start + i32::try_from(index).unwrap(), trace));
    }
    vector
}

pub fn row(
    phase: &str,
    spec: &CopySpec,
    source: &LinearVector<CopyItem<'_>>,
    result: Option<&LinearVector<CopyItem<'_>>>,
    before: *const CopyItem<'_>,
    trace: &CopyTrace,
    success: bool,
) -> String {
    let outcome = if success { "ok" } else { "throw" };
    let values = |vector: &LinearVector<CopyItem<'_>>| {
        vector
            .iter()
            .map(|item| format!("{},", item.value))
            .collect::<String>()
    };
    let (len, capacity, same, result_values) =
        result.map_or((0, 0, false, String::new()), |vector| {
            (
                vector.len(),
                vector.capacity(),
                before == vector.data_ptr(),
                values(vector),
            )
        });
    format!(
        "{phase}:{}:{outcome}:{len}:{capacity}:{}:{}:{}:{result_values}:{}",
        spec.name,
        u8::from(same),
        trace.events.borrow(),
        values(source),
        trace.resources.get()
    )
}
