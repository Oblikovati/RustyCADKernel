#include <Standard_DomainError.hxx>
#include <Standard_OutOfMemory.hxx>
#include <iostream>

static void derivedTarget()
{
  const Standard_Failure source("new", "trace");
  Standard_DomainError target("old", "old trace");
  Standard_Failure& base = target;
  std::cout << (&(base = source) == &base) << ' '
            << (base.what() == source.what()) << ' '
            << (base.GetStackString() == source.GetStackString()) << '\n';
  std::cout << base << '\n';
}

static void bufferedTarget()
{
  const Standard_Failure source("base", "stack");
  Standard_OutOfMemory target("reserved");
  Standard_Failure& base = target;
  base = source;
  std::cout << base.what() << '\n' << base << '\n';
}

static void bufferedSource()
{
  const Standard_OutOfMemory source("reserved");
  Standard_DomainError target("old", "trace");
  Standard_Failure& base = target;
  base = source;
  std::cout << (base.what()[0] == '\0') << ' '
            << (base.GetStackString()[0] == '\0') << ' ' << source.what() << '\n';
  std::cout << base.ExceptionType() << '\n';
}

static void selfAssignment()
{
  Standard_OutOfMemory target("reserved");
  Standard_Failure& base = target;
  const char* aMessage = base.what();
  const char* aStack = base.GetStackString();
  std::cout << (&(base = base) == &base) << ' '
            << (base.what() == aMessage) << ' '
            << (base.GetStackString() == aStack) << ' '
            << base.ExceptionType() << ' ' << base.what() << '\n';
}

int main()
{
  derivedTarget();
  bufferedTarget();
  bufferedSource();
  selfAssignment();
}
