// Pinned OCCT declaration/catch oracle; no production native dependency.
#include <Aspect_AspectFillAreaDefinitionError.hxx>
#include <Aspect_AspectLineDefinitionError.hxx>
#include <Aspect_AspectMarkerDefinitionError.hxx>
#include <Aspect_DisplayConnectionDefinitionError.hxx>
#include <Aspect_GraphicDeviceDefinitionError.hxx>
#include <Aspect_IdentDefinitionError.hxx>
#include <Aspect_WindowDefinitionError.hxx>
#include <Aspect_WindowError.hxx>
#include <Standard_RangeError.hxx>
#include <Standard_DomainError.hxx>
#include <Standard_ProgramError.hxx>
#include <iostream>
#include <cstring>
template<class Base, class Exception> int catches(const Exception& failure) {
  try { throw failure; }
  catch (const Base&) { return 1; }
  catch (...) { return 0; }
}
template<class Exception> void print(const Exception& failure, const char* mode) {
  std::cout << failure.ExceptionType() << "|" << mode << "|"
    << catches<Standard_OutOfRange>(failure)
    << catches<Standard_RangeError>(failure)
    << catches<Standard_DomainError>(failure)
    << catches<Standard_Failure>(failure)
    << catches<Standard_ProgramError>(failure) << "|"
    << failure.what() << "|" << failure.GetStackString() << "\n";
}
template<class Exception> void print_bytes() {
  const char message[] = {char(255), 0, 'a', 0};
  const char stack[] = {char(254), 0, 'b', 0};
  Exception failure(message, stack);
  std::cout << failure.ExceptionType() << "|bytes|"
    << catches<Standard_OutOfRange>(failure)
    << catches<Standard_RangeError>(failure)
    << catches<Standard_DomainError>(failure)
    << catches<Standard_Failure>(failure)
    << catches<Standard_ProgramError>(failure) << "|"
    << int(static_cast<unsigned char>(failure.what()[0])) << "|"
    << int(static_cast<unsigned char>(failure.GetStackString()[0])) << "|"
    << std::strlen(failure.what()) << "|" << std::strlen(failure.GetStackString()) << "\n";
}
template<class Exception> void record() {
  print(Exception(), "default");
  print(Exception("why"), "message");
  print(Exception("why", "trace"), "explicit-stack");
  print_bytes<Exception>();
}
int main() {
  Standard_Failure::SetDefaultStackTraceLength(0);
  record<Aspect_AspectFillAreaDefinitionError>();
  record<Aspect_AspectLineDefinitionError>();
  record<Aspect_AspectMarkerDefinitionError>();
  record<Aspect_DisplayConnectionDefinitionError>();
  record<Aspect_GraphicDeviceDefinitionError>();
  record<Aspect_IdentDefinitionError>();
  record<Aspect_WindowDefinitionError>();
  record<Aspect_WindowError>();
}
