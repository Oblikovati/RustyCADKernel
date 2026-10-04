#include <math_Householder.hxx>
#include <iostream>
class ExposedHouseholder : public math_Householder {
public:
  using math_Householder::math_Householder;
  void Reperform(const math_Matrix& a, const math_Matrix& b, double eps) {
    Perform(a, b, eps);
  }
};
void print(const ExposedHouseholder& solver) {
  std::cout << solver.IsDone() << ' ';
  try { std::cout << solver.AllValues()(1, 1) << ' ' << solver.AllValues()(2, 1); }
  catch (const Standard_Failure& failure) { std::cout << failure.ExceptionType(); }
  std::cout << '\n';
}
int main() {
  math_Matrix identity(1, 2, 1, 2), rhs(1, 2, 1, 1), zero(1, 2, 1, 2);
  identity.Init(0.0); identity(1, 1) = 1.0; identity(2, 2) = 1.0;
  rhs(1, 1) = 1.0; rhs(2, 1) = 2.0; zero.Init(0.0);
  ExposedHouseholder solver(identity, rhs);
  print(solver); solver.Reperform(zero, rhs, 1e-20); print(solver);
  rhs(1, 1) = 3.0; rhs(2, 1) = 4.0;
  solver.Reperform(identity, rhs, 1e-20); print(solver);
}
