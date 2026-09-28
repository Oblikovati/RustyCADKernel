// Message package differential operations (twin of oracle_probe/message/).
//
// Every string crosses the protocol as hex: bytes as pairs ("-" when empty), UTF-16 text
// as comma-separated units ("-" when empty). Step lists use ':'-separated fields.
// A Standard_Failure inside a step prints " E<ExceptionType>" and ends that operation.
#include <Standard_Failure.hxx>
#include <TCollection_AsciiString.hxx>
#include <TCollection_ExtendedString.hxx>

#include <cstdint>
#include <cstring>
#include <iomanip>
#include <istream>
#include <ostream>
#include <sstream>
#include <string>
#include <vector>

static std::string msgp_hex(const std::string& theBytes)
{
  if (theBytes.empty())
  {
    return "-";
  }
  std::ostringstream aText;
  aText << std::hex << std::setfill('0');
  for (unsigned char aByte : theBytes)
  {
    aText << std::setw(2) << static_cast<unsigned>(aByte);
  }
  return aText.str();
}

static std::string msgp_bytes(const std::string& theHex)
{
  std::string aBytes;
  if (theHex == "-")
  {
    return aBytes;
  }
  for (size_t anIndex = 0; anIndex + 1 < theHex.size(); anIndex += 2)
  {
    aBytes.push_back(static_cast<char>(std::stoul(theHex.substr(anIndex, 2), nullptr, 16)));
  }
  return aBytes;
}

static std::string msgp_ascii(const TCollection_AsciiString& theText)
{
  return msgp_hex(std::string(theText.ToCString(), static_cast<size_t>(theText.Length())));
}

static std::string msgp_units(const TCollection_ExtendedString& theText)
{
  if (theText.Length() == 0)
  {
    return "-";
  }
  std::ostringstream aText;
  aText << std::hex;
  for (int anIndex = 1; anIndex <= theText.Length(); ++anIndex)
  {
    aText << (anIndex == 1 ? "" : ",") << static_cast<unsigned>(theText.Value(anIndex));
  }
  return aText.str();
}

static TCollection_ExtendedString msgp_ext(const std::string& theToken)
{
  TCollection_ExtendedString aText;
  if (theToken == "-")
  {
    return aText;
  }
  std::istringstream aUnits(theToken);
  std::string        aUnit;
  while (std::getline(aUnits, aUnit, ','))
  {
    aText.AssignCat(static_cast<char16_t>(std::stoul(aUnit, nullptr, 16)));
  }
  return aText;
}

static double msgp_real(const std::string& theBits)
{
  const uint64_t aBits = std::stoull(theBits, nullptr, 16);
  double         aValue;
  std::memcpy(&aValue, &aBits, sizeof(aValue));
  return aValue;
}

static std::string msgp_real_bits(double theValue)
{
  uint64_t aBits;
  std::memcpy(&aBits, &theValue, sizeof(aBits));
  std::ostringstream aText;
  aText << std::hex << aBits;
  return aText.str();
}

static std::vector<std::string> msgp_fields(const std::string& theStep)
{
  std::vector<std::string> aFields;
  std::istringstream       aStep(theStep);
  std::string              aField;
  while (std::getline(aStep, aField, ':'))
  {
    aFields.push_back(aField);
  }
  return aFields;
}

static std::vector<std::string> msgp_rest(std::istream& in)
{
  std::vector<std::string> aTokens;
  std::string              aToken;
  while (in >> aToken)
  {
    aTokens.push_back(aToken);
  }
  return aTokens;
}

#include "tkernel_message_text_probe.cxx"
#include "tkernel_message_report_probe.cxx"
#include "tkernel_message_output_probe.cxx"

static bool message_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  return message_text_probe(op, in, out) || message_report_probe(op, in, out)
         || message_output_probe(op, in, out);
}
