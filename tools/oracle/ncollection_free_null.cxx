#include <NCollection_AccAllocator.hxx>
#include <NCollection_AlignedAllocator.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <NCollection_HeapAllocator.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Standard_Failure.hxx>
#include <Standard_ProgramError.hxx>
#include <iostream>

template <typename Allocator>
void probe(const char* name, Allocator& allocator)
{
  try
  {
    allocator.Free(nullptr);
    std::cout << name << ":ok\n";
  }
  catch (const Standard_ProgramError& error)
  {
    std::cout << name << ":Standard_ProgramError:" << error.what() << "\n";
  }
  catch (const Standard_Failure&)
  {
    std::cout << name << ":unexpected Standard_Failure\n";
  }
}

int main()
{
  auto& common = NCollection_BaseAllocator::CommonBaseAllocator();
  auto& heap = NCollection_HeapAllocator::GlobalHeapAllocator();
  NCollection_IncAllocator incremental(64);
  NCollection_AccAllocator accumulating(64);
  NCollection_AlignedAllocator aligned(16);
  probe("common", *common);
  probe("heap", *heap);
  probe("incremental", incremental);
  probe("accumulating", accumulating);
  probe("aligned", aligned);
}
