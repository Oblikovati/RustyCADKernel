#include <Quantity_DateDefinitionError.hxx>
#include <Quantity_PeriodDefinitionError.hxx>
#include <Standard_DomainError.hxx>

#include <iostream>

template <typename ErrorType>
static void print_error(const char* message)
{
  try
  {
    throw ErrorType(message);
  }
  catch (const Standard_DomainError& error)
  {
    std::cout << error.ExceptionType() << ':' << error.what() << '\n';
  }
}

template <typename ErrorType>
static void print_default_error()
{
  try
  {
    throw ErrorType();
  }
  catch (const Standard_DomainError& error)
  {
    std::cout << error.ExceptionType() << ':' << error.what() << '\n';
  }
}

template <typename ErrorType>
static void print_stack_error(const char* message, const char* stack)
{
  try
  {
    throw ErrorType(message, stack);
  }
  catch (const Standard_DomainError& error)
  {
    std::cout << error.ExceptionType() << ':' << error.what() << ':'
              << error.GetStackString() << '\n';
  }
}

int main()
{
  print_error<Quantity_DateDefinitionError>("date parameters");
  print_error<Quantity_PeriodDefinitionError>("period values");
  print_default_error<Quantity_DateDefinitionError>();
  print_default_error<Quantity_PeriodDefinitionError>();
  print_stack_error<Quantity_DateDefinitionError>("date failure", "date trace");
  print_stack_error<Quantity_PeriodDefinitionError>("period failure", "period trace");
}
