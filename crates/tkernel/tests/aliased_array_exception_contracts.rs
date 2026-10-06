//! Native exception names, exact payloads, validation precedence and output preservation.
use rustycad_tkernel::ncollection::AliasedArray;
#[path = "aliased_array_exceptions/support.rs"]
mod support;
use support::record;
fn constructor_rows(rows: &mut Vec<String>) {
    record(rows, "stride_zero", AliasedArray::empty(0));
    record(rows, "size_zero", AliasedArray::new(4, 0));
    record(rows, "both_zero", AliasedArray::new(0, 0));
    record(rows, "signed_both_zero", AliasedArray::new_signed(0, 0));
    record(rows, "signed_negative", AliasedArray::new_signed(4, -1));
    record(
        rows,
        "borrowed_zero",
        AliasedArray::from_slice(&mut [] as &mut [i32]),
    );
}
fn shape_rows(rows: &mut Vec<String>, array: &mut AliasedArray<'_>) {
    record(rows, "resize_zero", array.resize(0, true));
    record(rows, "resize_signed_zero", array.resize_signed(0, false));
    record(
        rows,
        "assign_stride",
        array.assign(&AliasedArray::new(8, 2).unwrap()),
    );
    record(
        rows,
        "assign_count",
        array.assign(&AliasedArray::new(4, 3).unwrap()),
    );
}
fn pointer_rows(rows: &mut Vec<String>, array: &mut AliasedArray<'_>) {
    record(rows, "data_type", array.data_ptr::<f64>());
    record(rows, "change_data_type", array.data_mut_ptr::<f64>());
    record(rows, "init_type", array.init(1_f64));
    record(rows, "raw_signed", array.byte_value_ptr_signed(-1));
    record(rows, "raw_unsigned", array.byte_value_ptr(2));
    record(
        rows,
        "raw_mut_signed",
        array.change_byte_value_ptr_signed(-1),
    );
    record(rows, "raw_mut_unsigned", array.change_byte_value_ptr(2));
}
macro_rules! typed_rows {
    ($rows:expr, $array:expr, $name:literal, $unsigned:ident, $signed:ident) => {{
        record(
            $rows,
            concat!($name, "_negative_type"),
            $array.$signed::<f64>(-1),
        );
        record(
            $rows,
            concat!($name, "_positive_type"),
            $array.$signed::<f64>(2),
        );
        record(
            $rows,
            concat!($name, "_negative"),
            $array.$signed::<i32>(-1),
        );
        record($rows, concat!($name, "_positive"), $array.$signed::<i32>(2));
        record(
            $rows,
            concat!($name, "_unsigned_type"),
            $array.$unsigned::<f64>(2),
        );
        record(
            $rows,
            concat!($name, "_unsigned"),
            $array.$unsigned::<i32>(2),
        );
    }};
}
fn output_exact_rows(
    rows: &mut Vec<String>,
    array: &AliasedArray<'_>,
    wide: &mut f64,
    narrow: &mut i32,
) {
    let negative = array.value_into_signed(-1, wide);
    record(rows, "output_negative_type", negative);
    let positive = array.value_into_signed(2, wide);
    record(rows, "output_positive_type", positive);
    record(rows, "output_unsigned", array.value_into(2, narrow));
}
fn output_prefix_rows(
    rows: &mut Vec<String>,
    array: &AliasedArray<'_>,
    wide: &mut f64,
    narrow: &mut i32,
) {
    let negative = array.value2_into_signed(-1, wide);
    record(rows, "output2_negative_type", negative);
    let positive = array.value2_into_signed(2, wide);
    record(rows, "output2_positive_type", positive);
    record(rows, "output2_unsigned", array.value2_into(2, narrow));
}
fn output_rows(rows: &mut Vec<String>, array: &AliasedArray<'_>) {
    let (mut wide, mut narrow) = (91_f64, 73_i32);
    output_exact_rows(rows, array, &mut wide, &mut narrow);
    output_prefix_rows(rows, array, &mut wide, &mut narrow);
    rows.push(format!("outputs_unchanged|{wide}|{narrow}"));
}
fn readonly_typed_rows(rows: &mut Vec<String>, array: &AliasedArray<'_>) {
    typed_rows!(rows, array, "Value", value, value_signed);
    typed_rows!(rows, array, "Value2", value2, value2_signed);
}
fn writable_typed_rows(rows: &mut Vec<String>, array: &mut AliasedArray<'_>) {
    typed_rows!(
        rows,
        array,
        "ChangeValue",
        change_value,
        change_value_signed
    );
    typed_rows!(
        rows,
        array,
        "ChangeValue2",
        change_value2,
        change_value2_signed
    );
}
#[test]
fn exact_native_error_payloads_and_overload_precedence() {
    let mut rows = Vec::new();
    constructor_rows(&mut rows);
    let mut array = AliasedArray::new(4, 2).unwrap();
    shape_rows(&mut rows, &mut array);
    pointer_rows(&mut rows, &mut array);
    readonly_typed_rows(&mut rows, &array);
    writable_typed_rows(&mut rows, &mut array);
    output_rows(&mut rows, &array);
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../../tools/oracle/tkernel_aliased_array_exceptions.expected")
    );
}
