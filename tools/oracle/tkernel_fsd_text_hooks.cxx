// Derived-class oracle for FSD_File's protected text primitives and Cmp overrides.
#include <FSD_CmpFile.hxx>
#include <FSD_File.hxx>
#include <TCollection_AsciiString.hxx>
#include <TCollection_ExtendedString.hxx>
#include <fstream>
#include <iomanip>
#include <iostream>

class ExposedFsdFile : public FSD_File
{
public:
  using FSD_File::FindTag;
  using FSD_File::FlushEndOfLine;
  using FSD_File::ReadChar;
  using FSD_File::ReadExtendedLine;
  using FSD_File::ReadLine;
  using FSD_File::ReadString;
  using FSD_File::ReadWord;
  using FSD_File::WriteExtendedLine;
};

class ExposedCmpFile : public FSD_CmpFile
{
public:
  using FSD_CmpFile::FindTag;
  using FSD_CmpFile::FlushEndOfLine;
  using FSD_CmpFile::ReadExtendedLine;
  using FSD_CmpFile::ReadLine;
  using FSD_CmpFile::ReadString;
  using FSD_CmpFile::WriteExtendedLine;
};

static void fixture(const char* thePath, const std::string& theBytes)
{
  std::ofstream aFile(thePath, std::ios::binary);
  aFile.write(theBytes.data(), static_cast<std::streamsize>(theBytes.size()));
}

static void ascii(const char* theName, const TCollection_AsciiString& theText)
{
  const unsigned char* aBytes = reinterpret_cast<const unsigned char*>(theText.ToCString());
  std::cout << theName << '=' << std::hex << std::setfill('0');
  for (std::size_t anIndex = 0; anIndex < theText.Length(); ++anIndex)
    std::cout << std::setw(2) << static_cast<unsigned int>(aBytes[anIndex]);
  std::cout << std::dec << '\n';
}

static void extended(const char* theName, const TCollection_ExtendedString& theText)
{
  std::cout << theName << '=';
  for (int anIndex = 1; anIndex <= theText.Length(); ++anIndex)
    std::cout << static_cast<unsigned int>(theText.Value(anIndex)) << ',';
  std::cout << '\n';
}

static void file_hex(const char* theName, const char* thePath)
{
  std::ifstream aFile(thePath, std::ios::binary);
  std::cout << theName << '=' << std::hex << std::setfill('0');
  for (char aByte = 0; aFile.get(aByte);)
    std::cout << std::setw(2) << static_cast<unsigned int>(static_cast<unsigned char>(aByte));
  std::cout << std::dec << '\n';
}

static void run_base_readers(const char* thePath)
{
  fixture(thePath, "abc\n  alpha beta\r\n  value\r\n");
  ExposedFsdFile aFile;
  aFile.Open(TCollection_AsciiString(thePath), Storage_VSRead);
  TCollection_AsciiString aText;
  aFile.ReadChar(aText, 3);
  ascii("char", aText);
  aFile.ReadChar(aText, 1);
  ascii("newline", aText);
  aFile.ReadWord(aText);
  ascii("word", aText);
  aFile.ReadLine(aText);
  ascii("line", aText);
  aFile.ReadString(aText);
  ascii("string", aText);
}

static void run_flush_and_find(const char* thePath)
{
  fixture(thePath, "discard\r\nkeep\r\n");
  ExposedCmpFile aFile;
  aFile.Open(TCollection_AsciiString(thePath), Storage_VSRead);
  aFile.FlushEndOfLine();
  TCollection_AsciiString aText;
  aFile.ReadLine(aText);
  ascii("flushed_line", aText);
  aFile.Close();
  fixture(thePath, " skip\r\n BEGIN_INFO_SECTION\r\nbody\r\n");
  aFile.Open(TCollection_AsciiString(thePath), Storage_VSRead);
  std::cout << "tag=" << static_cast<int>(aFile.FindTag("BEGIN_INFO_SECTION\0ignored")) << '\n';
  aFile.ReadLine(aText);
  ascii("after_tag", aText);
}

static void run_extended(const char* thePath)
{
  fixture(thePath, std::string("\0A\0B\0\n", 6));
  ExposedFsdFile aReader;
  aReader.Open(TCollection_AsciiString(thePath), Storage_VSRead);
  TCollection_ExtendedString aText("old");
  aReader.ReadExtendedLine(aText);
  extended("base_extended", aText);
  aReader.Close();
  ExposedFsdFile aWriter;
  aWriter.Open(TCollection_AsciiString(thePath), Storage_VSWrite);
  aWriter.WriteExtendedLine(TCollection_ExtendedString("AB"));
  aWriter.Close();
  file_hex("base_write", thePath);
}

static void run_cmp_overrides(const char* thePath)
{
  fixture(thePath, "one\r\n  two\r\n");
  ExposedCmpFile aReader;
  aReader.Open(TCollection_AsciiString(thePath), Storage_VSRead);
  TCollection_AsciiString anAscii;
  aReader.ReadLine(anAscii);
  ascii("cmp_line", anAscii);
  aReader.ReadString(anAscii);
  ascii("cmp_string", anAscii);
  aReader.Close();
  fixture(thePath, "2 65 66 \ntail\r\n");
  aReader.Open(TCollection_AsciiString(thePath), Storage_VSRead);
  TCollection_ExtendedString anExtended;
  aReader.ReadExtendedLine(anExtended);
  extended("cmp_extended", anExtended);
  aReader.ReadLine(anAscii);
  ascii("cmp_after_extended", anAscii);
}

static void run_cmp_write(const char* thePath)
{
  ExposedCmpFile aWriter;
  aWriter.Open(TCollection_AsciiString(thePath), Storage_VSWrite);
  aWriter.WriteExtendedLine(TCollection_ExtendedString("AB"));
  aWriter.Close();
  file_hex("cmp_write", thePath);
}

int main()
{
  const char* aPath = "/tmp/rustycad-fsd-text-hooks.dat";
  run_base_readers(aPath);
  run_flush_and_find(aPath);
  run_extended(aPath);
  run_cmp_overrides(aPath);
  run_cmp_write(aPath);
  std::remove(aPath);
}
