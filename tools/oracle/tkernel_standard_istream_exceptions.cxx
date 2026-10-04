#include <Standard_ReadLineBuffer.hxx>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <streambuf>
#include <string>

class ExposedLineBuffer : public Standard_ReadLineBuffer
{
public:
  using Standard_ReadLineBuffer::Standard_ReadLineBuffer;
  bool Read(std::istream& theStream, std::size_t theLength, std::size_t& theCount)
  { return readStream(theStream, theLength, theCount); }
  std::string Bytes(std::size_t theLength) const
  { return std::string(myReadBuffer.Data(), theLength); }
  std::size_t LastRead() const { return myBytesLastRead; }
  std::size_t Position() const { return myBufferPos; }
  void SeedState()
  {
    for (std::size_t anIndex = 0; anIndex < 4; ++anIndex) myReadBuffer[anIndex] = "prev"[anIndex];
    myBytesLastRead = 4;
    myBufferPos = 4;
  }
  const char* ReadLine(std::istream& theStream, std::size_t& theLength, std::int64_t& theCount)
  { return Standard_ReadLineBuffer::ReadLine(theStream, theLength, theCount); }
};

class ThrowAfterBytes : public std::streambuf
{
public:
  ThrowAfterBytes(std::string theBytes, std::size_t theLimit)
    : myBytes(std::move(theBytes)), myLimit(theLimit) {}
protected:
  int_type underflow() override
  {
    if (myPosition >= myLimit) throw std::runtime_error("injected streambuf failure");
    if (myPosition >= myBytes.size()) return traits_type::eof();
    return traits_type::to_int_type(myBytes[myPosition]);
  }
  int_type uflow() override
  {
    int_type aValue = underflow();
    if (!traits_type::eq_int_type(aValue, traits_type::eof())) ++myPosition;
    return aValue;
  }
private:
  std::string myBytes;
  std::size_t myLimit;
  std::size_t myPosition = 0;
};

static std::string hex(const std::string& theBytes)
{
  std::ostringstream aOut;
  for (unsigned char aByte : theBytes)
    aOut << std::hex << std::setw(2) << std::setfill('0') << int(aByte);
  return aOut.str();
}

static void probe_bad_read(ExposedLineBuffer& theBuffer)
{
  ThrowAfterBytes aSource("ab", 2);
  std::istream aStream(&aSource);
  aStream.exceptions(std::ios::badbit);
  std::size_t aCount = 77;
  bool didThrow = false;
  try { theBuffer.Read(aStream, 4, aCount); }
  catch (const std::exception&) { didThrow = true; }
  std::cout << "badread=" << aCount << '/' << aStream.gcount() << '/' << aStream.bad()
            << '/' << aStream.fail() << '/' << didThrow << '/' << hex(theBuffer.Bytes(4)) << '\n';
}

static void probe_unmasked_read(ExposedLineBuffer& theBuffer)
{
  ThrowAfterBytes anUnmaskedSource("cd", 2);
  std::istream anUnmaskedStream(&anUnmaskedSource);
  std::size_t aCount = 66;
  const bool isUnmaskedGood = theBuffer.Read(anUnmaskedStream, 4, aCount);
  std::cout << "unmasked=" << isUnmaskedGood << '/' << aCount << '/' << anUnmaskedStream.gcount()
            << '/' << anUnmaskedStream.bad() << '/' << anUnmaskedStream.fail() << "/0/"
            << hex(theBuffer.Bytes(4)) << '\n';
}

static void probe_sentry_mask(ExposedLineBuffer& theBuffer)
{
  ThrowAfterBytes aFailMaskSource("rs", 2);
  std::istream aFailMaskStream(&aFailMaskSource);
  aFailMaskStream.exceptions(std::ios::failbit);
  std::size_t aCount = 44;
  const bool isFirstReadGood = theBuffer.Read(aFailMaskStream, 4, aCount);
  bool didThrow = false;
  aCount = 45;
  try { theBuffer.Read(aFailMaskStream, 4, aCount); }
  catch (const std::ios_base::failure&) { didThrow = true; }
  std::cout << "sentry=" << isFirstReadGood << '/' << aCount << '/'
            << int(aFailMaskStream.rdstate()) << '/' << aFailMaskStream.gcount() << '/' << didThrow
            << '\n';
}

static void probe_short_read(ExposedLineBuffer& theBuffer)
{
  std::istringstream aShort("xy");
  aShort.exceptions(std::ios::failbit);
  std::size_t aCount = 88;
  bool didThrow = false;
  try { theBuffer.Read(aShort, 4, aCount); }
  catch (const std::exception&) { didThrow = true; }
  std::cout << "shortread=" << aCount << '/' << aShort.gcount() << '/' << aShort.eof()
            << '/' << aShort.fail() << '/' << aShort.bad() << '/' << didThrow << '/'
            << hex(theBuffer.Bytes(4)) << '\n';
}

static void probe_mask_changes()
{
  std::istringstream aMask("m");
  aMask.setstate(std::ios::failbit);
  bool didThrow = false;
  try { aMask.exceptions(std::ios::failbit); }
  catch (const std::ios_base::failure&) { didThrow = true; }
  std::cout << "setmask=" << aMask.fail() << '/' << int(aMask.exceptions()) << '/' << didThrow << '\n';
  try { aMask.clear(); } catch (...) { std::cout << "clearthrew "; }
  std::cout << "clear=" << aMask.good() << '/' << int(aMask.exceptions()) << '\n';

  std::istringstream aClearState("q");
  aClearState.exceptions(std::ios::failbit);
  didThrow = false;
  try { aClearState.clear(std::ios::failbit); }
  catch (const std::ios_base::failure&) { didThrow = true; }
  std::cout << "clearstate=" << aClearState.fail() << '/' << int(aClearState.exceptions()) << '/'
            << didThrow << '\n';
}

static void probe_zero_length_failure()
{
  std::istringstream aZeroFailure("z");
  aZeroFailure.exceptions(std::ios::badbit);
  try { aZeroFailure.setstate(std::ios::badbit); } catch (const std::ios_base::failure&) {}
  bool didThrow = false;
  char aZeroByte = 0;
  try { aZeroFailure.read(&aZeroByte, 0); }
  catch (const std::ios_base::failure&) { didThrow = true; }
  std::cout << "zeroerror=" << int(aZeroFailure.rdstate()) << '/' << aZeroFailure.gcount() << '/'
            << didThrow << '\n';
}

static void probe_line_state()
{
  ExposedLineBuffer aLine(4);
  ThrowAfterBytes aLineSource("xy", 2);
  std::istream aLineStream(&aLineSource);
  aLineStream.exceptions(std::ios::badbit);
  std::size_t aLineLength = 55;
  std::int64_t aReadData = 66;
  aLine.SeedState();
  bool didThrow = false;
  try { aLine.ReadLine(aLineStream, aLineLength, aReadData); }
  catch (const std::exception&) { didThrow = true; }
  std::cout << "line=" << aLineLength << '/' << aReadData << '/' << aLine.LastRead()
            << '/' << aLine.Position() << '/' << hex(aLine.Bytes(4)) << '/' << aLineStream.gcount()
            << '/' << didThrow << '\n';
}

int main()
{
  std::cout << std::unitbuf;
  ExposedLineBuffer aBuffer(4);
  probe_bad_read(aBuffer);
  probe_unmasked_read(aBuffer);
  probe_sentry_mask(aBuffer);
  probe_short_read(aBuffer);
  probe_mask_changes();
  probe_zero_length_failure();
  probe_line_state();
}
