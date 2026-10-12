//! One typed row formatter keeps bounded native replay helpers independent of formatting details.
use std::fmt::Write;
pub(super) fn shape_drawer_row(output: &mut String, label: &str, values: &[i32]) {
    write!(output, "{label}").unwrap();
    for value in values {
        write!(output, " {value}").unwrap();
    }
    writeln!(output).unwrap();
}
