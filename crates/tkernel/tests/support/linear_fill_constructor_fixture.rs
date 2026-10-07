//! Fill kernel observations keep the Rust owned-donor wrapper separately labeled.
use super::LinearVector;
pub use super::item::{CopyItem, CopyTrace};

pub struct FillSpec {
    pub name: &'static str,
    pub count: usize,
    pub fail: usize,
    pub owned: bool,
}

pub const FILL_CASES: [FillSpec; 20] = [
    FillSpec {
        name: "borrowed-zero",
        count: 0,
        fail: 0,
        owned: false,
    },
    FillSpec {
        name: "borrowed-one",
        count: 1,
        fail: 0,
        owned: false,
    },
    FillSpec {
        name: "borrowed-two",
        count: 2,
        fail: 0,
        owned: false,
    },
    FillSpec {
        name: "borrowed-three",
        count: 3,
        fail: 0,
        owned: false,
    },
    FillSpec {
        name: "borrowed-five",
        count: 5,
        fail: 0,
        owned: false,
    },
    FillSpec {
        name: "borrowed-one-first",
        count: 1,
        fail: 1,
        owned: false,
    },
    FillSpec {
        name: "borrowed-three-first",
        count: 3,
        fail: 1,
        owned: false,
    },
    FillSpec {
        name: "borrowed-middle",
        count: 3,
        fail: 2,
        owned: false,
    },
    FillSpec {
        name: "borrowed-final",
        count: 3,
        fail: 3,
        owned: false,
    },
    FillSpec {
        name: "borrowed-long-final",
        count: 5,
        fail: 5,
        owned: false,
    },
    FillSpec {
        name: "owned-zero",
        count: 0,
        fail: 0,
        owned: true,
    },
    FillSpec {
        name: "owned-one",
        count: 1,
        fail: 0,
        owned: true,
    },
    FillSpec {
        name: "owned-two",
        count: 2,
        fail: 0,
        owned: true,
    },
    FillSpec {
        name: "owned-three",
        count: 3,
        fail: 0,
        owned: true,
    },
    FillSpec {
        name: "owned-five",
        count: 5,
        fail: 0,
        owned: true,
    },
    FillSpec {
        name: "owned-one-first",
        count: 1,
        fail: 1,
        owned: true,
    },
    FillSpec {
        name: "owned-three-first",
        count: 3,
        fail: 1,
        owned: true,
    },
    FillSpec {
        name: "owned-middle",
        count: 3,
        fail: 2,
        owned: true,
    },
    FillSpec {
        name: "owned-final",
        count: 3,
        fail: 3,
        owned: true,
    },
    FillSpec {
        name: "owned-long-final",
        count: 5,
        fail: 5,
        owned: true,
    },
];

pub fn row(
    phase: &str,
    spec: &FillSpec,
    result: Option<&LinearVector<CopyItem<'_>>>,
    trace: &CopyTrace,
    success: bool,
) -> String {
    let outcome = if success { "ok" } else { "throw" };
    let (len, capacity, values) = result.map_or((0, 0, String::new()), |vector| {
        (
            vector.len(),
            vector.capacity(),
            vector
                .iter()
                .map(|item| format!("{},", item.value))
                .collect::<String>(),
        )
    });
    format!(
        "{phase}:{}:{outcome}:{len}:{capacity}:{}:{values}:{}:{}",
        spec.name,
        trace.events.borrow(),
        trace.resources.get(),
        u8::from(!spec.owned)
    )
}
