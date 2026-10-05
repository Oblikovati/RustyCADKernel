#include <OSD_LoadMode.hxx>
#include <OSD_SharedLibrary.hxx>

#include <cstring>
#include <iostream>

struct LibraryCases
{
  bool constructor;
  bool setName;
  bool open;
  bool plainSymbol;
  bool prefixSymbol;
  bool emptySymbol;
};

struct EmptyNameCase
{
  bool name;
  bool open;
  bool symbol;
};

EmptyNameCase inspectEmptyName()
{
  OSD_SharedLibrary library("\0libm.so.6");
  const bool opened = library.DlOpen(OSD_RTLD_NOW);
  const EmptyNameCase cases{library.Name() && library.Name()[0] == '\0', opened,
                            library.DlSymb("malloc") != nullptr};
  library.DlClose();
  return cases;
}

LibraryCases inspectLibrary()
{
  const char aPath[] = "libm.so.6\0ignored";
  OSD_SharedLibrary aLibrary(aPath);
  OSD_SharedLibrary aSetName;
  aSetName.SetName(aPath);
  const bool isOpen = aLibrary.DlOpen(OSD_RTLD_NOW);
  LibraryCases cases{std::strcmp(aLibrary.Name(), "libm.so.6") == 0,
                     std::strcmp(aSetName.Name(), "libm.so.6") == 0,
                     isOpen,
                     aLibrary.DlSymb("cos") != nullptr,
                     aLibrary.DlSymb("cos\0ignored") != nullptr,
                     aLibrary.DlSymb("\0cos") != nullptr};
  aLibrary.DlClose();
  return cases;
}

int main()
{
  const LibraryCases cases = inspectLibrary();
  const EmptyNameCase empty = inspectEmptyName();
  std::cout << "constructor=" << cases.constructor << " set_name=" << cases.setName
            << " open=" << cases.open << " plain_symbol=" << cases.plainSymbol
            << " symbol=" << cases.prefixSymbol << " empty_symbol=" << cases.emptySymbol
            << " empty=" << empty.name << " empty_open=" << empty.open
            << " empty_main_symbol=" << empty.symbol << '\n';
}
