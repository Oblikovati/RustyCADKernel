// OCCT 3d097a03: direct protected operations mutate the same public allocator engine.
#include <NCollection_AccAllocator.hxx>
#include <NCollection_IncAllocator.hxx>
#include <cstdint>
#include <iostream>
#include <limits>

void keys(size_t size)
{
  NCollection_AccAllocator arena(size);
  std::cout << "keys-" << size;
  for (std::uintptr_t address : {std::uintptr_t(0),std::uintptr_t(1),std::uintptr_t(63),
        std::uintptr_t(64),std::uintptr_t(65),std::numeric_limits<std::uintptr_t>::max()})
    std::cout << ' ' << arena.getKey(reinterpret_cast<void*>(address)).Value;
  std::cout << '\n';
}

void lookup_miss()
{
  NCollection_AccAllocator arena(64);
  NCollection_AccAllocator::Key key;
  auto* found = arena.findBlock(nullptr, key);
  std::cout << "key-miss " << arena.getKey(nullptr).Value << ' ' << key.Value
            << ' ' << (found == nullptr) << '\n';
}

void thresholds(bool slow)
{
  NCollection_IncAllocator arena(2048);
  arena.SetThreadSafe(true);
  auto* first = static_cast<char*>(arena.Allocate(1024));
  auto* second = static_cast<char*>(slow ? arena.allocateSlow(1016) : arena.Allocate(1016));
  auto* third = static_cast<char*>(arena.Allocate(8));
  std::cout << "threshold-" << slow << ' ' << second-first << ' '
            << (third == first+2040) << '\n';
}

void sorting(bool slow)
{
  NCollection_IncAllocator arena(2048);
  arena.SetThreadSafe(true);
  auto* first = static_cast<char*>(arena.Allocate(1600));
  auto* second = static_cast<char*>(arena.Allocate(1700));
  arena.Allocate(64);
  if (slow) arena.allocateSlow(64); else arena.Allocate(64);
  auto* next = static_cast<char*>(arena.Allocate(64));
  auto* tail = static_cast<char*>(arena.Allocate(8));
  std::cout << "sorting-" << slow << ' ' << (next == first+1728)
            << ' ' << (next == second+1700) << ' ' << (tail == next+64) << '\n';
}

void growth(size_t size)
{
  NCollection_IncAllocator arena(size);
  arena.increaseBlockSize();
  std::cout << "grow-" << size << ' ' << arena.myBlockSize << '\n';
}

void growth_storage()
{
  NCollection_IncAllocator arena(2048);
  arena.increaseBlockSize();
  auto* first = static_cast<char*>(arena.allocateSlow(2048));
  auto* second = static_cast<char*>(arena.allocateSlow(14336));
  arena.Reset(false);
  auto* reused = static_cast<char*>(arena.allocateSlow(16384));
  arena.Reset(true);
  auto* reset = static_cast<char*>(arena.allocateSlow(2048));
  auto* next = static_cast<char*>(arena.allocateSlow(10240));
  std::cout << "grow-storage " << (second==first+2048) << ' ' << (reused==first)
            << ' ' << (next==reset+2048) << '\n';
}

int main()
{
  for (size_t size : {61,64,65}) keys(size);
  lookup_miss();
  for (bool slow : {false,true}) { thresholds(slow); sorting(slow); }
  for (size_t size : {2048,12288,196608,786432,1000001,12582912}) growth(size);
  growth_storage();
}
