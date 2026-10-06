#[derive(Clone, Copy)]
pub enum Operation {
    Emplace,
    Emplaced,
    TryEmplace,
    TryEmplaced,
}
impl Operation {
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
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RejectedPhase {
    None,
    Construct,
    Hash,
    Equal,
}
#[derive(Clone, Copy)]
pub struct EmplaceCase {
    pub label: &'static str,
    pub count: usize,
    pub key: i32,
    pub rejected: RejectedPhase,
}
pub fn cases() -> [EmplaceCase; 10] {
    use RejectedPhase::*;
    [
        case("empty_ok", 0, 7, None),
        case("empty_construct", 0, 7, Construct),
        case("empty_hash", 0, 999, Hash),
        case("insert", 3, 999, None),
        case("duplicate", 3, 7, None),
        case("construct", 3, 7, Construct),
        case("hash", 3, 999, Hash),
        case("equal", 3, 7, Equal),
        case("grow", 102, 999, None),
        case("grow_construct", 102, 999, Construct),
    ]
}
fn case(label: &'static str, count: usize, key: i32, rejected: RejectedPhase) -> EmplaceCase {
    EmplaceCase {
        label,
        count,
        key,
        rejected,
    }
}
