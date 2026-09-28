// Standard_CLocaleSentry: the C locale object and number text inside a sentry.
// Included by tkernel_standard_services_probe.cxx before its case table.
#include <Standard_CLocaleSentry.hxx>
#include <clocale>
#include <cstdio>

namespace standard_services_probe
{
// "services_clocale <real bits>": GetCLocale identity, then %g text, radix and thread-locale
// restoration around (nested) sentries.
static std::string clocale_case(std::istream& theInput)
{
  std::uint64_t aBits = 0;
  theInput >> aBits;
  double aValue = 0.0;
  std::memcpy(&aValue, &aBits, sizeof(aValue));
  const auto  aFirst  = Standard_CLocaleSentry::GetCLocale();
  const auto  aSecond = Standard_CLocaleSentry::GetCLocale();
  std::string aText;
  const auto  aBefore = uselocale(nullptr);
  {
    Standard_CLocaleSentry anOuter;
    {
      Standard_CLocaleSentry anInner;
      char                   aBuffer[64];
      std::snprintf(aBuffer, sizeof(aBuffer), "%g", aValue);
      aText = std::string(aBuffer) + " point=" + std::localeconv()->decimal_point
              + " sep=" + tkernel_probe::hex(std::string(std::localeconv()->thousands_sep));
    }
  }
  const bool isRestored = uselocale(nullptr) == aBefore;
  return "nonnull=" + std::to_string(aFirst != nullptr) + " same=" + std::to_string(aFirst == aSecond)
         + " " + aText + " restored=" + std::to_string(isRestored);
}
} // namespace standard_services_probe
