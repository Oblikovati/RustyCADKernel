// Default-argument constructor oracle for pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Standard_DomainError.hxx>
#include <Standard_RangeError.hxx>
#include <Standard_NotImplemented.hxx>
#include <iostream>
#include <type_traits>

template <class Exception, class Parent>
static void emit_default_exception()
{
  const Exception aFailure;
  std::cout << aFailure.ExceptionType() << '|'
            << aFailure.what() << '|'
            << std::is_base_of_v<Parent, Exception> << '|';
  aFailure.Print(std::cout);
  std::cout << '\n';
}

int main()
{
  emit_default_exception<Standard_DomainError, Standard_Failure>();
  emit_default_exception<Standard_RangeError, Standard_DomainError>();
  emit_default_exception<Standard_NotImplemented, Standard_ProgramError>();
}
