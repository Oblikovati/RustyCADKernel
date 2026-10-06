// Pinned OCCT protected Initialize; displaced native resources are cleaned by this harness only.
#include <Standard_MMgrOpt.hxx>
#include <algorithm>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <tuple>
#include <fcntl.h>
#include <unistd.h>

class InitializeProbe : public Standard_MMgrOpt
{
public:
  explicit InitializeProbe(bool clear = true, bool mapping = false)
    : Standard_MMgrOpt(clear, mapping, 17, 100, 512)
  {
    myPageSize = 64; // Same protected-field setup as Rust: small real pools for focused tests.
  }
  auto topology() const
  {
    return std::make_tuple(myAllocList, myNextAddr, myEndBlock);
  }
  bool repeat()
  {
    auto* retired = myFreeList;
    Initialize();
    const bool freshZero = retired != myFreeList &&
      std::all_of(myFreeList, myFreeList + myFreeListMax + 1, [](auto* p) { return !p; });
    std::free(retired); // Native forgets this allocation; cleanup occurs after observation only.
    return freshZero;
  }
  void normalize()
  {
    myCellSize = 17; myNbPages = 7;
    repeat();
    std::cout << "normalization " << myCellSize << ' ' << myNbPages << ' ' << myFreeListMax
              << ' ' << (myPageSize > 0) << ' ' << (myFreeList[0] == nullptr) << '\n';
  }
  int descriptor() const
  {
    return myMMap;
  }
};

void live_pool()
{
  InitializeProbe allocator;
  void* block = allocator.Allocate(16);
  std::memset(block, 19, 16);
  auto before = allocator.topology();
  allocator.repeat();
  const auto* bytes = static_cast<unsigned char*>(block);
  const bool preserved = std::all_of(bytes, bytes + 16, [](auto b) { return b == 19; });
  std::cout << "live-pool " << (before == allocator.topology()) << ' ' << preserved
            << ' ' << (static_cast<size_t*>(block)[-1] == 16) << '\n';
  allocator.Free(block);
}

void forgotten_small()
{
  InitializeProbe allocator;
  void* retired = allocator.Allocate(16);
  allocator.Free(retired);
  auto before = allocator.topology();
  allocator.repeat();
  const bool preserved = before == allocator.topology();
  void* next = allocator.Allocate(16);
  std::cout << "forgotten-small " << preserved << ' ' << (next != retired) << ' '
            << (static_cast<char*>(next) - static_cast<char*>(retired)) << '\n';
  allocator.Free(next);
  void* reused = allocator.Allocate(16);
  std::cout << "new-small-free-list " << (reused == next) << '\n';
  allocator.Free(reused);
}

void forgotten_medium()
{
  InitializeProbe allocator;
  void* live = allocator.Allocate(40);
  std::memset(live, 7, 40);
  void* retired = allocator.Allocate(48);
  allocator.Free(retired);
  allocator.repeat();
  const int purged = allocator.Purge(false);
  void* next = allocator.Allocate(48);
  const auto* bytes = static_cast<unsigned char*>(live);
  std::cout << "forgotten-medium " << purged << ' ' << (next != retired) << ' '
            << std::all_of(bytes, bytes + 40, [](auto b) { return b == 7; }) << '\n';
  allocator.Free(next);
  std::cout << "new-medium-free-list " << allocator.Purge(false) << '\n';
  allocator.Free(live);
  std::free(static_cast<size_t*>(retired) - 1); // Forgotten native heap region, after observations.
}

void retained_clear(bool clear)
{
  InitializeProbe allocator(clear);
  allocator.repeat();
  void* block = allocator.Allocate(40);
  std::memset(block, 23, 40);
  allocator.Free(block);
  void* next = allocator.Allocate(40);
  const auto* bytes = static_cast<unsigned char*>(next);
  const unsigned char expected = clear ? 0 : 23;
  std::cout << "clear-" << clear << ' ' << (block == next) << ' '
            << std::all_of(bytes, bytes + 40, [expected](auto b) { return b == expected; }) << '\n';
  allocator.Free(next);
}

int callbackCount = 0;
void count_callback(bool, void*, size_t, size_t)
{
  // Count semantic allocator notifications, not the harness's resource cleanup.
  ++callbackCount;
}

void initialization_callback()
{
  InitializeProbe allocator;
  Standard_MMgrOpt::SetCallBackFunction(count_callback);
  allocator.repeat();
  int afterInitialize = callbackCount;
  void* block = allocator.Allocate(40);
  std::cout << "initialize-callback " << afterInitialize << ' ' << callbackCount << '\n';
  Standard_MMgrOpt::SetCallBackFunction(nullptr);
  allocator.Free(block);
}

void native_resource_effects()
{
  InitializeProbe allocator(true, true);
  int first = allocator.descriptor();
  bool freshZero = allocator.repeat();
  int second = allocator.descriptor();
  std::cout << "native-table " << freshZero << '\n';
  std::cout << "native-descriptors " << (first != second) << ' '
            << (fcntl(first, F_GETFD) >= 0) << ' ' << (fcntl(second, F_GETFD) >= 0) << '\n';
  close(first); close(second); // Native destructor does not close either descriptor.
}

int main()
{
  InitializeProbe allocator;
  allocator.normalize();
  live_pool(); forgotten_small(); forgotten_medium();
  retained_clear(true); retained_clear(false); initialization_callback();
  native_resource_effects();
}
