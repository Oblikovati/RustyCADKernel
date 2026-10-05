#include <Standard_DomainError.hxx>
#include <Standard_Failure.hxx>
#include <Standard_OutOfMemory.hxx>
#include <cstring>
#include <iostream>
#include <iomanip>
#include <sstream>

static void printBytes(const char* label, const std::string& bytes)
{
  std::cout << label << ':';
  for (unsigned char byte : bytes)
  {
    std::cout << ' ' << static_cast<unsigned int>(byte);
  }
  std::cout << '\n';
}

static void printCopies(const Standard_Failure& source)
{
  const Standard_Failure copy(source);
  Standard_DomainError target("old", "trace");
  static_cast<Standard_Failure&>(target) = source;
  std::cout << (copy.what() == source.what()) << ' '
            << (target.GetStackString() == source.GetStackString()) << ' '
            << target.ExceptionType() << '\n';
}

static void byteFields()
{
  const char message[] = {static_cast<char>(0xff), 'A', '\0', 'B'};
  const char stack[] = {static_cast<char>(0xfe), '\0', 'C'};
  const Standard_Failure source(message, stack);
  printBytes("message", source.what());
  printBytes("stack", source.GetStackString());
  std::ostringstream output;
  source.Print(output);
  printBytes("print", output.str());
  printCopies(source);
}

static void emptyFields()
{
  const Standard_Failure source(nullptr, nullptr);
  std::ostringstream output;
  source.Print(output);
  std::cout << std::strlen(source.what()) << ' ' << std::strlen(source.GetStackString())
            << ' ' << output.str() << '\n';
}

static void streamState()
{
  const char bytes[] = {static_cast<char>(0xff), '\0'};
  const Standard_Failure source(bytes, "");
  std::ostringstream padded;
  padded << std::setfill('_') << std::setw(20) << source;
  printBytes("padded", padded.str());
  std::ostringstream disabled;
  disabled.setstate(std::ios::badbit);
  source.Print(disabled);
  std::cout << "bad_state " << disabled.bad() << ' ' << disabled.str().empty() << '\n';
}

static void bufferedBytes()
{
  const std::string message(1030, static_cast<char>(0xff));
  Standard_OutOfMemory failure(message.c_str());
  std::cout << "oom_size " << std::strlen(failure.what()) << '\n';
  const char replacement[] = {static_cast<char>(0xfe), '\0', static_cast<char>(0xfd)};
  const Standard_OutOfMemory copy(failure);
  failure.SetMessageString(replacement);
  printBytes("oom_replaced", failure.what());
  std::cout << "oom_copy " << std::strlen(copy.what()) << ' '
            << static_cast<unsigned int>(static_cast<unsigned char>(copy.what()[0])) << '\n';
}

int main()
{
  byteFields();
  emptyFields();
  streamState();
  bufferedBytes();
}
