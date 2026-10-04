#include <cstring>
#include <Standard_ReadLineBuffer.hxx>

#include <iostream>
#include <sstream>

class ExposedLineBuffer : public Standard_ReadLineBuffer
{
public:
  using Standard_ReadLineBuffer::Standard_ReadLineBuffer;
  bool Read(std::istream& theStream, std::size_t theLength, std::size_t& theCount)
  {
    return readStream(theStream, theLength, theCount);
  }
};

int main()
{
  ExposedLineBuffer aBuffer(2);
  std::istringstream aBadStream("ok");
  aBadStream.setstate(std::ios::badbit);
  std::size_t aCount = 9;
  const bool isInitiallyBad = aBuffer.Read(aBadStream, 2, aCount);
  std::cout << "bad=" << isInitiallyBad << '/' << aCount << '/' << aBadStream.bad() << '/'
            << aBadStream.fail();
  aBadStream.clear();
  const bool isRecovered = aBuffer.Read(aBadStream, 2, aCount);
  std::cout << " recovered=" << isRecovered << '/' << aCount << '/' << aBadStream.good();

  std::istringstream anEofStream("x");
  const bool isShort = aBuffer.Read(anEofStream, 2, aCount);
  std::cout << " short=" << isShort << '/' << aCount << '/' << anEofStream.eof() << '/'
            << anEofStream.fail() << '/' << anEofStream.bad();
  const bool isRepeated = aBuffer.Read(anEofStream, 2, aCount);
  std::cout << " repeated=" << isRepeated << '/' << aCount << '/' << anEofStream.bad() << '\n';

  std::istringstream aZeroLengthStream("z");
  const bool isZeroLength = aBuffer.Read(aZeroLengthStream, 0, aCount);
  std::cout << "zero=" << isZeroLength << '/' << aCount << '/' << aZeroLengthStream.good() << '/'
            << aZeroLengthStream.eof() << '/' << aZeroLengthStream.fail() << '/'
            << aZeroLengthStream.bad();
  std::istringstream anEofBitStream("z");
  anEofBitStream.setstate(std::ios::eofbit);
  const bool isEofBitOnly = aBuffer.Read(anEofBitStream, 1, aCount);
  std::cout << " eofbit=" << isEofBitOnly << '/' << aCount << '/' << anEofBitStream.eof() << '/'
            << anEofBitStream.fail() << '/' << anEofBitStream.bad() << '\n';
}
