// Standard_ArrayStreamBuffer driven through the public std::streambuf interface.
// Included by tkernel_standard_services_probe.cxx before its case table.
#include <Standard_ArrayStreamBuffer.hxx>

namespace standard_services_probe
{
static std::string stream_byte(const int theByte)
{
  return theByte == std::char_traits<char>::eof() ? "eof" : std::to_string(theByte);
}

// One token: g<n> sgetn, c sgetc, b sbumpc, u sungetc, p<byte> sputbackc, a in_avail,
// B<n>/C<n>/E<n> pubseekoff from begin/current/end, P<n> pubseekpos, i<hex> Init.
static std::string array_stream_step(Standard_ArrayStreamBuffer& theBuffer,
                                     std::string&               theData,
                                     const std::string&         theToken)
{
  const char        anOp   = theToken[0];
  const std::string anArgs = theToken.substr(1);
  if (anOp == 'g')
  {
    std::string aRead(std::stoul(anArgs), '\0');
    aRead.resize(static_cast<std::size_t>(theBuffer.sgetn(aRead.data(), aRead.size())));
    return tkernel_probe::hex(aRead);
  }
  if (anOp == 'c')
    return stream_byte(theBuffer.sgetc());
  if (anOp == 'b')
    return stream_byte(theBuffer.sbumpc());
  if (anOp == 'u')
    return stream_byte(theBuffer.sungetc());
  if (anOp == 'p')
    return stream_byte(theBuffer.sputbackc(static_cast<char>(std::stoi(anArgs))));
  if (anOp == 'a')
    return std::to_string(theBuffer.in_avail());
  if (anOp == 'i')
  {
    theData = tkernel_probe::decode(anArgs);
    theBuffer.Init(theData.data(), theData.size());
    return "-";
  }
  const std::streamoff anOffset = std::stoll(anArgs);
  if (anOp == 'P')
    return std::to_string(static_cast<std::streamoff>(theBuffer.pubseekpos(anOffset)));
  const std::ios_base::seekdir aWay =
    anOp == 'B' ? std::ios_base::beg : (anOp == 'C' ? std::ios_base::cur : std::ios_base::end);
  return std::to_string(static_cast<std::streamoff>(theBuffer.pubseekoff(anOffset, aWay)));
}

// "services_array_stream <hex bytes> <token>...": one answer per token.
static std::string array_stream_case(std::istream& theInput)
{
  std::string                aData = tkernel_probe::next_bytes(theInput);
  Standard_ArrayStreamBuffer aBuffer(aData.data(), aData.size());
  std::string                aResult, aToken;
  while (theInput >> aToken)
    aResult += (aResult.empty() ? "" : " ") + array_stream_step(aBuffer, aData, aToken);
  return aResult;
}
} // namespace standard_services_probe
