// Explicit stack constructors preserve both native byte strings.
#include <iostream>
#include <Standard_AbortiveTransaction.hxx>
#include <Standard_ConstructionError.hxx>
#include <Standard_DimensionError.hxx>
#include <Standard_DimensionMismatch.hxx>
#include <Standard_DivideByZero.hxx>
#include <Standard_DomainError.hxx>
#include <Standard_ImmutableObject.hxx>
#include <Standard_LicenseError.hxx>
#include <Standard_LicenseNotFound.hxx>
#include <Standard_MultiplyDefined.hxx>
#include <Standard_NegativeValue.hxx>
#include <Standard_NoMoreObject.hxx>
#include <Standard_NoSuchObject.hxx>
#include <Standard_NotImplemented.hxx>
#include <Standard_NullObject.hxx>
#include <Standard_NullValue.hxx>
#include <Standard_NumericError.hxx>
#include <Standard_OutOfRange.hxx>
#include <Standard_Overflow.hxx>
#include <Standard_ProgramError.hxx>
#include <Standard_RangeError.hxx>
#include <Standard_TypeMismatch.hxx>
#include <Standard_Underflow.hxx>
#include <StdFail_InfiniteSolutions.hxx>
#include <StdFail_NotDone.hxx>
#include <StdFail_Undefined.hxx>
#include <StdFail_UndefinedDerivative.hxx>
#include <StdFail_UndefinedValue.hxx>
#include <Plugin_Failure.hxx>
#include <Storage_StreamExtCharParityError.hxx>
#include <Storage_StreamFormatError.hxx>
#include <Storage_StreamModeError.hxx>
#include <Storage_StreamReadError.hxx>
#include <Storage_StreamTypeMismatchError.hxx>
#include <Storage_StreamUnknownTypeError.hxx>
#include <Storage_StreamWriteError.hxx>
#include <Units_NoSuchType.hxx>
#include <Units_NoSuchUnit.hxx>

template<class Exception>
void inspect()
{
  const char message[] = {char(0xff), 'm', 0, 'x', 0};
  const char stack[] = {char(0xfe), 's', 0, 'x', 0};
  const Exception error(message, stack);
  const auto* bytes = reinterpret_cast<const unsigned char*>(error.what());
  const auto* trace = reinterpret_cast<const unsigned char*>(error.GetStackString());
  const Exception empty(nullptr, nullptr);
  const bool emptyStack = empty.GetStackString() == nullptr || *empty.GetStackString() == 0;
  std::cout << error.ExceptionType() << " " << std::hex << unsigned(bytes[0])
            << unsigned(bytes[1]) << " " << unsigned(trace[0]) << unsigned(trace[1])
            << " empty=" << (*empty.what() == 0 && emptyStack) << "\n";
}

int main()
{
  inspect<Standard_AbortiveTransaction>();
  inspect<Standard_ConstructionError>();
  inspect<Standard_DimensionError>();
  inspect<Standard_DimensionMismatch>();
  inspect<Standard_DivideByZero>();
  inspect<Standard_DomainError>();
  inspect<Standard_ImmutableObject>();
  inspect<Standard_LicenseError>();
  inspect<Standard_LicenseNotFound>();
  inspect<Standard_MultiplyDefined>();
  inspect<Standard_NegativeValue>();
  inspect<Standard_NoMoreObject>();
  inspect<Standard_NoSuchObject>();
  inspect<Standard_NotImplemented>();
  inspect<Standard_NullObject>();
  inspect<Standard_NullValue>();
  inspect<Standard_NumericError>();
  inspect<Standard_OutOfRange>();
  inspect<Standard_Overflow>();
  inspect<Standard_ProgramError>();
  inspect<Standard_RangeError>();
  inspect<Standard_TypeMismatch>();
  inspect<Standard_Underflow>();
  inspect<StdFail_InfiniteSolutions>();
  inspect<StdFail_NotDone>();
  inspect<StdFail_Undefined>();
  inspect<StdFail_UndefinedDerivative>();
  inspect<StdFail_UndefinedValue>();
  inspect<Plugin_Failure>();
  inspect<Storage_StreamExtCharParityError>();
  inspect<Storage_StreamFormatError>();
  inspect<Storage_StreamModeError>();
  inspect<Storage_StreamReadError>();
  inspect<Storage_StreamTypeMismatchError>();
  inspect<Storage_StreamUnknownTypeError>();
  inspect<Storage_StreamWriteError>();
  inspect<Units_NoSuchType>();
  inspect<Units_NoSuchUnit>();
}
