use super::fixture::BorrowTrace;
#[derive(Clone, Copy)]
pub enum Method {
    Emplace,
    Emplaced,
    TryEmplace,
    TryEmplaced,
}
impl Method {
    pub fn name(self) -> &'static str {
        match self {
            Self::Emplace => "emplace",
            Self::Emplaced => "emplaced",
            Self::TryEmplace => "try_emplace",
            Self::TryEmplaced => "try_emplaced",
        }
    }
    pub fn is_try(self) -> bool {
        matches!(self, Self::TryEmplace | Self::TryEmplaced)
    }
}
#[derive(Clone, Copy)]
pub struct BorrowCase {
    pub label: &'static str,
    pub count: usize,
    pub key: i32,
    pub value: bool,
    pub copy: bool,
    pub hash: bool,
    pub equal: bool,
}
impl BorrowCase {
    pub fn configure(self, trace: &BorrowTrace<'_>) {
        trace.active.set(true);
        trace.fail_value.set(self.value);
        trace.fail_key.set(self.copy);
        trace.fail_hash.set(self.hash);
        trace.fail_equal.set(self.equal);
    }
}
const EMPTY: BorrowCase = BorrowCase {
    label: "empty",
    count: 0,
    key: 7,
    value: false,
    copy: false,
    hash: false,
    equal: false,
};
const INSERT: BorrowCase = BorrowCase {
    label: "insert",
    count: 3,
    key: 999,
    ..EMPTY
};
const DUPLICATE: BorrowCase = BorrowCase {
    label: "duplicate",
    count: 3,
    ..EMPTY
};
const GROW: BorrowCase = BorrowCase {
    label: "grow",
    count: 102,
    ..INSERT
};
pub const CASES: [BorrowCase; 12] = [
    EMPTY,
    BorrowCase {
        label: "empty_value",
        value: true,
        ..EMPTY
    },
    BorrowCase {
        label: "empty_copy",
        copy: true,
        ..EMPTY
    },
    INSERT,
    DUPLICATE,
    BorrowCase {
        label: "duplicate_copy",
        copy: true,
        ..DUPLICATE
    },
    BorrowCase {
        label: "duplicate_value",
        value: true,
        ..DUPLICATE
    },
    BorrowCase {
        label: "hash",
        hash: true,
        ..INSERT
    },
    BorrowCase {
        label: "equal",
        equal: true,
        ..DUPLICATE
    },
    GROW,
    BorrowCase {
        label: "grow_value",
        value: true,
        ..GROW
    },
    BorrowCase {
        label: "grow_copy",
        copy: true,
        ..GROW
    },
];
