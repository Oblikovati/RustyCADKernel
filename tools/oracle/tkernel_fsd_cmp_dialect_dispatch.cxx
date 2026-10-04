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
  aReader.directReadString(aLine);
  std::cout << aLine.ToCString() << '\n';
  TCollection_ExtendedString anExtended;
  aReader.directReadExtendedLine(anExtended);
  std::cout << aReader.flushCalls << '\n';
  aReader.directReadLine(aLine);
  std::cout << aLine.ToCString() << '\n';
  aReader.Close();

  DerivedCmpFile aWriter;
  aWriter.Open(aWritePath, Storage_VSWrite);
  aWriter.directWriteExtendedLine(TCollection_ExtendedString(u"AB"));
  aWriter.Close();
  std::ifstream aOutput(aWritePath, std::ios::binary);
  std::cout << aOutput.rdbuf();
  aOutput.close();
  std::remove(aReadPath);
  std::remove(aWritePath);
}
