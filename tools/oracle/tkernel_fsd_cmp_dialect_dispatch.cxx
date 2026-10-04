#include <FSD_CmpFile.hxx>
#include <Storage_OpenMode.hxx>
#include <TCollection_AsciiString.hxx>
#include <TCollection_ExtendedString.hxx>

#include <fstream>
#include <iostream>

class DerivedCmpFile final : public FSD_CmpFile
{
public:
  int flushCalls = 0;
  int lineCalls = 0;
  int stringCalls = 0;
  int extendedReadCalls = 0;
  int extendedWriteCalls = 0;

  void directReadLine(TCollection_AsciiString& theBuffer) { ReadLine(theBuffer); }
  void directReadString(TCollection_AsciiString& theBuffer) { ReadString(theBuffer); }
  void directReadExtendedLine(TCollection_ExtendedString& theBuffer)
  {
    ReadExtendedLine(theBuffer);
  }
  void directWriteExtendedLine(const TCollection_ExtendedString& theBuffer)
  {
    WriteExtendedLine(theBuffer);
  }

protected:
  void ReadLine(TCollection_AsciiString& theBuffer) override
  {
    ++lineCalls;
    FSD_CmpFile::ReadLine(theBuffer);
  }

  void ReadString(TCollection_AsciiString& theBuffer) override
  {
    ++stringCalls;
    FSD_CmpFile::ReadString(theBuffer);
  }

  void ReadExtendedLine(TCollection_ExtendedString& theBuffer) override
  {
    ++extendedReadCalls;
    FSD_CmpFile::ReadExtendedLine(theBuffer);
  }

  void WriteExtendedLine(const TCollection_ExtendedString& theBuffer) override
  {
    ++extendedWriteCalls;
    FSD_CmpFile::WriteExtendedLine(theBuffer);
  }

  void FlushEndOfLine() override
  {
    ++flushCalls;
    FSD_File::FlushEndOfLine();
  }
};

int main()
{
  const char* aReadPath = "/tmp/tkernel-fsd-cmp-dialect-read.txt";
  const char* aWritePath = "/tmp/tkernel-fsd-cmp-dialect-write.txt";
  {
    std::ofstream aInput(aReadPath, std::ios::binary);
    aInput << "one\r\n  two\r\n0\n  tail\r\n";
  }

  DerivedCmpFile aReader;
  std::cout << (aReader.Open(aReadPath, Storage_VSRead) == Storage_VSOk) << '\n';
  TCollection_AsciiString aLine;
  aReader.directReadLine(aLine);
  std::cout << aLine.ToCString() << '\n';
  std::cout << aReader.lineCalls << ' ' << aReader.stringCalls << '\n';
  aReader.directReadString(aLine);
  std::cout << aLine.ToCString() << '\n';
  TCollection_ExtendedString anExtended;
  aReader.directReadExtendedLine(anExtended);
  std::cout << aReader.flushCalls << '\n';
  std::cout << aReader.extendedReadCalls << ' ' << aReader.lineCalls << '\n';
  aReader.directReadLine(aLine);
  std::cout << aLine.ToCString() << '\n';
  aReader.Close();

  DerivedCmpFile aWriter;
  aWriter.Open(aWritePath, Storage_VSWrite);
  aWriter.directWriteExtendedLine(TCollection_ExtendedString(u"AB"));
  std::cout << aWriter.extendedWriteCalls << '\n';
  aWriter.Close();
  std::ifstream aOutput(aWritePath, std::ios::binary);
  std::cout << aOutput.rdbuf();
  aOutput.close();
  std::remove(aReadPath);
  std::remove(aWritePath);
}
