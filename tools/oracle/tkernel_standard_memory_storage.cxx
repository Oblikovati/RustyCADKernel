// OCCT 3d097a03 NATIVE manager: zero-byte identity, initialized prefixes and failure ownership.
#include <Standard.hxx>
#include <Standard_OutOfMemory.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <limits>

void zero_pair(bool optimal)
{
  void* first = optimal ? Standard::AllocateOptimal(0) : Standard::Allocate(0);
  void* second = optimal ? Standard::AllocateOptimal(0) : Standard::Allocate(0);
  std::cout << "zero-" << optimal << ' ' << (first != nullptr) << ' ' << (second != nullptr)
            << ' ' << (first != second) << ' ' << (reinterpret_cast<uintptr_t>(first) % 16 == 0)
            << ' ' << (reinterpret_cast<uintptr_t>(second) % 16 == 0) << '\n';
  Standard::Free(first); Standard::Free(second);
}

void aligned_pair(size_t alignment)
{
  void* first = Standard::AllocateAligned(0, alignment);
  void* second = Standard::AllocateAligned(0, alignment);
  const size_t expected = alignment < 16 ? 16 : alignment;
  std::cout << "aligned-zero-" << alignment << ' ' << (first != nullptr) << ' '
            << (first != second) << ' ' << (reinterpret_cast<uintptr_t>(first) % expected == 0)
            << ' ' << (reinterpret_cast<uintptr_t>(second) % expected == 0) << '\n';
  Standard::FreeAligned(first); Standard::FreeAligned(second);
}

void realloc_null_zero()
{
  void* first = Standard::Reallocate(nullptr, 0);
  void* second = Standard::Reallocate(nullptr, 0);
  std::cout << "realloc-null-zero " << (first != nullptr) << ' ' << (first != second) << ' '
            << (reinterpret_cast<uintptr_t>(first) % 16 == 0) << '\n';
  Standard::Free(first); Standard::Free(second);
}

void realloc_prefix()
{
  char* pointer = static_cast<char*>(Standard::Allocate(8));
  std::memcpy(pointer, "abcdefgh", 8);
  pointer = static_cast<char*>(Standard::Reallocate(pointer, 32));
  bool growth = std::memcmp(pointer, "abcdefgh", 8) == 0;
  pointer = static_cast<char*>(Standard::Reallocate(pointer, 3));
  std::cout << "realloc-prefix " << growth << ' ' << (std::memcmp(pointer, "abc", 3) == 0) << '\n';
  Standard::Free(pointer);
}

void realloc_positive_failure()
{
  char* pointer = static_cast<char*>(Standard::Allocate(8));
  std::memcpy(pointer, "abcdefgh", 8);
  bool caught = false;
  try { Standard::Reallocate(pointer, std::numeric_limits<size_t>::max()); }
  catch (const Standard_OutOfMemory&) { caught = true; }
  std::cout << "realloc-positive-failure " << caught << ' '
            << (std::memcmp(pointer, "abcdefgh", 8) == 0) << '\n';
  Standard::Free(pointer);
}

void realloc_existing_zero()
{
  void* pointer = Standard::Allocate(8);
  bool caught = false;
  try { pointer = Standard::Reallocate(pointer, 0); }
  catch (const Standard_OutOfMemory&) { caught = true; pointer = nullptr; }
  // On the pinned GNU/Linux CRT the zero-size call already frees the old allocation.
  std::cout << "realloc-existing-zero " << caught << ' ' << (pointer == nullptr) << '\n';
  Standard::Free(pointer);
}

void common_interchange()
{
  void* standard = Standard::Allocate(3);
  void* common = NCollection_BaseAllocator::CommonBaseAllocator()->Allocate(3);
  NCollection_BaseAllocator::CommonBaseAllocator()->Free(standard);
  Standard::Free(common);
  std::cout << "common-interchange 1\n";
}

int main()
{
  std::cout << "manager " << static_cast<int>(Standard::GetAllocatorType()) << ' ' << Standard::Purge() << '\n';
  zero_pair(false); zero_pair(true);
  for (size_t alignment : {1, 2, 4, 8, 16, 64}) aligned_pair(alignment);
  realloc_null_zero(); realloc_prefix(); realloc_positive_failure();
  realloc_existing_zero(); common_interchange();
}
