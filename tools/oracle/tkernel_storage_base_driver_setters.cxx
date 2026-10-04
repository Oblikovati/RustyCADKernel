#include <FSD_File.hxx>
#include <Storage_BaseDriver.hxx>
#include <Storage_OpenMode.hxx>
#include <Storage_Error.hxx>
#include <TCollection_AsciiString.hxx>

#include <iostream>

class ExposedFile final : public FSD_File
{
public:
  void assignName(const TCollection_AsciiString& theName) { SetName(theName); }
  void assignMode(const Storage_OpenMode theMode) { SetOpenMode(theMode); }
};

int main()
{
  ExposedFile aFile;
  aFile.assignName("before");
  aFile.assignMode(Storage_VSRead);
  std::cout << aFile.Name().ToCString() << '\n';
  std::cout << (aFile.OpenMode() == Storage_VSRead) << '\n';
  std::cout << (aFile.Open("request", Storage_VSWrite) == Storage_VSAlreadyOpen) << '\n';
  std::cout << aFile.Name().ToCString() << '\n';
  std::cout << (aFile.Close() == Storage_VSOk) << '\n';
  std::cout << (aFile.OpenMode() == Storage_VSNone) << '\n';
  std::cout << aFile.Name().ToCString() << '\n';
}
