// Shared helpers for the TKernel API differential probe (tkernel_api_probe.cxx).
// Every probe case reads one request line and returns one response line.
#pragma once
#include <Standard_Failure.hxx>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <map>
#include <sstream>
#include <string>

namespace tkernel_probe
{
using Case  = std::string (*)(std::istream&);
using Table = std::map<std::string, Case>;

// IEEE-754 bit pattern of a double, so results are compared exactly.
inline std::uint64_t bits(double theValue)
{
  std::uint64_t aResult;
  std::memcpy(&aResult, &theValue, 8);
  return aResult;
}

// IEEE-754 bit pattern of a float.
inline std::uint32_t bits32(float theValue)
{
  std::uint32_t aResult;
  std::memcpy(&aResult, &theValue, 4);
  return aResult;
}

// Lower-case hex of raw bytes; "-" for an empty span so every field is one token.
inline std::string hex(const void* theData, std::size_t theSize)
{
  if (theSize == 0)
    return "-";
  std::ostringstream anOut;
  const auto*        aBytes = static_cast<const unsigned char*>(theData);
  for (std::size_t i = 0; i < theSize; ++i)
    anOut << std::hex << std::setw(2) << std::setfill('0') << static_cast<unsigned>(aBytes[i]);
  return anOut.str();
}

inline std::string hex(const std::string& theText)
{
  return hex(theText.data(), theText.size());
}

// Inverse of hex(): request fields carry arbitrary bytes, including NUL, as hex.
inline std::string decode(const std::string& theHex)
{
  std::string aBytes;
  if (theHex == "-")
    return aBytes;
  for (std::size_t i = 0; i + 1 < theHex.size(); i += 2)
    aBytes.push_back(static_cast<char>(std::stoul(theHex.substr(i, 2), nullptr, 16)));
  return aBytes;
}

// Reads the next whitespace token of a request as hex-encoded bytes.
inline std::string next_bytes(std::istream& theInput)
{
  std::string aHex;
  theInput >> aHex;
  return decode(aHex);
}

// Runs one operation; an OCCT exception becomes "ERR <type>: <message>", like Rust's Err.
template <class Operation>
std::string guarded(Operation theOperation)
{
  try
  {
    return "OK " + theOperation();
  }
  catch (const Standard_Failure& anError)
  {
    return std::string("ERR ") + anError.ExceptionType() + ": " + anError.what();
  }
}
} // namespace tkernel_probe
