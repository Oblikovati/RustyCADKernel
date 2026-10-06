// OCCT 3d097a03: raw accumulating reservations for geometries aligned to typed Rust layouts.
#include <NCollection_AccAllocator.hxx>
#include <cstdint>
#include <Standard_Failure.hxx>
#include <iostream>
#include <vector>

struct ReservationArena : NCollection_AccAllocator
{
  ReservationArena(size_t size):NCollection_AccAllocator(size) {}
  void Observe(void* pointer);
  void ObserveEmpty();
};

void ReservationArena::Observe(void* pointer)
{
  Key key;
  Block* block = findBlock(pointer, key);
  auto offset = reinterpret_cast<std::uintptr_t>(pointer)
              - reinterpret_cast<std::uintptr_t>(block->address);
  std::cout << ' ' << offset << ':' << block->FreeSize()
            << ':' << block->allocCount << ':' << myBlocks.Size();
}

void ReservationArena::ObserveEmpty()
{
  std::cout << " empty " << mypLastBlock->FreeSize() << ':'
            << mypLastBlock->allocCount << ':' << myBlocks.Size() << '\n';
}

void scenario(const char* label, size_t blockSize, std::initializer_list<size_t> sizes)
{
  ReservationArena arena(blockSize);
  std::vector<void*> pointers;
  std::cout << label;
  for (size_t size : sizes)
  {
    void* pointer = arena.Allocate(size);
    pointers.push_back(pointer);
    arena.Observe(pointer);
  }
  for (void* pointer : pointers) arena.Free(pointer);
  arena.ObserveEmpty();
}

void tiny_raw_lookup()
{
  ReservationArena arena(16);
  auto* block = arena.allocateNewBlock(512);
  void* pointer = arena.Allocate(64);
  const int before = block->allocCount;
  bool failed = false;
  try { arena.Free(pointer); }
  catch (const Standard_Failure&) { failed = true; }
  std::cout << "tiny-raw " << before << ' ' << failed << ' ' << block->allocCount << '\n';
}

int main()
{
  scenario("mixed", 1024, {32,40,32,40});
  scenario("exhaust", 96, {32,32,32,32});
  scenario("previous", 128, {80,80,40,32});
  scenario("oversize", 128, {160,32});
  scenario("prefix-aligned", 96, {8,32,40});
  scenario("unaligned-raw", 132, {32,32});
  tiny_raw_lookup();
}
