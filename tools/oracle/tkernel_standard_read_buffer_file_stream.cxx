#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#include <cstring>
#include <Standard_ReadBuffer.hxx>
#include <Standard_ReadLineBuffer.hxx>

#include <cerrno>
#include <cstdio>
#include <cstring>
#include <iostream>
#include <string>

class ExposedLineBuffer : public Standard_ReadLineBuffer
{
public:
  using Standard_ReadLineBuffer::Standard_ReadLineBuffer;
  bool Read(FILE* theStream, std::size_t theLength, std::size_t& theCount)
  {
    return readStream(theStream, theLength, theCount);
  }
};

struct ScriptedFile
{
  std::size_t position = 0;
};

static ssize_t scripted_read(void* theCookie, char* theBuffer, std::size_t)
{
  ScriptedFile& aFile = *static_cast<ScriptedFile*>(theCookie);
  if (aFile.position == 0)
  {
    ++aFile.position;
    errno = EIO;
    return -1;
  }
  if (aFile.position > 4)
    return 0;
  theBuffer[0] = "xdata"[aFile.position++];
  return 1;
}

int main()
{
  ScriptedFile aFile;
  cookie_io_functions_t aFunctions{};
  aFunctions.read = scripted_read;
  FILE* aStream = fopencookie(&aFile, "r", aFunctions);
  std::setvbuf(aStream, nullptr, _IONBF, 0);
  ExposedLineBuffer aLineBuffer(1);
  std::size_t aCount = 9;
  const bool isLineGood = aLineBuffer.Read(aStream, 1, aCount);
  Standard_ReadBuffer aChunkBuffer(4, 4);
  const char* aChunk = aChunkBuffer.ReadDataChunk(aStream);
  std::cout << "line_good=" << isLineGood << " line_count=" << aCount
            << " chunk=" << (aChunk == nullptr ? "null" : std::string(aChunk, 4))
            << " error=" << std::ferror(aStream) << '\n';
  std::fclose(aStream);
}
