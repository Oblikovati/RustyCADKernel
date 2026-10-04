//! Supplemental Linux pointer-text oracle; addresses are never dereferenced.
use rustycad_tkernel::standard::c_string::{FormatArg, sprintf_into_slice};

#[cfg(target_pointer_width = "64")]
#[test]
fn pointer_formats_match_linux_native_rows() {
    let rows = include_str!("../../../tools/oracle/tkernel_pointer_format.expected");
    assert_eq!(rows.lines().count(), 72);
    for row in rows.lines() {
        assert_pointer_row(row);
    }
}

#[cfg(target_pointer_width = "64")]
fn assert_pointer_row(row: &str) {
    let fields: Vec<&str> = row.split('|').collect();
    let address = usize::from_str_radix(fields[0], 16).unwrap();
    let pointer = std::ptr::without_provenance(address);
    let mut output = [0xa5_u8; 128];
    let count = sprintf_into_slice(
        &mut output,
        fields[1].as_bytes(),
        &[FormatArg::Pointer(pointer)],
    )
    .unwrap();
    assert_eq!(count, fields[2].parse::<usize>().unwrap(), "{row}");
    let hexadecimal: String = output[..count]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(hexadecimal, fields[3], "{row}");
    assert_eq!(output[count], 0);
    assert!(output[count + 1..].iter().all(|&byte| byte == 0xa5));
}

#[test]
fn pointer_formats_accept_live_addresses_and_reject_length_modifiers() {
    let value = 7_u8;
    let pointer = (&value as *const u8).cast();
    let mut output = [0xa5_u8; 128];
    let count = sprintf_into_slice(&mut output, b"%p", &[FormatArg::Pointer(pointer)]).unwrap();
    assert_eq!(
        &output[..count],
        format!("0x{:x}", pointer.addr()).as_bytes()
    );
    let before = output;
    assert!(sprintf_into_slice(&mut output, b"%lp", &[FormatArg::Pointer(pointer)]).is_err());
    assert!(sprintf_into_slice(&mut output, b"%p", &[FormatArg::Size(1)]).is_err());
    assert_eq!(output, before);
}
