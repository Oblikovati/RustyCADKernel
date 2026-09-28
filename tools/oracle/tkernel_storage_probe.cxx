// Storage/FSD differential operations (twin of crates/tkernel/examples/oracle_probe/storage/).
//
// Byte strings travel as lowercase hex ("-" for empty); UTF-16 strings as four hex digits
// per unit. Exceptions are reported by class name only ("!Class"): the Rust messages carry
// diagnostics the native ones lack. Drivers work on real files in a private temporary
// directory; the Rust twin injects an in-memory file system holding the same bytes.
#include <Standard_Failure.hxx>
#include <TCollection_AsciiString.hxx>
#include <TCollection_ExtendedString.hxx>

#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <istream>
#include <iterator>
#include <ostream>
#include <sstream>
#include <string>
#include <unistd.h>
#include <vector>

namespace
{
std::string stor_hex(const std::string& theBytes)
{
  static const char* aDigits = "0123456789abcdef";
  if (theBytes.empty())
    return "-";
  std::string aText;
  for (unsigned char aByte : theBytes)
  {
    aText += aDigits[aByte >> 4];
    aText += aDigits[aByte & 15];
  }
  return aText;
}

std::string stor_unhex(const std::string& theText)
{
  std::string aBytes;
  if (theText == "-")
    return aBytes;
  for (size_t anIndex = 0; anIndex + 1 < theText.size(); anIndex += 2)
    aBytes.push_back(static_cast<char>(std::stoi(theText.substr(anIndex, 2), nullptr, 16)));
  return aBytes;
}

std::string stor_ascii_hex(const TCollection_AsciiString& theText)
{
  return stor_hex(std::string(theText.ToCString(), static_cast<size_t>(theText.Length())));
}

std::string stor_ext_hex(const TCollection_ExtendedString& theText)
{
  if (theText.Length() == 0)
    return "-";
  char        aUnit[5];
  std::string aText;
  for (int anIndex = 1; anIndex <= theText.Length(); ++anIndex)
  {
    std::snprintf(aUnit, sizeof(aUnit), "%04x", static_cast<unsigned>(theText.Value(anIndex)));
    aText += aUnit;
  }
  return aText;
}

TCollection_ExtendedString stor_ext_unhex(const std::string& theText)
{
  TCollection_ExtendedString aText;
  if (theText == "-")
    return aText;
  for (size_t anIndex = 0; anIndex + 3 < theText.size(); anIndex += 4)
    aText.AssignCat(static_cast<char16_t>(std::stoi(theText.substr(anIndex, 4), nullptr, 16)));
  return aText;
}

// A private scratch file, unique per process.
std::string stor_scratch(const char* theTag)
{
  return std::string("/tmp/tkernel_storage_probe_") + std::to_string(getpid()) + "_" + theTag;
}

void stor_write_file(const std::string& thePath, const std::string& theBytes)
{
  std::ofstream aFile(thePath, std::ios::binary | std::ios::trunc);
  aFile.write(theBytes.data(), static_cast<std::streamsize>(theBytes.size()));
}

std::string stor_read_file(const std::string& thePath)
{
  std::ifstream aFile(thePath, std::ios::binary);
  if (!aFile)
    return "absent";
  return stor_hex(std::string(std::istreambuf_iterator<char>(aFile), std::istreambuf_iterator<char>()));
}

std::vector<std::string> stor_split(const std::string& theText, char theSeparator)
{
  std::vector<std::string> aParts;
  std::string              aPart;
  std::istringstream       aStream(theText);
  while (std::getline(aStream, aPart, theSeparator))
    aParts.push_back(aPart);
  if (!theText.empty() && theText.back() == theSeparator)
    aParts.push_back("");
  return aParts;
}

std::string stor_bits(double theValue)
{
  uint64_t aBits = 0;
  std::memcpy(&aBits, &theValue, sizeof(aBits));
  char aText[17];
  std::snprintf(aText, sizeof(aText), "%016llx", static_cast<unsigned long long>(aBits));
  return aText;
}

std::string stor_bits(float theValue)
{
  uint32_t aBits = 0;
  std::memcpy(&aBits, &theValue, sizeof(aBits));
  char aText[9];
  std::snprintf(aText, sizeof(aText), "%08x", static_cast<unsigned>(aBits));
  return aText;
}

double stor_double(const std::string& theHex)
{
  uint64_t aBits  = std::stoull(theHex, nullptr, 16);
  double   aValue = 0.0;
  std::memcpy(&aValue, &aBits, sizeof(aValue));
  return aValue;
}

float stor_float(const std::string& theHex)
{
  uint32_t aBits  = static_cast<uint32_t>(std::stoul(theHex, nullptr, 16));
  float    aValue = 0.0f;
  std::memcpy(&aValue, &aBits, sizeof(aValue));
  return aValue;
}
} // namespace

#include "tkernel_storage_streams_probe.cxx"
#include "tkernel_storage_drivers_probe.cxx"
#include "tkernel_storage_schema_probe.cxx"

static bool storage_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  return storage_streams_probe(op, in, out) || storage_drivers_probe(op, in, out)
         || storage_schema_probe(op, in, out);
}
