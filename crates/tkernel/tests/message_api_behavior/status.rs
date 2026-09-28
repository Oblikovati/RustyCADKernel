//! Message_ExecStatus and the status enumerations.
use rustycad_tkernel::message::{ExecStatus, Status, StatusType};

#[test]
fn flags_are_set_tested_and_cleared_per_group() {
    let mut status = ExecStatus::from_status(Status::WARN3);
    assert!(status.is_warn() && !status.is_done() && !status.is_alarm() && !status.is_fail());
    assert!(status.is_set(Status::WARN3) && !status.is_set(Status::WARN4));
    status.set(Status::FAIL32);
    assert!(status.is_fail() && status.is_set(Status::FAIL32));
    status.clear_status(Status::WARN3);
    assert!(!status.is_warn());
    status.set(Status::NONE);
    assert_eq!(status, ExecStatus::from_status(Status::FAIL32));
    status.clear();
    assert_eq!(status, ExecStatus::new());
}

#[test]
fn whole_groups_and_set_algebra() {
    let mut status = ExecStatus::new();
    status.set_all_done();
    status.set_all_warn();
    status.set_all_alarm();
    status.set_all_fail();
    assert!(status.is_set(Status::DONE17) && status.is_set(Status::ALARM32));
    status.clear_all_done();
    status.clear_all_warn();
    status.clear_all_alarm();
    status.clear_all_fail();
    assert!(!status.is_done() && !status.is_warn() && !status.is_alarm() && !status.is_fail());
    let mut union = ExecStatus::from_status(Status::DONE1);
    union.add(&ExecStatus::from_status(Status::WARN2));
    union |= &ExecStatus::from_status(Status::ALARM3);
    assert!(union.is_done() && union.is_warn() && union.is_alarm());
    union.and(&ExecStatus::from_status(Status::WARN2));
    assert!(union.is_warn() && !union.is_done());
    union &= &ExecStatus::from_status(Status::DONE1);
    assert_eq!(union, ExecStatus::new());
}

#[test]
fn indices_types_and_the_out_of_enum_index_zero() {
    assert_eq!(ExecStatus::status_index(Status::DONE1), 1);
    assert_eq!(ExecStatus::status_index(Status::WARN1), 33);
    assert_eq!(ExecStatus::status_index(Status::FAIL32), 128);
    assert_eq!(ExecStatus::status_index(Status::NONE), 0);
    assert_eq!(ExecStatus::local_status_index(Status::ALARM5), 5);
    assert_eq!(
        ExecStatus::type_of_status(Status::ALARM5),
        StatusType::ALARM
    );
    assert_eq!(ExecStatus::status_by_index(33), Status::WARN1);
    assert_eq!(ExecStatus::status_by_index(128), Status::FAIL32);
    assert_eq!(ExecStatus::status_by_index(129), Status::NONE);
    // Native quirk: index 0 is Message_DONE - 1, a status outside the enumerators.
    assert_eq!(ExecStatus::status_by_index(0), Status(0xFF));
    assert_eq!(
        (
            ExecStatus::FIRST_STATUS,
            ExecStatus::STATUSES_PER_TYPE,
            ExecStatus::NB_STATUSES,
            ExecStatus::LAST_STATUS
        ),
        (1, 32, 128, 129)
    );
}
