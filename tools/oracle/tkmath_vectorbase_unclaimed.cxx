#include <math_Matrix.hxx>
#include <math_Vector.hxx>
#include <iostream>

static void dump(const math_Vector& value) {
  std::cout << value.Lower() << ':' << value.Upper() << '=';
  for (int index = value.Lower(); index <= value.Upper(); ++index) {
    std::cout << value(index) << ',';
  }
  std::cout << '\n';
}

static math_Vector left_vector() {
  math_Vector left(2, 4);
  left(2) = 1.25; left(3) = -2.0; left(4) = 4.0;
  return left;
}

static math_Vector right_vector() {
  math_Vector right(-1, 1);
  right(-1) = 2.0; right(0) = 3.0; right(1) = -1.0;
  return right;
}

static void exercise_copy_arithmetic(const math_Vector& left, const math_Vector& right) {
  math_Vector initialized(7, 9); initialized.Initialized(right); dump(initialized);
  math_Vector assigned(7, 9); assigned = left; dump(assigned);
  math_Vector add(left); add.Add(right); dump(add); dump(left.Added(right));
  math_Vector subtract(left); subtract.Subtract(right); dump(subtract); dump(left.Subtracted(right));
  math_Vector set(2, 4); set.Set(2, 4, right); dump(set);
  math_Vector sum(5, 7); sum.Add(left, right); dump(sum);
  math_Vector difference(-3, -1); difference.Subtract(left, right); dump(difference);
}

static math_Matrix product_matrix() {
  math_Matrix matrix(-1, 0, 5, 7);
  matrix(-1, 5) = 1; matrix(-1, 6) = 2; matrix(-1, 7) = 3;
  matrix(0, 5) = 4; matrix(0, 6) = 5; matrix(0, 7) = 6;
  return matrix;
}

static void exercise_matrix_products(const math_Vector& left, const math_Matrix& matrix) {
  math_Vector mv(10, 11); mv.Multiply(matrix, left); dump(mv);
  math_Vector row(5, 6); row(5) = -1; row(6) = 2;
  math_Vector vm(-4, -2); vm.Multiply(row, matrix); dump(vm);
  math_Vector tmv(4, 6); tmv.TMultiply(matrix, row); dump(tmv);
  math_Vector vtm(8, 9); vtm.TMultiply(left, matrix); dump(vtm);
}

int main() {
  const math_Vector left = left_vector();
  const math_Vector right = right_vector();
  exercise_copy_arithmetic(left, right);
  exercise_matrix_products(left, product_matrix());
  std::cout << left.Multiplied(right) << '\n';
}
