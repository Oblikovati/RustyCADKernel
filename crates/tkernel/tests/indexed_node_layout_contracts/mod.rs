//! Matching C++ and Rust payload layout is a precondition, not a universal generic ABI promise.
#[repr(C, align(32))]
#[derive(Default)]
pub struct Wide {
    value: u8,
}

pub fn native_layout_columns(label: &str, columns: &[usize]) {
    let cells: Vec<String> = columns.iter().map(usize::to_string).collect();
    let actual = format!("{label} {}", cells.join(" "));
    let expected = include_str!("../../../../tools/oracle/tkernel_indexed_node_layout.expected");
    let native = expected
        .lines()
        .find(|row| row.starts_with(&format!("{label} ")))
        .unwrap();
    assert_eq!(actual, native);
}
