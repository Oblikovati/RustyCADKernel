// Standard_Real, Standard_ShortReal, Standard_Integer, Standard_Character and
// Standard_ExtCharacter cases of the standard_values probe (included by
// tkernel_standard_values_probe.cxx).
#include "tkernel_api_common.hxx"
#include <Standard_Character.hxx>
#include <Standard_ExtCharacter.hxx>
#include <Standard_Integer.hxx>
#include <Standard_Real.hxx>
#include <Standard_ShortReal.hxx>
#include <functional>

namespace standard_values_scalar_probe
{
// 16 lower-case hex digits of a 64-bit pattern.
inline std::string hex64(std::uint64_t theValue)
{
  std::ostringstream anOut;
  anOut << std::hex << std::setw(16) << std::setfill('0') << theValue;
  return anOut.str();
}

// 8 lower-case hex digits of a 32-bit pattern.
inline std::string hex32(std::uint32_t theValue)
{
  std::ostringstream anOut;
  anOut << std::hex << std::setw(8) << std::setfill('0') << theValue;
  return anOut.str();
}

// Reads a double sent as its IEEE-754 bit pattern.
inline double next_real(std::istream& theInput)
{
  std::string aToken;
  theInput >> aToken;
  const std::uint64_t aBits = std::stoull(aToken, nullptr, 16);
  double              aValue;
  std::memcpy(&aValue, &aBits, 8);
  return aValue;
}

// Reads a float sent as its IEEE-754 bit pattern.
inline float next_short_real(std::istream& theInput)
{
  std::string aToken;
  theInput >> aToken;
  const std::uint32_t aBits = static_cast<std::uint32_t>(std::stoul(aToken, nullptr, 16));
  float               aValue;
  std::memcpy(&aValue, &aBits, 4);
  return aValue;
}

// Reads one byte sent as two hex digits.
inline unsigned next_hex_unit(std::istream& theInput)
{
  std::string aToken;
  theInput >> aToken;
  return static_cast<unsigned>(std::stoul(aToken, nullptr, 16));
}

static const std::map<std::string, std::function<double(double)>>& real_unary()
{
  static const std::map<std::string, std::function<double(double)>> THE_TABLE = {
    {"acos", [](double x) { return ACos(x); }},
    {"acos_approx", [](double x) { return ACosApprox(x); }},
    {"asin", [](double x) { return ASin(x); }},
    {"atan", [](double x) { return ATan(x); }},
    {"atanh", [](double x) { return ATanh(x); }},
    {"acosh", [](double x) { return ACosh(x); }},
    {"asinh", [](double x) { return ASinh(x); }},
    {"cosh", [](double x) { return Cosh(x); }},
    {"sinh", [](double x) { return Sinh(x); }},
    {"tanh", [](double x) { return Tanh(x); }},
    {"cos", [](double x) { return Cos(x); }},
    {"sin", [](double x) { return Sin(x); }},
    {"tan", [](double x) { return Tan(x); }},
    {"exp", [](double x) { return Exp(x); }},
    {"log", [](double x) { return Log(x); }},
    {"log10", [](double x) { return Log10(x); }},
    {"sqrt", [](double x) { return Sqrt(x); }},
    {"abs", [](double x) { return Abs(x); }},
    {"ceiling", [](double x) { return Ceiling(x); }},
    {"floor", [](double x) { return Floor(x); }},
    {"integer_part", [](double x) { return IntegerPart(x); }},
    {"round", [](double x) { return Round(x); }},
    {"epsilon", [](double x) { return Epsilon(x); }},
    {"real_part", [](double x) { return RealPart(x); }},
    {"square", [](double x) { return Square(x); }}};
  return THE_TABLE;
}

static const std::map<std::string, std::function<double(double, double)>>& real_binary()
{
  static const std::map<std::string, std::function<double(double, double)>> THE_TABLE = {
    {"atan2", [](double x, double y) { return ATan2(x, y); }},
    {"next_after", [](double x, double y) { return NextAfter(x, y); }},
    {"sign", [](double x, double y) { return Sign(x, y); }},
    {"max", [](double x, double y) { return Max(x, y); }},
    {"min", [](double x, double y) { return Min(x, y); }},
    {"pow", [](double x, double y) { return Pow(x, y); }},
    {"is_equal", [](double x, double y) { return IsEqual(x, y) ? 1.0 : 0.0; }}};
  return THE_TABLE;
}

// "real1 <name> <bits>": one deprecated or plain unary Standard_Real function.
static std::string real1_case(std::istream& theInput)
{
  std::string aName;
  theInput >> aName;
  const double aValue = next_real(theInput);
  return hex64(tkernel_probe::bits(real_unary().at(aName)(aValue)));
}

// "real2 <name> <bits> <bits>": one binary Standard_Real function.
static std::string real2_case(std::istream& theInput)
{
  std::string aName;
  theInput >> aName;
  const double aFirst  = next_real(theInput);
  const double aSecond = next_real(theInput);
  return hex64(tkernel_probe::bits(real_binary().at(aName)(aFirst, aSecond)));
}

static std::string real_to_int_case(std::istream& theInput)
{
  return std::to_string(RealToInt(next_real(theInput)));
}

static std::string real_to_short_real_case(std::istream& theInput)
{
  return hex32(tkernel_probe::bits32(RealToShortReal(next_real(theInput))));
}

static std::string int_to_real_case(std::istream& theInput)
{
  int aValue = 0;
  theInput >> aValue;
  return hex64(tkernel_probe::bits(IntToReal(aValue)));
}

// "short_real <abs|max|min> <bits32> <bits32>": the deprecated float overloads.
static std::string short_real_case(std::istream& theInput)
{
  std::string aName;
  theInput >> aName;
  const float aFirst  = next_short_real(theInput);
  const float aSecond = next_short_real(theInput);
  const float aResult = aName == "abs"   ? Abs(aFirst)
                        : aName == "max" ? Max(aFirst, aSecond)
                                         : Min(aFirst, aSecond);
  return hex32(tkernel_probe::bits32(aResult));
}

// "integer <name> <a> <b>": the Standard_Integer helpers (the corpus avoids their UB).
static std::string integer_case(std::istream& theInput)
{
  std::string aName;
  int         aFirst = 0, aSecond = 0;
  theInput >> aName >> aFirst >> aSecond;
  if (aName == "is_even" || aName == "is_odd")
    return (aName == "is_even" ? IsEven(aFirst) : IsOdd(aFirst)) ? "1" : "0";
  const int aResult = aName == "abs"     ? Abs(aFirst)
                      : aName == "max"   ? Max(aFirst, aSecond)
                      : aName == "min"   ? Min(aFirst, aSecond)
                      : aName == "square" ? Square(aFirst)
                                          : Modulus(aFirst, aSecond);
  return std::to_string(aResult);
}

// "character <hex byte>": the 13 classifier bits, then LowerCase and UpperCase as hex.
static std::string character_case(std::istream& theInput)
{
  const char aChar = static_cast<char>(next_hex_unit(theInput));
  const bool aFlags[] = {IsAlphabetic(aChar), IsDigit(aChar),     IsXDigit(aChar),
                         IsAlphanumeric(aChar), IsControl(aChar),  IsGraphic(aChar),
                         IsLowerCase(aChar),  IsPrintable(aChar), IsPunctuation(aChar),
                         IsSpace(aChar),      IsUpperCase(aChar)};
  std::string aResult;
  for (bool aFlag : aFlags)
    aResult += aFlag ? '1' : '0';
  const char aLower = LowerCase(aChar);
  const char anUpper = UpperCase(aChar);
  return aResult + ' ' + tkernel_probe::hex(&aLower, 1) + ' ' + tkernel_probe::hex(&anUpper, 1);
}

static std::string char_equal_case(std::istream& theInput)
{
  const char aFirst  = static_cast<char>(next_hex_unit(theInput));
  const char aSecond = static_cast<char>(next_hex_unit(theInput));
  return IsEqual(aFirst, aSecond) ? "1" : "0";
}

static std::string to_ext_character_case(std::istream& theInput)
{
  const char aChar = static_cast<char>(next_hex_unit(theInput));
  return std::to_string(static_cast<unsigned>(ToExtCharacter(aChar)));
}

// "ext_character <unit> <unit>": ToCharacter, IsAnAscii and IsEqual.
static std::string ext_character_case(std::istream& theInput)
{
  const char16_t aFirst  = static_cast<char16_t>(next_hex_unit(theInput));
  const char16_t aSecond = static_cast<char16_t>(next_hex_unit(theInput));
  const char     aChar   = ToCharacter(aFirst);
  return tkernel_probe::hex(&aChar, 1) + ' ' + (IsAnAscii(aFirst) ? '1' : '0') + ' '
         + (IsEqual(aFirst, aSecond) ? '1' : '0');
}

static const tkernel_probe::Table cases = {
  {"standard_values_real1", real1_case},
  {"standard_values_real2", real2_case},
  {"standard_values_real_to_int", real_to_int_case},
  {"standard_values_real_to_short_real", real_to_short_real_case},
  {"standard_values_int_to_real", int_to_real_case},
  {"standard_values_short_real", short_real_case},
  {"standard_values_integer", integer_case},
  {"standard_values_character", character_case},
  {"standard_values_char_equal", char_equal_case},
  {"standard_values_to_ext_character", to_ext_character_case},
  {"standard_values_ext_character", ext_character_case}};
} // namespace standard_values_scalar_probe
