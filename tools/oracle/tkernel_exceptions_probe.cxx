// Exception class tree, type names and printing of the TKernel DEFINE_STANDARD_EXCEPTION classes.
#include "tkernel_api_common.hxx"
#include <Plugin_Failure.hxx>
#include <StdFail_InfiniteSolutions.hxx>
#include <StdFail_NotDone.hxx>
#include <StdFail_Undefined.hxx>
#include <StdFail_UndefinedDerivative.hxx>
#include <StdFail_UndefinedValue.hxx>
#include <Standard_AbortiveTransaction.hxx>
#include <Standard_ConstructionError.hxx>
#include <Standard_DimensionMismatch.hxx>
#include <Standard_DivideByZero.hxx>
#include <Standard_ImmutableObject.hxx>
#include <Standard_LicenseNotFound.hxx>
#include <Standard_MultiplyDefined.hxx>
#include <Standard_NegativeValue.hxx>
#include <Standard_NoMoreObject.hxx>
#include <Standard_NoSuchObject.hxx>
#include <Standard_NotImplemented.hxx>
#include <Standard_NullObject.hxx>
#include <Standard_NullValue.hxx>
#include <Standard_OutOfRange.hxx>
#include <Standard_Overflow.hxx>
#include <Standard_TypeMismatch.hxx>
#include <Standard_Underflow.hxx>
#include <functional>
#include <vector>

namespace exceptions_probe
{
// X-macro of every class; the order is the column order of the catch matrix.
#define TKERNEL_EXCEPTIONS(X)                                                                      \
  X(Standard_Failure)                                                                              \
  X(Standard_AbortiveTransaction)                                                                  \
  X(Standard_ConstructionError)                                                                    \
  X(Standard_DimensionError)                                                                       \
  X(Standard_DimensionMismatch)                                                                    \
  X(Standard_DivideByZero)                                                                         \
  X(Standard_DomainError)                                                                          \
  X(Standard_ImmutableObject)                                                                      \
  X(Standard_LicenseError)                                                                         \
  X(Standard_LicenseNotFound)                                                                      \
  X(Standard_MultiplyDefined)                                                                      \
  X(Standard_NegativeValue)                                                                        \
  X(Standard_NoMoreObject)                                                                         \
  X(Standard_NoSuchObject)                                                                         \
  X(Standard_NotImplemented)                                                                       \
  X(Standard_NullObject)                                                                           \
  X(Standard_NullValue)                                                                            \
  X(Standard_NumericError)                                                                         \
  X(Standard_OutOfRange)                                                                           \
  X(Standard_Overflow)                                                                             \
  X(Standard_ProgramError)                                                                         \
  X(Standard_RangeError)                                                                           \
  X(Standard_TypeMismatch)                                                                         \
  X(Standard_Underflow)                                                                            \
  X(StdFail_InfiniteSolutions)                                                                     \
  X(StdFail_NotDone)                                                                               \
  X(StdFail_Undefined)                                                                             \
  X(StdFail_UndefinedDerivative)                                                                   \
  X(StdFail_UndefinedValue)                                                                        \
  X(Plugin_Failure)

using Thrower = std::function<void(const char*)>;

static const std::map<std::string, Thrower>& throwers()
{
#define TKERNEL_THROWER(C) {#C, [](const char* theMessage) { throw C(theMessage); }},
  static const std::map<std::string, Thrower> THE_THROWERS = {TKERNEL_EXCEPTIONS(TKERNEL_THROWER)};
#undef TKERNEL_THROWER
  return THE_THROWERS;
}

// One '0'/'1' per class of the X-macro: would `catch (Column&)` catch the thrown class?
static std::string catch_matrix(const Thrower& theThrow)
{
  std::string aRow;
#define TKERNEL_CATCH(C)                                                                           \
  try                                                                                              \
  {                                                                                                \
    theThrow("");                                                                                  \
  }                                                                                                \
  catch (const C&)                                                                                 \
  {                                                                                                \
    aRow += '1';                                                                                   \
  }                                                                                                \
  catch (const Standard_Failure&)                                                                  \
  {                                                                                                \
    aRow += '0';                                                                                   \
  }
  TKERNEL_EXCEPTIONS(TKERNEL_CATCH)
#undef TKERNEL_CATCH
  return aRow;
}

static std::string format_exception(const Thrower& aThrow, const char* theMessage)
{
  try
  {
    aThrow(theMessage);
  }
  catch (const Standard_Failure& anError)
  {
    std::ostringstream aPrinted;
    aPrinted << anError;
    return catch_matrix(aThrow) + ' ' + anError.ExceptionType() + ' '
           + tkernel_probe::hex(std::string(anError.what())) + ' '
           + tkernel_probe::hex(aPrinted.str());
  }
  return "not thrown";
}

// "exception_kind <Class> <hex message>": catch matrix, ExceptionType, what() and Print().
static std::string kind_case(std::istream& theInput)
{
  std::string aName;
  theInput >> aName;
  const std::string aMessage = tkernel_probe::next_bytes(theInput);
  return format_exception(throwers().at(aName), aMessage.c_str());
}

static std::string default_case(std::istream& theInput)
{
  std::string aName;
  theInput >> aName;
  static const std::map<std::string, Thrower> aDefaults = {
    {"Standard_DomainError", [](const char*) { throw Standard_DomainError(); }},
    {"Standard_RangeError", [](const char*) { throw Standard_RangeError(); }},
    {"Standard_NotImplemented", [](const char*) { throw Standard_NotImplemented(); }}};
  return format_exception(aDefaults.at(aName), "");
}

static const tkernel_probe::Table cases = {
  {"exception_kind", kind_case}, {"exception_default", default_case}};
} // namespace exceptions_probe
