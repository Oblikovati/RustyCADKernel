#include <OSD_Exception.hxx>
#include <OSD_Exception_ACCESS_VIOLATION.hxx>
#include <OSD_OSDError.hxx>
#include <OSD_SIGSEGV.hxx>
#include <OSD_Signal.hxx>
#include <Standard_Failure.hxx>
#include <Standard_Transient.hxx>

#include <iostream>
#include <type_traits>

static_assert(!std::is_base_of_v<Standard_Transient, Standard_Failure>);
static_assert(!std::is_base_of_v<Standard_Transient, OSD_Exception>);
static_assert(!std::is_base_of_v<Standard_Transient, OSD_Signal>);
static_assert(!std::is_base_of_v<Standard_Transient, OSD_OSDError>);

template <class Exception, class Base>
void report_caught_as(const char* theLabel, const char* theBase)
{
  try
  {
    throw Exception("probe");
  }
  catch (const Base& theCaught)
  {
    std::cout << theLabel << ' ' << theCaught.ExceptionType() << ' ' << theBase << '\n';
  }
}

int main()
{
  report_caught_as<OSD_Exception_ACCESS_VIOLATION, OSD_Exception>("seh", "OSD_Exception");
  report_caught_as<OSD_SIGSEGV, OSD_Signal>("signal", "OSD_Signal");
  report_caught_as<OSD_OSDError, Standard_Failure>("osd_error", "Standard_Failure");
  std::cout << "standard_transient_base 0\n";
}
