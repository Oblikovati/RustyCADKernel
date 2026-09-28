//! Additional contract tests for behavior absent from the translated legacy suites.
use rustycad_tkernel::{
    ErrorKind,
    quantity::{Date, Period},
    standard::{Guid, Uuid},
};

#[test]
fn period_construction_checks_combined_totals_as_occt_does() {
    assert!(!Period::is_valid_components(1, -1, 0, 0, 0, 0));
    assert_eq!(
        Period::from_components(1, -1, 0, 0, 0, 0).unwrap().values(),
        (82800, 0)
    );
    assert_eq!(
        Period::from_components(0, 0, 0, 0, -1, 1000)
            .unwrap()
            .values(),
        (0, 0)
    );
    let error = Period::new(-1, 0).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::PeriodDefinition);
    assert_eq!(
        error.message(),
        "Quantity_Period::SetValues invalid parameters"
    );
}

#[test]
fn rejected_setters_preserve_existing_state() {
    let mut period = Period::new(2, 500).unwrap();
    assert!(period.set_values(-1, 0).is_err());
    assert_eq!(period.values(), (2, 500));
    assert!(period.set_components(0, 0, 0, -1, 0, 0).is_err());
    assert_eq!(period.values(), (2, 500));
    let mut date = Date::new(2, 29, 2020, 12, 30, 0, 1, 2).unwrap();
    let original = date;
    assert_eq!(
        date.set_values(2, 29, 2021, 0, 0, 0, 0, 0)
            .unwrap_err()
            .kind(),
        ErrorKind::DateDefinition
    );
    assert_eq!(date, original);
}

#[test]
fn intervals_and_dates_preserve_microsecond_order_and_operators() {
    let a = Period::new(0, 1).unwrap();
    let b = Period::new(0, 2).unwrap();
    assert!(a < b);
    assert!(a.is_shorter(b));
    assert!(b.is_longer(a));
    assert_eq!((a + b).unwrap().values(), (0, 3));
    assert_eq!((a - b).unwrap().values(), (0, 1));
    let epoch = Date::default();
    let first = (epoch + a).unwrap();
    let second = (epoch + b).unwrap();
    assert!(first < second);
    assert!(first.is_earlier(second));
    assert!(second.is_later(first));
    assert_eq!(epoch.difference(first).values(), (0, 1));
    assert_eq!(first.difference(epoch).values(), (0, 1));
    assert_eq!((second - b).unwrap(), epoch);
    let error = (epoch - a).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::DateDefinition);
    assert_eq!(
        error.message(),
        "Quantity_Date::Subtract : The result date is anterior to Jan,1 1979"
    );
}

#[test]
fn undefined_cpp_overflow_is_a_typed_rust_error() {
    // These inputs are intentionally excluded from oracle comparisons: signed C++ overflow is UB.
    assert_eq!(
        Period::new(i32::MAX, 1_000_000).unwrap_err().kind(),
        ErrorKind::NumericOverflow
    );
    assert_eq!(
        (Period::new(i32::MAX, 0).unwrap() + Period::new(1, 0).unwrap())
            .unwrap_err()
            .kind(),
        ErrorKind::NumericOverflow
    );
    assert_eq!(
        Period::from_components(i32::MAX, 0, 0, 0, 0, 0)
            .unwrap_err()
            .kind(),
        ErrorKind::NumericOverflow
    );
    assert!(Date::is_valid(1, 1, i32::MAX, 0, 0, 0, 0, 0));
    assert_eq!(
        Date::new(1, 1, i32::MAX, 0, 0, 0, 0, 0).unwrap_err().kind(),
        ErrorKind::NumericOverflow
    );
}

#[test]
fn guid_encoding_assignment_and_c_string_contracts() {
    let guid: Guid = "12345678-1234-5678-9ABC-DEF012345678".parse().unwrap();
    assert_eq!(guid.to_string(), "12345678-1234-5678-9abc-def012345678");
    assert_eq!(
        &guid.to_c_string(),
        b"12345678-1234-5678-9abc-def012345678\0"
    );
    let utf16 = guid.to_ext_string();
    assert_eq!(utf16[36], 0);
    assert_eq!(Guid::from_utf16(&utf16).unwrap(), guid);
    assert!(Guid::from_utf16(&utf16[..35]).is_err());
    assert_eq!(Guid::from_utf16(&utf16.map(|v| v | 0x100)).unwrap(), guid);
    assert_eq!(
        guid.to_uuid(),
        Uuid {
            data1: 0x12345678,
            data2: 0x1234,
            data3: 0x5678,
            data4: [0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78]
        }
    );
    let mut copy = Guid::new();
    copy.assign(guid);
    assert_eq!(copy, guid);
    copy.assign_uuid(Guid::new().to_uuid());
    assert_eq!(copy, Guid::new());
    assert!(!Guid::check_format(None));
    assert!(Guid::check_format(Some(
        b"12345678-1234-5678-9abc-def012345678\0trailing"
    )));
    assert!(!Guid::check_format(Some(
        b"12345678-1234-5678-9abc-def012345678trailing"
    )));
    let mut output = String::new();
    guid.shallow_dump(&mut output).unwrap();
    assert_eq!(output, "12345678-1234-5678-9abc-def012345678");
    assert_eq!("bad".parse::<Guid>().unwrap_err().kind(), ErrorKind::Range);
}
