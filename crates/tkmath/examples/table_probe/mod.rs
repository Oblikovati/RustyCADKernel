//! Native table storage, indexing, rebasing, copy and move comparison.
use rustycad_tkmath::math::DoubleTab;
fn table_values(table: &DoubleTab<'_>) {
    print!(
        "{} {} {} {} {} {} {} ",
        table.lower_row(),
        table.upper_row(),
        table.lower_col(),
        table.upper_col(),
        table.nb_rows(),
        table.nb_columns(),
        i32::from(table.is_deletable())
    );
    for row in 0..table.row_size() {
        for col in 0..table.col_size() {
            print!("{} ", table.at(row, col).unwrap().to_bits());
        }
    }
}
pub(super) fn table_case(v: &[f64]) {
    let (rows, cols, rl, cl) = (v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32);
    let mut original = DoubleTab::with_bounds(rl, rl + rows - 1, cl, cl + cols - 1).unwrap();
    original.init(v[4]);
    *original
        .change_at(rows as usize - 1, cols as usize - 1)
        .unwrap() = -v[4];
    table_values(&original);
    let copy = original.clone();
    table_values(&copy);
    let mut moved = DoubleTab::move_construct(&mut original);
    table_values(&moved);
    table_values(&original);
    moved.set_lower_row(cl);
    moved.set_lower_col(rl);
    table_values(&moved);
    let mut destination = DoubleTab::with_bounds(cl, cl + rows - 1, rl, rl + cols - 1).unwrap();
    destination.move_assign(&mut moved);
    table_values(&destination);
    table_values(&moved);
    external_values(&copy, rows, cols, rl, cl, v[4]);
    copy.copy_to(&mut destination).unwrap();
    table_values(&destination);
    println!();
}
fn external_values(copy: &DoubleTab<'_>, rows: i32, cols: i32, rl: i32, cl: i32, value: f64) {
    let mut external = vec![42.0; (rows * cols) as usize];
    let mut borrowed =
        DoubleTab::from_buffer(&mut external, rl, rl + rows - 1, cl, cl + cols - 1).unwrap();
    borrowed.assign(copy);
    table_values(&borrowed);
    *borrowed.change_value(rl, cl).unwrap() = value + 1.0;
    // The separate write-through regression checks the external slice after its borrow ends.
    print!("{} ", borrowed.at(0, 0).unwrap().to_bits());
    let moved = DoubleTab::move_construct(&mut borrowed);
    table_values(&moved);
    table_values(&borrowed);
}
