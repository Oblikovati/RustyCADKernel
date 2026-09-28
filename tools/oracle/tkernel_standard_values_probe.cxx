// Differential cases for Standard scalar, character, hashing, time, GUID and atomic helpers.
// The scalar and hash cases live in the two included files; GUID, time, atomic, limits and
// version cases are here.
#include "tkernel_api_common.hxx"
#include "tkernel_standard_values_hash_probe.cxx"
#include <Standard_Atomic.hxx>
#include <Standard_GUID.hxx>
#include <Standard_Time.hxx>
#include <Standard_VersionInfo.hxx>
#include <functional>

namespace standard_values_probe
{
using standard_values_scalar_probe::hex32;
using standard_values_scalar_probe::hex64;

// Text, UUID fields, std::hash, ToExtString and ShallowDump of one GUID.
static std::string guid_image(const Standard_GUID& theGuid)
{
  char aText[Standard_GUID_SIZE_ALLOC];
  theGuid.ToCString(aText);
  char16_t anExt[Standard_GUID_SIZE_ALLOC];
  theGuid.ToExtString(anExt);
  std::ostringstream aDump;
  theGuid.ShallowDump(aDump);
  const Standard_UUID anUuid = theGuid.ToUUID();
  std::ostringstream  anOut;
  anOut << aText << ' ' << anUuid.Data1 << ' ' << anUuid.Data2 << ' ' << anUuid.Data3 << ' '
        << tkernel_probe::hex(anUuid.Data4, 8) << ' ' << hex64(std::hash<Standard_GUID>{}(theGuid))
        << ' ' << tkernel_probe::hex(anExt, sizeof(anExt)) << ' ' << aDump.str();
  return anOut.str();
}

// "guid_text <hex>": CheckGUIDFormat and Standard_GUID(const char*) on a C string.
static std::string guid_text_case(std::istream& theInput)
{
  const std::string aText = tkernel_probe::next_bytes(theInput);
  const char*       aC    = aText.c_str();
  return std::string(Standard_GUID::CheckGUIDFormat(aC) ? "1 " : "0 ")
         + tkernel_probe::guarded([&] { return guid_image(Standard_GUID(aC)); });
}

// "guid_utf16 <hex units>": Standard_GUID(const char16_t*) on at least 36 units.
static std::string guid_utf16_case(std::istream& theInput)
{
  std::string aHex;
  theInput >> aHex;
  std::u16string aUnits;
  for (std::size_t i = 0; i + 3 < aHex.size(); i += 4)
    aUnits.push_back(static_cast<char16_t>(std::stoul(aHex.substr(i, 4), nullptr, 16)));
  return tkernel_probe::guarded([&] { return guid_image(Standard_GUID(aUnits.c_str())); });
}

// "guid_components a b c d e f g h i j": the component and UUID constructors.
static std::string guid_components_case(std::istream& theInput)
{
  long long aParts[10] = {};
  for (long long& aPart : aParts)
    theInput >> aPart;
  const Standard_GUID aGuid(static_cast<int>(aParts[0]),
                            static_cast<char16_t>(aParts[1]),
                            static_cast<char16_t>(aParts[2]),
                            static_cast<char16_t>(aParts[3]),
                            static_cast<uint8_t>(aParts[4]),
                            static_cast<uint8_t>(aParts[5]),
                            static_cast<uint8_t>(aParts[6]),
                            static_cast<uint8_t>(aParts[7]),
                            static_cast<uint8_t>(aParts[8]),
                            static_cast<uint8_t>(aParts[9]));
  const Standard_GUID aCopy(aGuid.ToUUID());
  return guid_image(aGuid) + ' ' + (aCopy == aGuid ? '1' : '0');
}

// "guid_compare <hex> <hex>": ==, !=, IsSame, IsNotSame and both Assign/operator= overloads.
static std::string guid_compare_case(std::istream& theInput)
{
  const Standard_GUID aFirst(tkernel_probe::next_bytes(theInput).c_str());
  const Standard_GUID aSecond(tkernel_probe::next_bytes(theInput).c_str());
  Standard_GUID       anAssigned, aFromUuid, anOperator, anUuidOperator;
  anAssigned.Assign(aSecond);
  aFromUuid.Assign(aFirst.ToUUID());
  anOperator     = aSecond;
  anUuidOperator = aFirst.ToUUID();
  std::string aFlags;
  for (bool aFlag : {aFirst == aSecond, aFirst != aSecond, aFirst.IsSame(aSecond),
                     aFirst.IsNotSame(aSecond), anAssigned.IsSame(aSecond),
                     aFromUuid.IsSame(aFirst), anOperator == aSecond, anUuidOperator == aFirst})
    aFlags += aFlag ? '1' : '0';
  return aFlags;
}

// "time_equal <a> <b>": the std::time_t IsEqual overload.
static std::string time_equal_case(std::istream& theInput)
{
  long long aFirst = 0, aSecond = 0;
  theInput >> aFirst >> aSecond;
  return IsEqual(static_cast<std::time_t>(aFirst), static_cast<std::time_t>(aSecond)) ? "1" : "0";
}

// "atomic <start> <old> <new>": increment, decrement twice, increment, then two CAS calls.
static std::string atomic_case(std::istream& theInput)
{
  int aStart = 0, anOld = 0, aNew = 0;
  theInput >> aStart >> anOld >> aNew;
  volatile int       aValue = aStart;
  std::ostringstream anOut;
  anOut << Standard_Atomic_Increment(&aValue) << ' ' << Standard_Atomic_Decrement(&aValue) << ' ';
  anOut << Standard_Atomic_Decrement(&aValue) << ' ' << Standard_Atomic_Increment(&aValue) << ' ';
  anOut << Standard_Atomic_CompareAndSwap(&aValue, anOld, aNew) << ' ' << aValue << ' ';
  anOut << Standard_Atomic_CompareAndSwap(&aValue, anOld, aStart) << ' ' << aValue;
  return anOut.str();
}

// "limits": every Standard_Real/ShortReal/Integer limit function.
static std::string limits_case(std::istream&)
{
  std::ostringstream anOut;
  anOut << hex64(tkernel_probe::bits(RealSmall())) << ' ' << RealDigits() << ' '
        << hex64(tkernel_probe::bits(RealEpsilon())) << ' '
        << hex64(tkernel_probe::bits(RealFirst())) << ' ' << RealFirst10Exp() << ' '
        << hex64(tkernel_probe::bits(RealLast())) << ' ' << RealLast10Exp() << ' '
        << RealMantissa() << ' ' << RealRadix() << ' ' << RealSize() << ' '
        << hex32(tkernel_probe::bits32(ShortRealSmall())) << ' ' << ShortRealDigits() << ' '
        << hex32(tkernel_probe::bits32(ShortRealEpsilon())) << ' '
        << hex32(tkernel_probe::bits32(ShortRealFirst())) << ' ' << ShortRealFirst10Exp() << ' '
        << hex32(tkernel_probe::bits32(ShortRealLast())) << ' ' << ShortRealLast10Exp() << ' '
        << ShortRealMantissa() << ' ' << ShortRealRadix() << ' ' << ShortRealSize() << ' '
        << IntegerFirst() << ' ' << IntegerLast() << ' ' << IntegerSize();
  return anOut.str();
}

// "version": the Standard_VersionInfo functions.
static std::string version_case(std::istream&)
{
  std::ostringstream anOut;
  anOut << tkernel_probe::hex(OCCT_DevelopmentVersion()) << ' '
        << hex64(tkernel_probe::bits(OCCT_Version_Double())) << ' ' << OCCT_Version_String() << ' '
        << OCCT_Version_String_Complete() << ' ' << OCCT_Version_String_Extended();
  return anOut.str();
}

static tkernel_probe::Table merged_cases()
{
  tkernel_probe::Table aCases = {{"standard_values_guid_text", guid_text_case},
                                 {"standard_values_guid_utf16", guid_utf16_case},
                                 {"standard_values_guid_components", guid_components_case},
                                 {"standard_values_guid_compare", guid_compare_case},
                                 {"standard_values_time_equal", time_equal_case},
                                 {"standard_values_atomic", atomic_case},
                                 {"standard_values_limits", limits_case},
                                 {"standard_values_version", version_case}};
  aCases.insert(standard_values_scalar_probe::cases.begin(), standard_values_scalar_probe::cases.end());
  aCases.insert(standard_values_hash_probe::cases.begin(), standard_values_hash_probe::cases.end());
  return aCases;
}

static const tkernel_probe::Table cases = merged_cases();
} // namespace standard_values_probe
