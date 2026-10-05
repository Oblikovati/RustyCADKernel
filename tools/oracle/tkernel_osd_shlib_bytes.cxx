#include <OSD_LoadMode.hxx>
#include <OSD_SharedLibrary.hxx>

#include <cstdio>
#include <cstring>
#include <iostream>
#include <cstdlib>
#include <string>
#include <unistd.h>

std::string hexBytes(const char* bytes)
{
  static const char digits[] = "0123456789abcdef";
  std::string result;
  for (const unsigned char* cursor = reinterpret_cast<const unsigned char*>(bytes);
       *cursor != 0; ++cursor)
  {
    result.push_back(digits[*cursor >> 4]);
    result.push_back(digits[*cursor & 0x0f]);
  }
  return result;
}

int main(int argc, char** argv)
{
  if (argc != 2)
  {
    return 2;
  }
  char directory[] = "/tmp/rustycad-shlib-bytes-XXXXXX";
  if (mkdtemp(directory) == nullptr)
  {
    return 3;
  }
  const std::string path = std::string(directory) + "/lib-\xff.so";
  if (symlink(argv[1], path.c_str()) != 0)
  {
    rmdir(directory);
    return 4;
  }
  OSD_SharedLibrary library(path.c_str());
  const bool sameName = std::strcmp(library.Name(), path.c_str()) == 0;
  const bool opened = library.DlOpen(OSD_RTLD_NOW);
  const char missingSymbol[] = "missing_\xff_symbol";
  const bool found = library.DlSymb(missingSymbol) != nullptr;
  const char* error = library.DlError();
  std::cout << "same_name=" << sameName << " open=" << opened << " symbol=" << found
            << " error_hex=" << (error == nullptr ? "none" : hexBytes(error)) << '\n';
  library.DlClose();
  unlink(path.c_str());
  rmdir(directory);
}
