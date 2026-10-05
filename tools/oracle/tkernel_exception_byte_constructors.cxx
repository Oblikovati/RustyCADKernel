// Pinned OCCT macro exceptions must preserve arbitrary C-string bytes.
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

template<class Exception, class Parent>
void inspect()
{
  const char message[] = {char(0xff), 'x', 0, 'y', 0};
  const Exception error(message);
  const auto* bytes = reinterpret_cast<const unsigned char*>(error.what());
  std::cout << error.ExceptionType() << " " << std::hex << unsigned(bytes[0])
            << unsigned(bytes[1]) << " " << unsigned(bytes[2]);
  try { throw error; }
  catch (const Parent&) { std::cout << " parent=1\n"; }
}

int main()
{
  Standard_Failure::SetDefaultStackTraceLength(0);
  inspect<Standard_AbortiveTransaction, Standard_Failure>();
  inspect<Standard_ConstructionError, Standard_DomainError>();
  inspect<Standard_DimensionError, Standard_DomainError>();
  inspect<Standard_DimensionMismatch, Standard_DimensionError>();
  inspect<Standard_DivideByZero, Standard_NumericError>();
  inspect<Standard_DomainError, Standard_Failure>();
  inspect<Standard_ImmutableObject, Standard_DomainError>();
  inspect<Standard_LicenseError, Standard_Failure>();
  inspect<Standard_LicenseNotFound, Standard_LicenseError>();
  inspect<Standard_MultiplyDefined, Standard_DomainError>();
  inspect<Standard_NegativeValue, Standard_RangeError>();
  inspect<Standard_NoMoreObject, Standard_DomainError>();
  inspect<Standard_NoSuchObject, Standard_DomainError>();
  inspect<Standard_NotImplemented, Standard_ProgramError>();
  inspect<Standard_NullObject, Standard_DomainError>();
  inspect<Standard_NullValue, Standard_RangeError>();
  inspect<Standard_NumericError, Standard_Failure>();
  inspect<Standard_OutOfRange, Standard_RangeError>();
  inspect<Standard_Overflow, Standard_NumericError>();
  inspect<Standard_ProgramError, Standard_Failure>();
  inspect<Standard_RangeError, Standard_DomainError>();
  inspect<Standard_TypeMismatch, Standard_DomainError>();
  inspect<Standard_Underflow, Standard_NumericError>();
  inspect<StdFail_InfiniteSolutions, Standard_Failure>();
  inspect<StdFail_NotDone, Standard_Failure>();
  inspect<StdFail_Undefined, Standard_Failure>();
  inspect<StdFail_UndefinedDerivative, Standard_DomainError>();
  inspect<StdFail_UndefinedValue, Standard_DomainError>();
  inspect<Plugin_Failure, Standard_Failure>();
  inspect<Storage_StreamExtCharParityError, Storage_StreamReadError>();
  inspect<Storage_StreamFormatError, Standard_Failure>();
  inspect<Storage_StreamModeError, Standard_Failure>();
  inspect<Storage_StreamReadError, Standard_Failure>();
  inspect<Storage_StreamTypeMismatchError, Storage_StreamReadError>();
  inspect<Storage_StreamUnknownTypeError, Storage_StreamReadError>();
  inspect<Storage_StreamWriteError, Standard_Failure>();
  inspect<Units_NoSuchType, Standard_NoSuchObject>();
  inspect<Units_NoSuchUnit, Standard_NoSuchObject>();
}
