//! `Message_Status` and `Message_StatusType` as integer newtypes.
//!
//! Native code builds statuses arithmetically: `Message_ExecStatus::StatusByIndex(0)`
//! yields `(Message_Status)(Message_DONE - 1)`, a value outside the enumerators. An
//! integer newtype keeps such values representable, as the C++ enum does.

/// Status group of a `Status`, the high byte of its value; `StatusType::WARN`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StatusType(pub u32);
impl StatusType {
    /// `Message_DONE`: a successfully completed operation.
    pub const DONE: StatusType = StatusType(0x0000_0100);
    /// `Message_WARN`: a potentially wrong situation.
    pub const WARN: StatusType = StatusType(0x0000_0200);
    /// `Message_ALARM`: a severe warning about user data.
    pub const ALARM: StatusType = StatusType(0x0000_0400);
    /// `Message_FAIL`: the algorithm failed.
    pub const FAIL: StatusType = StatusType(0x0000_0800);
}

/// One execution status flag of `Message_Status`; `Status::WARN3`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Status(pub i32);

macro_rules! status_group {
    ($base:expr, $($name:ident = $offset:expr),* $(,)?) => {
        impl Status {
            $(
                #[doc = concat!("`Message_", stringify!($name), "`.")]
                pub const $name: Status = Status($base as i32 + $offset);
            )*
        }
    };
}
impl Status {
    /// `Message_None`: no status.
    pub const NONE: Status = Status(0);
}
status_group!(
    0x100,
    DONE1 = 0,
    DONE2 = 1,
    DONE3 = 2,
    DONE4 = 3,
    DONE5 = 4,
    DONE6 = 5,
    DONE7 = 6,
    DONE8 = 7,
    DONE9 = 8,
    DONE10 = 9,
    DONE11 = 10,
    DONE12 = 11,
    DONE13 = 12,
    DONE14 = 13,
    DONE15 = 14,
    DONE16 = 15,
    DONE17 = 16,
    DONE18 = 17,
    DONE19 = 18,
    DONE20 = 19,
    DONE21 = 20,
    DONE22 = 21,
    DONE23 = 22,
    DONE24 = 23,
    DONE25 = 24,
    DONE26 = 25,
    DONE27 = 26,
    DONE28 = 27,
    DONE29 = 28,
    DONE30 = 29,
    DONE31 = 30,
    DONE32 = 31
);
status_group!(
    0x200,
    WARN1 = 0,
    WARN2 = 1,
    WARN3 = 2,
    WARN4 = 3,
    WARN5 = 4,
    WARN6 = 5,
    WARN7 = 6,
    WARN8 = 7,
    WARN9 = 8,
    WARN10 = 9,
    WARN11 = 10,
    WARN12 = 11,
    WARN13 = 12,
    WARN14 = 13,
    WARN15 = 14,
    WARN16 = 15,
    WARN17 = 16,
    WARN18 = 17,
    WARN19 = 18,
    WARN20 = 19,
    WARN21 = 20,
    WARN22 = 21,
    WARN23 = 22,
    WARN24 = 23,
    WARN25 = 24,
    WARN26 = 25,
    WARN27 = 26,
    WARN28 = 27,
    WARN29 = 28,
    WARN30 = 29,
    WARN31 = 30,
    WARN32 = 31
);
status_group!(
    0x400,
    ALARM1 = 0,
    ALARM2 = 1,
    ALARM3 = 2,
    ALARM4 = 3,
    ALARM5 = 4,
    ALARM6 = 5,
    ALARM7 = 6,
    ALARM8 = 7,
    ALARM9 = 8,
    ALARM10 = 9,
    ALARM11 = 10,
    ALARM12 = 11,
    ALARM13 = 12,
    ALARM14 = 13,
    ALARM15 = 14,
    ALARM16 = 15,
    ALARM17 = 16,
    ALARM18 = 17,
    ALARM19 = 18,
    ALARM20 = 19,
    ALARM21 = 20,
    ALARM22 = 21,
    ALARM23 = 22,
    ALARM24 = 23,
    ALARM25 = 24,
    ALARM26 = 25,
    ALARM27 = 26,
    ALARM28 = 27,
    ALARM29 = 28,
    ALARM30 = 29,
    ALARM31 = 30,
    ALARM32 = 31
);
status_group!(
    0x800,
    FAIL1 = 0,
    FAIL2 = 1,
    FAIL3 = 2,
    FAIL4 = 3,
    FAIL5 = 4,
    FAIL6 = 5,
    FAIL7 = 6,
    FAIL8 = 7,
    FAIL9 = 8,
    FAIL10 = 9,
    FAIL11 = 10,
    FAIL12 = 11,
    FAIL13 = 12,
    FAIL14 = 13,
    FAIL15 = 14,
    FAIL16 = 15,
    FAIL17 = 16,
    FAIL18 = 17,
    FAIL19 = 18,
    FAIL20 = 19,
    FAIL21 = 20,
    FAIL22 = 21,
    FAIL23 = 22,
    FAIL24 = 23,
    FAIL25 = 24,
    FAIL26 = 25,
    FAIL27 = 26,
    FAIL28 = 27,
    FAIL29 = 28,
    FAIL30 = 29,
    FAIL31 = 30,
    FAIL32 = 31
);
