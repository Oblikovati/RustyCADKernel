// Supplemental protected string hooks for OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <FSD_BinaryFile.hxx>
#include <Standard_Failure.hxx>
#include <fstream>
#include <iostream>
#include <sstream>
#include <vector>

struct BinaryStringProbe : FSD_BinaryFile {
  using FSD_BinaryFile::ReadChar;
  using FSD_BinaryFile::ReadString;
  using FSD_BinaryFile::ReadExtendedString;
  using FSD_BinaryFile::WriteString;
  using FSD_BinaryFile::WriteExtendedString;
};

static void write_payload(const char* path, int size, const std::vector<char>& bytes)
{
  std::ofstream file(path, std::ios::binary | std::ios::trunc);
  file.write(reinterpret_cast<const char*>(&size), sizeof(size));
  file.write(bytes.data(), bytes.size());
}

static void read_ascii(const char* path, int size, const std::vector<char>& bytes)
{
  write_payload(path, size, bytes);
  BinaryStringProbe driver;
  driver.Open(path, Storage_VSRead);
  TCollection_AsciiString text("keep");
  try { driver.ReadString(text); std::cout << "ok"; }
  catch (const Standard_Failure& error) { std::cout << error.ExceptionType(); }
  std::cout << ' ' << text.Length() << ' ' << text.ToCString() << ' ' << driver.Tell()
            << ' ' << driver.IsEnd() << '\n';
  driver.Close();
}

static void read_extended(const char* path, int size, const std::vector<char>& bytes)
{
  write_payload(path, size, bytes);
  BinaryStringProbe driver;
  driver.Open(path, Storage_VSRead);
  TCollection_ExtendedString text("keep");
  try { driver.ReadExtendedString(text); std::cout << "ok"; }
  catch (const Standard_Failure& error) { std::cout << error.ExceptionType(); }
  std::cout << ' ' << text.Length();
  for (int index = 1; index <= text.Length(); ++index) std::cout << ' ' << int(text.Value(index));
  std::cout << ' ' << driver.Tell() << ' ' << driver.IsEnd() << '\n';
  driver.Close();
}

static void write_and_read_chars(const char* path)
{
  BinaryStringProbe driver;
  driver.Open(path, Storage_VSWrite);
  driver.WriteString(TCollection_AsciiString("abc"));
  driver.WriteExtendedString(TCollection_ExtendedString(u"\u20ac\U0001f600"));
  std::cout << "written " << driver.Tell() << '\n';
  driver.Close();
  driver.Open(path, Storage_VSRead);
  TCollection_AsciiString ascii;
  TCollection_ExtendedString extended;
  driver.ReadString(ascii);
  driver.ReadExtendedString(extended);
  std::cout << ascii.ToCString() << ' ' << extended.Length() << ' ' << driver.Tell() << '\n';
  driver.Close();
}

static void char_eof(const char* path)
{
  { std::ofstream file(path, std::ios::binary | std::ios::trunc); file << "abc"; }
  BinaryStringProbe driver;
  driver.Open(path, Storage_VSRead);
  TCollection_AsciiString text;
  driver.ReadChar(text, 5);
  std::cout << text.ToCString() << ' ' << driver.Tell() << ' ' << driver.IsEnd() << '\n';
  driver.ReadChar(text, 1);
  std::cout << text.Length() << '\n';
  driver.Close();
}

static void stream_counts()
{
  std::ostringstream stream;
  const TCollection_AsciiString ascii("abc");
  const TCollection_ExtendedString extended(u"\u20ac\U0001f600");
  std::cout << BinaryStringProbe::WriteString(stream, ascii, true) << ' '
            << BinaryStringProbe::WriteExtendedString(stream, extended, true) << ' '
            << stream.str().size() << ' '
            << BinaryStringProbe::WriteString(stream, ascii, false) << ' '
            << BinaryStringProbe::WriteExtendedString(stream, extended, false) << ' '
            << stream.str().size() << '\n';
}

int main(int argc, char** argv)
{
  if (argc != 2) return 2;
  read_ascii(argv[1], 3, {'a', '\0', 'b'});
  read_ascii(argv[1], 0, {});
  read_ascii(argv[1], -1, {});
  read_ascii(argv[1], 5, {'a', 'b'});
  const char16_t units[] = {0x20ac, 0, 0x41};
  const char* bytes = reinterpret_cast<const char*>(units);
  read_extended(argv[1], 3, std::vector<char>(bytes, bytes + sizeof(units)));
  read_extended(argv[1], 0, {});
  read_extended(argv[1], -1, {});
  read_extended(argv[1], 3, {'a'});
  write_and_read_chars(argv[1]);
  char_eof(argv[1]);
  stream_counts();
}
