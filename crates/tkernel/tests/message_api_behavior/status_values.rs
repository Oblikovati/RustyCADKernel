//! Exhaustive values of the public Message status enums.
use rustycad_tkernel::message::{Status, StatusType};

const DONE: [Status; 32] = [
    Status::DONE1,
    Status::DONE2,
    Status::DONE3,
    Status::DONE4,
    Status::DONE5,
    Status::DONE6,
    Status::DONE7,
    Status::DONE8,
    Status::DONE9,
    Status::DONE10,
    Status::DONE11,
    Status::DONE12,
    Status::DONE13,
    Status::DONE14,
    Status::DONE15,
    Status::DONE16,
    Status::DONE17,
    Status::DONE18,
    Status::DONE19,
    Status::DONE20,
    Status::DONE21,
    Status::DONE22,
    Status::DONE23,
    Status::DONE24,
    Status::DONE25,
    Status::DONE26,
    Status::DONE27,
    Status::DONE28,
    Status::DONE29,
    Status::DONE30,
    Status::DONE31,
    Status::DONE32,
];
const WARN: [Status; 32] = [
    Status::WARN1,
    Status::WARN2,
    Status::WARN3,
    Status::WARN4,
    Status::WARN5,
    Status::WARN6,
    Status::WARN7,
    Status::WARN8,
    Status::WARN9,
    Status::WARN10,
    Status::WARN11,
    Status::WARN12,
    Status::WARN13,
    Status::WARN14,
    Status::WARN15,
    Status::WARN16,
    Status::WARN17,
    Status::WARN18,
    Status::WARN19,
    Status::WARN20,
    Status::WARN21,
    Status::WARN22,
    Status::WARN23,
    Status::WARN24,
    Status::WARN25,
    Status::WARN26,
    Status::WARN27,
    Status::WARN28,
    Status::WARN29,
    Status::WARN30,
    Status::WARN31,
    Status::WARN32,
];
const ALARM: [Status; 32] = [
    Status::ALARM1,
    Status::ALARM2,
    Status::ALARM3,
    Status::ALARM4,
    Status::ALARM5,
    Status::ALARM6,
    Status::ALARM7,
    Status::ALARM8,
    Status::ALARM9,
    Status::ALARM10,
    Status::ALARM11,
    Status::ALARM12,
    Status::ALARM13,
    Status::ALARM14,
    Status::ALARM15,
    Status::ALARM16,
    Status::ALARM17,
    Status::ALARM18,
    Status::ALARM19,
    Status::ALARM20,
    Status::ALARM21,
    Status::ALARM22,
    Status::ALARM23,
    Status::ALARM24,
    Status::ALARM25,
    Status::ALARM26,
    Status::ALARM27,
    Status::ALARM28,
    Status::ALARM29,
    Status::ALARM30,
    Status::ALARM31,
    Status::ALARM32,
];
const FAIL: [Status; 32] = [
    Status::FAIL1,
    Status::FAIL2,
    Status::FAIL3,
    Status::FAIL4,
    Status::FAIL5,
    Status::FAIL6,
    Status::FAIL7,
    Status::FAIL8,
    Status::FAIL9,
    Status::FAIL10,
    Status::FAIL11,
    Status::FAIL12,
    Status::FAIL13,
    Status::FAIL14,
    Status::FAIL15,
    Status::FAIL16,
    Status::FAIL17,
    Status::FAIL18,
    Status::FAIL19,
    Status::FAIL20,
    Status::FAIL21,
    Status::FAIL22,
    Status::FAIL23,
    Status::FAIL24,
    Status::FAIL25,
    Status::FAIL26,
    Status::FAIL27,
    Status::FAIL28,
    Status::FAIL29,
    Status::FAIL30,
    Status::FAIL31,
    Status::FAIL32,
];

fn assert_status_sequence(statuses: &[Status; 32], first: i32) {
    for (offset, status) in statuses.iter().enumerate() {
        assert_eq!(status.0, first + offset as i32);
    }
}

#[test]
fn done_status_values_match_occt_enum_sequence() {
    assert_status_sequence(&DONE, StatusType::DONE.0 as i32);
}

#[test]
fn warning_status_values_match_occt_enum_sequence() {
    assert_status_sequence(&WARN, StatusType::WARN.0 as i32);
}

#[test]
fn alarm_status_values_match_occt_enum_sequence() {
    assert_status_sequence(&ALARM, StatusType::ALARM.0 as i32);
}

#[test]
fn failure_status_values_match_occt_enum_sequence() {
    assert_status_sequence(&FAIL, StatusType::FAIL.0 as i32);
}

#[test]
fn none_and_status_type_values_match_occt_enums() {
    assert_eq!(Status::NONE.0, 0);
    assert_eq!(
        [
            StatusType::DONE.0,
            StatusType::WARN.0,
            StatusType::ALARM.0,
            StatusType::FAIL.0
        ],
        [0x100, 0x200, 0x400, 0x800]
    );
}
