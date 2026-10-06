// OCCT 3d097a03: interpose only the block-allocation boundary to retain native OOM prefixes.
#include <NCollection_IncAllocator.hxx>
#include <Standard.hxx>
#include <Standard_OutOfMemory.hxx>
#include <iostream>

struct FailingBlockAllocation {
  static bool enabled;
};
bool FailingBlockAllocation::enabled = false;

void* Standard::AllocateOptimal(size_t size)
{
  if (FailingBlockAllocation::enabled)
    throw Standard_OutOfMemory("injected block allocation failure");
  return Standard::Allocate(size);
}

void fail_next(NCollection_IncAllocator& arena, size_t size, const char* label)
{
  auto ordered = arena.myOrderedBlocks;
  auto used = arena.myUsedHeap;
  auto available = arena.myAllocationHeap;
  FailingBlockAllocation::enabled = true;
  bool caught = false;
  try { arena.Allocate(size); } catch (Standard_OutOfMemory&) { caught = true; }
  FailingBlockAllocation::enabled = false;
  std::cout << label << ' ' << caught << ' ' << arena.myBlockCount << ' '
            << arena.myBlockSize << ' ' << (ordered == arena.myOrderedBlocks) << ' '
            << (used == arena.myUsedHeap) << ' ' << (available == arena.myAllocationHeap)
            << '\n';
}

void fifth_failure()
{
  NCollection_IncAllocator arena(2048);
  for (int index = 0; index < 4; ++index) arena.Allocate(2048);
  fail_next(arena, 8, "fifth-failure");
  arena.Allocate(8);
  std::cout << "fifth-retry " << arena.myBlockCount << ' ' << arena.myBlockSize << '\n';
}

void first_failure()
{
  NCollection_IncAllocator arena(2048);
  fail_next(arena, 4096, "first-failure");
  arena.Allocate(8);
  std::cout << "first-retry " << arena.myBlockCount << ' ' << arena.myBlockSize << '\n';
}

void available_failure()
{
  NCollection_IncAllocator arena(2048);
  arena.Allocate(8);
  fail_next(arena, 8192, "available-failure");
  arena.Allocate(8);
  std::cout << "available-reuse " << arena.myBlockCount << ' ' << arena.myBlockSize << '\n';
}

int main()
{
  first_failure();
  fifth_failure();
  available_failure();
}
