#include <FSD_File.hxx>
#include <Storage_Error.hxx>
#include <Storage_OpenMode.hxx>
#include <TCollection_AsciiString.hxx>

#include <fstream>
#include <iostream>

static int GlobalReadCharCalls = 0;

class StatefulFile final : public FSD_File
{
public:
  int readCharCalls = 0;
  int readWordCalls = 0;
  int readStringCalls = 0;
  int readLineCalls = 0;
  int flushCalls = 0;
  int findTagCalls = 0;

  void directReadChar(TCollection_AsciiString& theBuffer, size_t theSize)
  {
    ReadChar(theBuffer, theSize);
  }

  void directReadWord(TCollection_AsciiString& theBuffer) { ReadWord(theBuffer); }
  void directFlushEndOfLine() { FlushEndOfLine(); }
  Storage_Error directFindTag(const char* theTag) { return FindTag(theTag); }

protected:
  void ReadChar(TCollection_AsciiString& theBuffer, size_t theSize) override
  {
    ++GlobalReadCharCalls;
    ++readCharCalls;
    FSD_File::ReadChar(theBuffer, theSize);
  }

  void ReadWord(TCollection_AsciiString& theBuffer) override
  {
    ++readWordCalls;
    theBuffer = TCollection_AsciiString("dialect-word");
  }

  void ReadString(TCollection_AsciiString& theBuffer) override
  {
    ++readStringCalls;
    FSD_File::ReadString(theBuffer);
  }

  void ReadLine(TCollection_AsciiString& theBuffer) override
  {
    ++readLineCalls;
    FSD_File::ReadLine(theBuffer);
  }

  void FlushEndOfLine() override
  {
    ++flushCalls;
    FSD_File::FlushEndOfLine();
  }

  Storage_Error FindTag(const char* theTag) override
  {
    ++findTagCalls;
    return FSD_File::FindTag(theTag);
  }
};

int main()
{
  const char* aPath = "/tmp/tkernel-fsd-text-dialect-dispatch.txt";
  {
    std::ofstream aFile(aPath);
    aFile << "FSDFILE\nBEGIN_INFO_SECTION\n1\n2 ignored\n";
  }

  const int aCallsBeforeStaticProbe = GlobalReadCharCalls;
  const auto aStaticStatus = StatefulFile::IsGoodFileType(aPath);
  std::cout << (aStaticStatus == Storage_VSOk) << ' '
            << (GlobalReadCharCalls == aCallsBeforeStaticProbe) << '\n';

  StatefulFile aFile;
  std::cout << (aFile.Open(aPath, Storage_VSRead) == Storage_VSOk) << '\n';
  std::cout << (aFile.BeginReadInfoSection() == Storage_VSOk) << '\n';
  std::cout << aFile.readCharCalls << ' ' << aFile.findTagCalls << ' ' << aFile.readStringCalls
            << '\n';
  std::cout << aFile.RootSectionSize() << '\n';
  std::cout << aFile.flushCalls << ' ' << aFile.readLineCalls << '\n';

  int                 aReference = 0;
  TCollection_AsciiString aName, aType;
  aFile.ReadRoot(aName, aReference, aType);
  std::cout << aFile.readWordCalls << ' ' << aName.ToCString() << ' ' << aType.ToCString() << '\n';

  TCollection_AsciiString aCharacter;
  aFile.directReadChar(aCharacter, 1);
  aFile.directReadWord(aName);
  std::cout << aFile.readCharCalls << ' ' << aCharacter.Length() << ' ' << aFile.readWordCalls
            << ' ' << aName.ToCString() << '\n';
  aFile.directFlushEndOfLine();
  std::cout << aFile.flushCalls << ' ' << aFile.readLineCalls << '\n';
  std::cout << (aFile.directFindTag("absent") == Storage_VSSectionNotFound) << ' '
            << aFile.findTagCalls << ' ' << aFile.readStringCalls << '\n';
  aFile.Close();
  std::remove(aPath);
}
