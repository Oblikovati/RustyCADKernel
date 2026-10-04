// Derived-class access oracle for Standard_ReadLineBuffer's protected istream hook.
#include <Standard_ReadLineBuffer.hxx>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <streambuf>
#include <string>

class ExposedReadLineBuffer : public Standard_ReadLineBuffer
{
public:
  using Standard_ReadLineBuffer::Standard_ReadLineBuffer;
  bool Read(std::istream& theStream, std::size_t theLength, std::size_t& theCount)
  {
    return readStream(theStream, theLength, theCount);
  }
  std::string Bytes(std::size_t theLength) const
  {
    return std::string(myReadBuffer.Data(), theLength);
  }
};

class ErrorAfterBytes : public std::streambuf
{
public:
  ErrorAfterBytes(std::string theBytes, std::size_t theFailAfter)
      : myBytes(std::move(theBytes)), myFailAfter(theFailAfter)
  {
  }

protected:
  int_type underflow() override
  {
    if (myPosition >= myFailAfter)
      throw std::runtime_error("injected stream failure");
    if (myPosition >= myBytes.size())
      return traits_type::eof();
    return traits_type::to_int_type(myBytes[myPosition]);
  }
  int_type uflow() override
  {
    const int_type aByte = underflow();
    if (!traits_type::eq_int_type(aByte, traits_type::eof()))
      ++myPosition;
    return aByte;
  }

private:
  std::string myBytes;
  std::size_t myFailAfter;
  std::size_t myPosition = 0;
};

static std::string hex(const std::string& theBytes)
{
  std::ostringstream aResult;
  for (unsigned char aByte : theBytes)
    aResult << std::hex << std::setw(2) << std::setfill('0') << static_cast<int>(aByte);
  return aResult.str();
}

static void read_and_print(ExposedReadLineBuffer& theBuffer, std::istream& theStream)
{
  std::size_t aCount = 999;
  const bool aGood = theBuffer.Read(theStream, 4, aCount);
  std::cout << "good=" << aGood << " count=" << aCount << " bytes=" << hex(theBuffer.Bytes(4))
            << '\n';
}

int main()
{
  ExposedReadLineBuffer aBuffer(4);
  std::istringstream aNormal("abcdef");
  read_and_print(aBuffer, aNormal);
  read_and_print(aBuffer, aNormal);
  read_and_print(aBuffer, aNormal);
  ExposedReadLineBuffer anErrorBuffer(4);
  std::istringstream aSeed("old!");
  read_and_print(anErrorBuffer, aSeed);
  ErrorAfterBytes aFailure("", 0);
  std::istream anError(&aFailure);
  read_and_print(anErrorBuffer, anError);
}
