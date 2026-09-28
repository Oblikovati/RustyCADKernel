// Standard_ReadBuffer and Standard_ReadLineBuffer over byte streams that may fail.
// Included by tkernel_standard_services_probe.cxx before its case table.
#include <Standard_ReadBuffer.hxx>
#include <Standard_ReadLineBuffer.hxx>
#include <cstdint>
#include <limits>
#include <stdexcept>

namespace standard_services_probe
{
// Serves the bytes one at a time and throws after `theFailAfter` of them, which
// std::istream::read turns into badbit with gcount() of the bytes before the failure.
class FailingBuffer : public std::streambuf
{
public:
  FailingBuffer(std::string theBytes, std::size_t theFailAfter)
      : myBytes(std::move(theBytes)),
        myFailAfter(theFailAfter)
  {
  }

protected:
  int_type underflow() override
  {
    if (myPosition >= myBytes.size())
      return traits_type::eof();
    if (myPosition >= myFailAfter)
      throw std::runtime_error("probe stream failure");
    return traits_type::to_int_type(myBytes[myPosition]);
  }

  int_type uflow() override
  {
    const int_type aByte = underflow();
    if (aByte != traits_type::eof())
      ++myPosition;
    return aByte;
  }

private:
  std::string myBytes;
  std::size_t myFailAfter;
  std::size_t myPosition = 0;
};

static std::size_t fail_after(const std::string& theToken)
{
  return theToken == "-" ? std::numeric_limits<std::size_t>::max() : std::stoull(theToken);
}

// "services_read_buffer <data len> <chunk> <partial 0|1> <fail after|-> <hex bytes> <d|u>":
// IsDone, then every chunk (hex, or as a native-endian uint32 in mode u) until NULL.
static std::string read_buffer_case(std::istream& theInput)
{
  long long   aDataLen = 0;
  std::size_t aChunk   = 0;
  int         isPartial = 0;
  std::string aFail, aMode;
  theInput >> aDataLen >> aChunk >> isPartial >> aFail;
  const std::string aBytes = tkernel_probe::next_bytes(theInput);
  theInput >> aMode;
  return tkernel_probe::guarded([&] {
    Standard_ReadBuffer aBuffer(aDataLen, aChunk, isPartial != 0);
    FailingBuffer       aSource(aBytes, fail_after(aFail));
    std::istream        aStream(&aSource);
    std::string         aText = "done=" + std::to_string(aBuffer.IsDone());
    for (int aCall = 0; aCall < 5000; ++aCall)
    {
      if (aMode == "u")
      {
        const uint32_t* aValue = aBuffer.ReadChunk<uint32_t>(aStream);
        aText += aValue == nullptr ? " null" : " " + std::to_string(*aValue);
        if (aValue == nullptr)
          break;
        continue;
      }
      const char* aChunkData = aBuffer.ReadDataChunk(aStream);
      aText += aChunkData == nullptr ? " null" : " " + tkernel_probe::hex(aChunkData, aChunk);
      if (aChunkData == nullptr)
        break;
    }
    return aText + " done=" + std::to_string(aBuffer.IsDone());
  });
}

// "services_read_line <max> <multiline> <gap> <fail after|-> <clear after|-> <hex bytes>":
// every line as hex:readData until NULL; Clear() runs after the given number of lines.
static std::string read_line_case(std::istream& theInput)
{
  std::size_t aMax = 0;
  int         isMultiline = 0, isGap = 0;
  std::string aFail, aClear;
  theInput >> aMax >> isMultiline >> isGap >> aFail >> aClear;
  const std::string       aBytes = tkernel_probe::next_bytes(theInput);
  Standard_ReadLineBuffer aBuffer(aMax);
  aBuffer.SetMultilineMode(isMultiline != 0, isGap != 0);
  FailingBuffer aSource(aBytes, fail_after(aFail));
  std::istream  aStream(&aSource);
  std::string   aText;
  for (int aCall = 0; aCall < 2000; ++aCall)
  {
    std::size_t aLength = 0;
    int64_t     aRead   = 0;
    const char* aLine   = aBuffer.ReadLine(aStream, aLength, aRead);
    aText += (aText.empty() ? "" : " ") + (aLine == nullptr ? std::string("null") : tkernel_probe::hex(aLine, aLength))
             + ":" + std::to_string(aRead);
    if (aLine == nullptr)
      break;
    if (aClear != "-" && std::to_string(aCall + 1) == aClear)
      aBuffer.Clear();
  }
  return aText + " multi=" + std::to_string(aBuffer.IsMultilineMode())
         + " gap=" + std::to_string(aBuffer.ToPutGapInMultiline());
}
} // namespace standard_services_probe
