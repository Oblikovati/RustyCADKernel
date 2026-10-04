#include <Geom2d_UndefinedDerivative.hxx>
#include <Geom2d_UndefinedValue.hxx>
#include <Geom_UndefinedDerivative.hxx>
#include <Geom_UndefinedValue.hxx>
#include <iostream>

template<class Exception>
void print_type() {
  Exception original("geometry probe");
  Exception copied(original);
  const Standard_Failure& base = copied;
  std::cout << original.ExceptionType() << ' ' << base.ExceptionType() << '\n';
}

int main() {
  print_type<Geom2d_UndefinedDerivative>();
  print_type<Geom2d_UndefinedValue>();
  print_type<Geom_UndefinedDerivative>();
  print_type<Geom_UndefinedValue>();
}
