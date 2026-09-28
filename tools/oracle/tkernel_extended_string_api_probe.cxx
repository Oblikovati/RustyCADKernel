// Differential cases for TCollection_ExtendedString and TCollection_HExtendedString.
//
// Request fields: code-unit strings are hex of little-endian UTF-16 units ("-" = empty),
// "null" stands for a null char16_t pointer, byte strings are hex bytes, integers are
// decimal and reals are the decimal IEEE-754 bit pattern. A string result prints as
// "<Length> <units hex>". Exceptions print as "ERR <class>": the Rust messages add the
// offending value to OCCT's text, so only the class is compared.
#include "tkernel_api_common.hxx"
#include <TCollection_AsciiString.hxx>
#include <TCollection_ExtendedString.hxx>
#include <TCollection_HAsciiString.hxx>
#include <TCollection_HExtendedString.hxx>
#include <functional>
#include <vector>

namespace extended_string_api_probe
{
using Units   = std::u16string;
using SubCase = std::function<std::string(std::istream&)>;

// A nullable char16_t pointer with the units behind it; the buffer always ends with a zero.
struct Span
{
  bool  IsNull = false;
  Units Buffer;

  const char16_t* Pointer() const { return IsNull ? nullptr : Buffer.c_str(); }
};

inline Units next_units(std::istream& theInput)
{
  const std::string aBytes = tkernel_probe::next_bytes(theInput);
  Units             aUnits;
  for (std::size_t i = 0; i + 1 < aBytes.size(); i += 2)
    aUnits.push_back(char16_t(static_cast<unsigned char>(aBytes[i])
                              | (static_cast<unsigned char>(aBytes[i + 1]) << 8)));
  return aUnits;
}

inline Span next_span(std::istream& theInput)
{
  std::string aToken;
  theInput >> aToken;
  Span aSpan;
  if (aToken == "null")
  {
    aSpan.IsNull = true;
    return aSpan;
  }
  std::istringstream aField(aToken);
  aSpan.Buffer = next_units(aField);
  return aSpan;
}

template <class T>
T next(std::istream& theInput)
{
  T aValue{};
  theInput >> aValue;
  return aValue;
}

inline double next_real(std::istream& theInput)
{
  const std::uint64_t aBits = next<std::uint64_t>(theInput);
  double              aValue;
  std::memcpy(&aValue, &aBits, 8);
  return aValue;
}

inline std::string units_hex(const char16_t* theUnits, int theLength)
{
  return tkernel_probe::hex(theUnits, std::size_t(theLength) * 2);
}

inline std::string describe(const TCollection_ExtendedString& theString)
{
  return std::to_string(theString.Length()) + " "
         + units_hex(theString.ToExtString(), theString.Length());
}

inline std::string flag(bool theValue)
{
  return theValue ? "1" : "0";
}

// guarded() without the message: "OK <result>" or "ERR <class>".
template <class Operation>
std::string guarded_kind(Operation theOperation)
{
  const std::string aResult = tkernel_probe::guarded(theOperation);
  if (aResult.compare(0, 4, "ERR ") != 0)
    return aResult;
  return aResult.substr(0, aResult.find(": "));
}

// An AsciiString holding arbitrary bytes, embedded NULs included.
inline TCollection_AsciiString ascii_of(const std::string& theBytes)
{
  TCollection_AsciiString anAscii(int(theBytes.size()), '\0');
  for (std::size_t i = 0; i < theBytes.size(); ++i)
    if (theBytes[i] != '\0')
      anAscii.SetValue(int(i) + 1, theBytes[i]);
  return anAscii;
}

inline std::string dispatch(const std::map<std::string, SubCase>& theTable, std::istream& theInput)
{
  const std::string aName = next<std::string>(theInput);
  return theTable.at(aName)(theInput);
}
} // namespace extended_string_api_probe

#include "tkernel_extended_string_api_string.cxx"
#include "tkernel_extended_string_api_edit.cxx"
#include "tkernel_extended_string_api_handle.cxx"

namespace extended_string_api_probe
{
static const tkernel_probe::Table cases = {
  {"extended_string_new", new_case},
  {"extended_string_convert", convert_case},
  {"extended_string_numeric", numeric_case},
  {"extended_string_append", append_case},
  {"extended_string_edit", edit_case},
  {"extended_string_span", span_case},
  {"extended_string_whole", whole_case},
  {"extended_string_token", token_case},
  {"extended_string_locate", locate_case},
  {"extended_string_value", value_case},
  {"extended_string_hnew", hnew_case},
  {"extended_string_hedit", hedit_case},
  {"extended_string_hkey", hkey_case},
};
} // namespace extended_string_api_probe
