#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#include <Standard_ReadLineBuffer.hxx>

#include <cerrno>
#include <cstdio>
#include <cstring>
#include <iostream>
#include <string>

class ExposedReadLineBuffer : public Standard_ReadLineBuffer
{
public:
  using Standard_ReadLineBuffer::Standard_ReadLineBuffer;
  bool Read(FILE* theStream, std::size_t theLength, std::size_t& theCount)
  {
    return readStream(theStream, theLength, theCount);
  }
  std::string Bytes(std::size_t theLength) const
  {
    return std::string(myReadBuffer.Data(), theLength);
  }
};

struct ScriptedFile
{
  int calls = 0;
  int dataPosition = 0;
};

static ssize_t scripted_read(void* theCookie, char* theBuffer, std::size_t theLength)
{
  ScriptedFile& aScript = *static_cast<ScriptedFile*>(theCookie);
  ++aScript.calls;
  if (aScript.calls == 1)
  {
    theBuffer[0] = 'a';
    ++aScript.dataPosition;
    return 1;
  }
  if (aScript.calls == 2)
  {
    errno = EIO;
    return -1;
  }
  if (aScript.dataPosition == 5)
    return 0;
  theBuffer[0] = "abcdef"[aScript.dataPosition++];
  return 1;
}

static void report(ExposedReadLineBuffer& theBuffer, FILE* theStream, int theCall)
{
  std::size_t aCount = 99;
  const bool isGood = theBuffer.Read(theStream, 4, aCount);
  std::cout << "call=" << theCall << " good=" << isGood << " count=" << aCount
            << " error=" << std::ferror(theStream) << " eof=" << std::feof(theStream)
            << " bytes=";
  for (unsigned char aByte : theBuffer.Bytes(4))
    std::cout << std::hex << static_cast<int>(aByte) << ',';
  std::cout << std::dec << '\n';
}

int main()
{
  ScriptedFile aScript;
  cookie_io_functions_t aFunctions{};
  aFunctions.read = scripted_read;
  FILE* aStream = fopencookie(&aScript, "r", aFunctions);
  std::setvbuf(aStream, nullptr, _IONBF, 0);
  ExposedReadLineBuffer aBuffer(4);
  report(aBuffer, aStream, 1);
  report(aBuffer, aStream, 2);
  report(aBuffer, aStream, 3);
  report(aBuffer, aStream, 4);
  std::cout << "calls=" << aScript.calls << '\n';
  std::fclose(aStream);
}
