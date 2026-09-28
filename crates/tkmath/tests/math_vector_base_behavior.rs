//! math_IntegerVector semantics, dumps, and the remaining math_Matrix members.
use rustycad_tkmath::gp::Xy;
use rustycad_tkmath::math::{
    IntegerVector, Matrix, Vector, VectorItem, not_square, singular_matrix,
};

fn integers(lower: i32, items: &[i32]) -> IntegerVector<'static> {
    let mut vector = IntegerVector::with_bounds(lower, lower + items.len() as i32 - 1).unwrap();
    for (offset, item) in items.iter().enumerate() {
        *vector.change_at(offset).unwrap() = *item;
    }
    vector
}
fn items(vector: &IntegerVector<'_>) -> Vec<i32> {
    (vector.lower()..=vector.upper())
        .map(|i| vector[i])
        .collect()
}
fn matrix(lower: [i32; 2], rows: &[&[f64]]) -> Matrix<'static> {
    let (r, c) = (rows.len() as i32, rows[0].len() as i32);
    let mut result =
        Matrix::with_bounds(lower[0], lower[0] + r - 1, lower[1], lower[1] + c - 1).unwrap();
    for (i, row) in rows.iter().enumerate() {
        for (j, value) in row.iter().enumerate() {
            *result.change_at(i, j).unwrap() = *value;
        }
    }
    result
}

