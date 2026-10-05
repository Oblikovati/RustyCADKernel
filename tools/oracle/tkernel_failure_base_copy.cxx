#include <Standard_DomainError.hxx>
#include <Standard_OutOfMemory.hxx>
#include <iostream>

static void derivedSource()
{
  const Standard_DomainError source("message", "stack");
  const Standard_Failure copy(source);
  std::cout << copy << '\n' << source.ExceptionType() << '\n';
  std::cout << (copy.what() == source.what()) << ' '
            << (copy.GetStackString() == source.GetStackString()) << '\n';
}

static void bufferedSource()
{
  Standard_OutOfMemory source("private");
  static_cast<Standard_Failure&>(source) = Standard_Failure("", "trace");
  const Standard_Failure copy(source);
  std::cout << copy.ExceptionType() << ' ' << (copy.what()[0] == '\0') << ' '
            << (copy.GetStackString() == source.GetStackString()) << '\n';
  std::cout << source.what() << ' ' << copy.GetStackString() << '\n';
}

static void byteSource()
{
  const char bytes[] = {static_cast<char>(0xff), '\0'};
  const Standard_Failure source(bytes, bytes);
  const Standard_Failure copy(source);
  std::cout << static_cast<unsigned int>(static_cast<unsigned char>(copy.what()[0]))
            << ' ' << (copy.what() == source.what()) << '\n';
}

int main()
{
  derivedSource();
  bufferedSource();
  byteSource();
}
