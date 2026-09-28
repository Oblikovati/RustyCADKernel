// Differential cases for TCollection_AsciiString, TCollection_HAsciiString and TCollection.
//
// Requests are `<group> <member> <fields...>`; strings travel as hex so NUL and high-bit
// bytes survive. A string is built with Copy(ptr, len), which keeps embedded NULs, and is
// printed as `<hex> <Length()>`. Exceptions are compared by class only: the Rust messages
// add the offending value that OCCT's messages lack.
#include "tkernel_api_common.hxx"
#include <TCollection.hxx>
#include <TCollection_AsciiString.hxx>
#include <TCollection_ExtendedString.hxx>
#include <TCollection_HAsciiString.hxx>
#include <TCollection_HExtendedString.hxx>
#include <functional>
#include <string_view>

namespace ascii_string_api_probe
{
using Istream = std::istream;
using Ascii   = TCollection_AsciiString;

// A string holding exactly `theBytes`, embedded NULs included.
static Ascii make(const std::string& theBytes)
{
  Ascii aString;
  aString.Copy(theBytes.data(), static_cast<int>(theBytes.size()));
  return aString;
}

static std::string show(const Ascii& theString)
{
  return tkernel_probe::hex(theString.ToCString(), theString.Length()) + " "
         + std::to_string(theString.Length());
}

// "ERR <class>: <message>" becomes "ERR <class>".
static std::string kind_only(const std::string& theResponse)
{
  return theResponse.rfind("ERR ", 0) == 0 ? theResponse.substr(0, theResponse.find(':'))
                                            : theResponse;
}

static int next_int(Istream& theInput)
{
  long long aValue = 0;
  theInput >> aValue;
  return static_cast<int>(aValue);
}

static char next_char(Istream& theInput)
{
  return static_cast<char>(next_int(theInput));
}

static bool next_flag(Istream& theInput)
{
  return next_int(theInput) != 0;
}

static double next_real(Istream& theInput)
{
  std::string aHex;
  theInput >> aHex;
  const std::uint64_t aBits = std::stoull(aHex, nullptr, 16);
  double              aValue;
  std::memcpy(&aValue, &aBits, 8);
  return aValue;
}

static std::string next_text(Istream& theInput)
{
  return tkernel_probe::next_bytes(theInput);
}

// "<count> <u32>...": wchar_t units, terminated after the last one.
static std::wstring next_wide(Istream& theInput)
{
  std::wstring aText;
  for (int aCount = next_int(theInput); aCount > 0; --aCount)
  {
    unsigned long long aUnit = 0;
    theInput >> aUnit;
    aText.push_back(static_cast<wchar_t>(static_cast<std::uint32_t>(aUnit)));
  }
  return aText;
}

// Hex of 4-digit UTF-16 units; the ExtendedString copies through the first zero unit.
static TCollection_ExtendedString next_extended(Istream& theInput)
{
  std::string    aHex;
  std::u16string aUnits;
  theInput >> aHex;
  for (std::size_t i = 0; aHex != "-" && i + 3 < aHex.size(); i += 4)
    aUnits.push_back(static_cast<char16_t>(std::stoul(aHex.substr(i, 4), nullptr, 16)));
  return TCollection_ExtendedString(aUnits.c_str());
}

static std::string real_bits(double theValue)
{
  std::ostringstream anOut;
  anOut << std::hex << tkernel_probe::bits(theValue);
  return anOut.str();
}

using Edit = std::function<std::string(Ascii&, Istream&)>;
using Query = std::function<std::string(const Ascii&, Istream&)>;
using Make = std::function<Ascii(Istream&)>;

// Looks up `<member>` in `theTable`, or reports the probe bug as an unknown member.
template <class Map>
static const typename Map::mapped_type& member(const Map& theTable, Istream& theInput)
{
  std::string aName;
  theInput >> aName;
  const auto aFound = theTable.find(aName);
  if (aFound == theTable.end())
    throw std::runtime_error("unknown ascii_string_api member " + aName);
  return aFound->second;
}

#include "tkernel_ascii_string_api_edits.cxx"
#include "tkernel_ascii_string_api_queries.cxx"
#include "tkernel_ascii_string_api_h_probe.cxx"

static std::string edit_case(Istream& theInput)
{
  const Edit& anEdit = member(edits(), theInput);
  Ascii       aString = make(next_text(theInput));
  return kind_only(tkernel_probe::guarded([&] {
    const std::string anExtra = anEdit(aString, theInput);
    return show(aString) + anExtra;
  }));
}

static std::string query_case(Istream& theInput)
{
  const Query& aQuery  = member(queries(), theInput);
  const Ascii  aString = make(next_text(theInput));
  return kind_only(tkernel_probe::guarded([&] { return aQuery(aString, theInput); }));
}

static std::string make_case(Istream& theInput)
{
  const Make& aMake = member(makers(), theInput);
  return kind_only(tkernel_probe::guarded([&] { return show(aMake(theInput)); }));
}

static const tkernel_probe::Table cases = {
  {"ascii_make", make_case},
  {"ascii_edit", edit_case},
  {"ascii_query", query_case},
  {"ascii_stream", stream_case},
  {"ascii_h_make", h_make_case},
  {"ascii_h_edit", h_edit_case},
  {"ascii_h_query", h_query_case},
  {"ascii_next_prime", next_prime_case},
};
} // namespace ascii_string_api_probe