#[test]
fn integer_items_convert_like_cxx() {
    assert_eq!(<i32 as VectorItem>::from_real(-2.7), -2);
    assert_eq!(<i32 as VectorItem>::from_real(2.7), 2);
    assert_eq!(3_i32.to_real(), 3.0);
    assert_eq!(7_i32.item_div(-2), -3);
    assert_eq!(i32::MAX.item_add(1), i32::MIN);
    assert_eq!(1.5_f64.stream_text(), "1.5");
    assert_eq!(1234567.0_f64.stream_text(), "1.23457e+06");
    assert_eq!((-4_i32).stream_text(), "-4");
}
#[test]
fn integer_divided_multiplies_by_a_truncated_reciprocal() {
    let vector = integers(1, &[7, -7, 3]);
    assert_eq!(items(&vector.divided(2).unwrap()), [0, 0, 0]);
    assert_eq!(items(&vector.divided(-1).unwrap()), [-7, 7, -3]);
    let mut divided = vector.clone();
    divided.divide(2).unwrap();
    assert_eq!(items(&divided), [3, -3, 1]);
    assert_eq!(
        vector.divided(0).unwrap_err().exception_type(),
        "Standard_DivideByZero"
    );
}
#[test]
fn integer_normalize_and_products_truncate_each_double_result() {
    let mut vector = integers(1, &[3, 4]);
    assert_eq!(vector.norm().unwrap(), 5.0);
    vector.normalize().unwrap();
    assert_eq!(items(&vector), [0, 0]);
    let halves = matrix([1, 1], &[&[0.5, 0.5]]);
    let mut product = IntegerVector::with_bounds(1, 1).unwrap();
    product
        .set_matrix_vector(&halves, &integers(1, &[1, 1]))
        .unwrap();
    assert_eq!(product[1], 0, "every partial sum 0.5 truncates to 0");
    let mut real = Vector::with_bounds(1, 1).unwrap();
    real.set_matrix_vector(&halves, &Vector::filled(1, 2, 1.0).unwrap())
        .unwrap();
    assert_eq!(real[1], 1.0);
}
#[test]
fn integer_extrema_dot_and_scalar_forms() {
    let vector = integers(-1, &[2, -5, 9, 9]);
    assert_eq!((vector.max_index(), vector.min_index()), (1, 0));
    assert_eq!(vector.dot(&vector).unwrap(), 191);
    assert_eq!(items(&vector.t_multiplied(2)), [4, -10, 18, 18]);
    assert_eq!(items(&(3 * &vector)), [6, -15, 27, 27]);
    assert_eq!(items(&IntegerVector::from_xy(Xy::new(2.9, -2.9))), [2, -2]);
    assert_eq!(items(&IntegerVector::with_size_filled(2, 4)), [4, 4]);
}
#[test]
fn dump_and_array_view_follow_the_template() {
    let vector = integers(-1, &[3, -4]);
    assert_eq!(
        vector.to_string(),
        "math_Vector of Length = 2\nmath_Vector(-1) = 3\nmath_Vector(0) = -4\n"
    );
    let reals = Vector::filled(2, 3, 0.1).unwrap();
    assert_eq!(
        format!("{reals}"),
        "math_Vector of Length = 2\nmath_Vector(2) = 0.1\nmath_Vector(3) = 0.1\n"
    );
    let array = vector.array1();
    assert_eq!((array.lower(), array.upper()), (-1, 0));
    assert_eq!(vector.size(), 2);
}
#[test]
fn matrix_dump_and_scalar_operators() {
    let m = matrix([1, 3], &[&[1.0, -2.5]]);
    assert_eq!(
        m.to_string(),
        "math_Matrix of RowNumber = 1 and ColNumber = 2\n\
         math_Matrix ( 1, 3 ) = 1\nmath_Matrix ( 1, 4 ) = -2.5\n"
    );
    assert_eq!((-&m)[(1, 4)], 2.5);
    assert_eq!((2.0 * &m)[(1, 4)], -5.0);
    assert_eq!(m.t_multiplied(2.0)[(1, 3)], 2.0);
    assert_eq!((&m / 2.0)[(1, 3)], 0.5);
    assert_eq!((&m + &m)[(1, 3)], 2.0);
    assert_eq!((&m - &m)[(1, 3)], 0.0);
    assert_eq!((m.row_size(), m.col_size()), (1, 2));
    assert_eq!(Matrix::with_size_filled(2, 3, 1.5)[(1, 2)], 1.5);
}
#[test]
fn matrix_in_place_and_two_operand_sums_use_positions() {
    let m = matrix([1, 1], &[&[1.0, 2.0], &[3.0, 4.0]]);
    let other = matrix([5, -3], &[&[10.0, 20.0], &[30.0, 40.0]]);
    let mut copy = m.clone();
    copy.add(&other).unwrap();
    assert_eq!(copy[(2, 2)], 44.0);
    copy.subtract(&other).unwrap();
    assert_eq!(copy[(2, 2)], 4.0);
    let mut target = Matrix::with_bounds(0, 1, 0, 1).unwrap();
    target.set_sum(&m, &other).unwrap();
    assert_eq!(target[(0, 1)], 22.0);
    target.set_difference(&other, &m).unwrap();
    assert_eq!(target[(1, 0)], 27.0);
    let wide = Matrix::with_bounds(1, 1, 1, 4).unwrap();
    assert_eq!(
        copy.add(&wide).unwrap_err().exception_type(),
        "Standard_DimensionError"
    );
}
#[test]
fn matrix_products_keep_native_bounds() {
    let m = matrix([1, 1], &[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]);
    let gram = m.t_multiply(&m).unwrap();
    assert_eq!((gram.lower_row(), gram.upper_col()), (1, 3));
    assert_eq!(gram[(3, 3)], 45.0);
    let mut outer = Matrix::with_bounds(-1, 0, 4, 5).unwrap();
    outer.set_product(&m, &m.transposed()).unwrap();
    assert_eq!(outer[(0, 5)], 77.0);
    let mut t_product = Matrix::with_bounds(0, 2, 0, 2).unwrap();
    t_product.set_transpose_product(&m, &m).unwrap();
    assert_eq!(t_product[(0, 0)], 17.0);
    assert!(outer.set_product(&m, &m).is_err());
    let mut square = matrix([1, 1], &[&[1.0, 2.0], &[3.0, 4.0]]);
    square.square().unwrap();
    assert_eq!(square[(1, 1)], 7.0);
}
#[test]
fn matrix_block_set_writes_until_out_of_range() {
    let mut m = Matrix::filled(1, 2, 1, 2, 0.0).unwrap();
    let block = Matrix::filled(1, 2, 1, 1, 5.0).unwrap();
    assert_eq!(
        m.set(2, 3, 2, 2, &block).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        m[(2, 2)],
        5.0,
        "the first in-range cell was already written"
    );
    assert_eq!(
        m.set(1, 1, 1, 1, &block).unwrap_err().exception_type(),
        "Standard_DimensionError"
    );
}
#[test]
fn in_place_transpose_depends_on_bound_order() {
    let mut aligned = matrix([2, 2], &[&[1.0, 2.0], &[3.0, 4.0]]);
    aligned.transpose().unwrap();
    assert_eq!(aligned[(2, 3)], 3.0);
    let mut low_cols = matrix([3, 1], &[&[1.0, 2.0], &[3.0, 4.0]]);
    low_cols.transpose().unwrap();
    assert_eq!((low_cols.lower_row(), low_cols.lower_col()), (1, 3));
    assert_eq!(
        low_cols[(1, 4)],
        2.0,
        "no swap happens when columns start lower"
    );
    let mut high_cols = matrix([1, 3], &[&[1.0, 2.0], &[3.0, 4.0]]);
    assert_eq!(
        high_cols.transpose().unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        high_cols.lower_col(),
        1,
        "bounds were rebased before the failure"
    );
    let mut wide = Matrix::with_bounds(1, 1, 1, 2).unwrap();
    assert_eq!(
        wide.transpose().unwrap_err().exception_type(),
        "math_NotSquare"
    );
}
#[test]
fn borrowed_matrix_writes_through_and_exceptions_carry_type_names() {
    let mut cells = [0.0; 4];
    {
        let mut m = Matrix::from_buffer(&mut cells, 1, 2, 1, 2).unwrap();
        m.set_diag(3.0).unwrap();
    }
    assert_eq!(cells, [3.0, 0.0, 0.0, 3.0]);
    assert_eq!(not_square("x").exception_type(), "math_NotSquare");
    assert_eq!(singular_matrix("").exception_type(), "math_SingularMatrix");
}
