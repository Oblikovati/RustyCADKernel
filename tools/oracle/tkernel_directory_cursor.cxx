// Demand-driven readdir lifecycle for pinned OCCT's POSIX OSD iterators.
#include <OSD_FileIterator.hxx>
#include <OSD_Path.hxx>
#include <TCollection_AsciiString.hxx>
#include <dirent.h>
#include <dlfcn.h>
#include <fcntl.h>
#include <unistd.h>
#include <cstdlib>
#include <iostream>
#include <string>

static int opened = 0, reads = 0, closed = 0;
static DIR* lastStream = nullptr;
extern "C" DIR* opendir(const char* path) {
  using Function = DIR* (*)(const char*);
  static Function original = reinterpret_cast<Function>(dlsym(RTLD_NEXT, "opendir"));
  ++opened;
  lastStream = original(path);
  return lastStream;
}
extern "C" dirent* readdir(DIR* stream) {
  using Function = dirent* (*)(DIR*);
  static Function original = reinterpret_cast<Function>(dlsym(RTLD_NEXT, "readdir"));
  ++reads;
  return original(stream);
}
extern "C" int closedir(DIR* stream) {
  using Function = int (*)(DIR*);
  static Function original = reinterpret_cast<Function>(dlsym(RTLD_NEXT, "closedir"));
  ++closed;
  return original(stream);
}

int main() {
  char name[] = "/tmp/occt-directory-cursor-XXXXXX";
  const char* directory = mkdtemp(name);
  if (!directory) return 2;
  const std::string first = std::string(directory) + "/a";
  const std::string second = std::string(directory) + "/b";
  close(creat(first.c_str(), 0600));
  close(creat(second.c_str(), 0600));
  OSD_Path path{TCollection_AsciiString(directory)};
  OSD_FileIterator files(path, "*");
  std::cout << "lazy " << (opened == 0) << ' ' << files.More();
  const int firstReads = reads;
  std::cout << ' ' << (opened == 1 && closed == 0 && reads < 5);
  files.More();
  std::cout << ' ' << (reads == firstReads);
  files.Next();
  std::cout << ' ' << (files.More() && reads > firstReads && closed == 0);
  files.Next();
  std::cout << ' ' << (!files.More() && closed == 1) << '\n';
  files.Initialize(path, "*");
  std::cout << "restart " << (opened == 1) << ' ' << files.More();
  files.Destroy();
  std::cout << ' ' << (closed == 1);
  files.Initialize(path, "*");
  std::cout << ' ' << (closed == 2 && opened == 2) << '\n';
  const int beforeDrop = closed;
  { OSD_FileIterator early(path, "*"); early.More(); }
  std::cout << "drop_keeps_native_stream " << (closed == beforeDrop) << '\n';
  closedir(lastStream); // Clean up the pinned Unix destructor leak after observing it.
  unlink(first.c_str());
  unlink(second.c_str());
  rmdir(directory);
}
