#include <NCollection_Buffer.hxx>
#include <NCollection_HeapAllocator.hxx>
#include <NCollection_IncAllocator.hxx>
#include <NCollection_AccAllocator.hxx>
#include <NCollection_AlignedAllocator.hxx>
#include <Standard_ProgramError.hxx>
#include <algorithm>
#include <iostream>
using Allocator = occ::handle<NCollection_BaseAllocator>;
static Allocator allocator(int policy) {
  switch (policy) {
    case 0: return NCollection_BaseAllocator::CommonBaseAllocator();
    case 1: return Allocator(NCollection_HeapAllocator::GlobalHeapAllocator().get());
    case 2: return new NCollection_IncAllocator(4096);
    case 3: return new NCollection_AccAllocator(256);
    default: return new NCollection_AlignedAllocator(32);
  }
}
static uint8_t* initialized(const Allocator& owner, bool optimal, size_t size) {
  auto* pointer = static_cast<uint8_t*>(optimal ? owner->AllocateOptimal(size) : owner->Allocate(size));
  std::fill_n(pointer, size, uint8_t(7));
  return pointer;
}
static void prefix(int policy, bool optimal, size_t size) {
  auto owner = allocator(policy);
  auto* pointer = initialized(owner, optimal, 16);
  NCollection_Buffer buffer(owner, size, pointer);
  size_t sum = 0;
  for (uint8_t value : buffer) sum += value;
  std::cout << "prefix " << policy << ' ' << optimal << ' ' << size << ' '
            << buffer.Size() << ' ' << buffer.IsEmpty() << ' ' << (buffer.Data() == pointer)
            << ' ' << (buffer.end() - buffer.begin()) << ' ' << sum << '\n';
}
static void null_pointer(int policy, size_t size) {
  auto owner = allocator(policy);
  try {
    NCollection_Buffer buffer(owner, size, nullptr);
    std::cout << "null " << policy << ' ' << size << ' ' << buffer.Size() << ' ' << buffer.IsEmpty() << '\n';
  } catch (const Standard_ProgramError&) {
    std::cout << "null " << policy << ' ' << size << " Standard_ProgramError\n";
  }
}
static void replace(int policy) {
  auto owner = allocator(policy);
  auto replacement = NCollection_BaseAllocator::CommonBaseAllocator();
  NCollection_Buffer buffer(owner, 4, initialized(owner, false, 16));
  buffer.SetAllocator(replacement);
  std::cout << "replace " << policy << ' ' << buffer.Size() << ' ' << buffer.IsEmpty()
            << ' ' << (buffer.Allocator() == replacement) << '\n';
}
static void allocate_again(int policy) {
  auto owner = allocator(policy);
  NCollection_Buffer buffer(owner, 4, initialized(owner, false, 16));
  bool success = buffer.Allocate(6);
  std::fill(buffer.begin(), buffer.end(), uint8_t(3));
  size_t sum = 0;
  for (uint8_t value : buffer) sum += value;
  std::cout << "allocate " << policy << ' ' << success << ' ' << buffer.Size() << ' ' << sum << '\n';
}
static void free_buffer(int policy) {
  auto owner = allocator(policy);
  NCollection_Buffer buffer(owner, 4, initialized(owner, false, 16));
  buffer.Free();
  std::cout << "free " << policy << ' ' << buffer.Size() << ' ' << buffer.IsEmpty()
            << ' ' << (buffer.Allocator() == owner) << '\n';
}
class CountedAccAllocator : public NCollection_AccAllocator {
public:
  CountedAccAllocator() : NCollection_AccAllocator(256) {}
  int count() const { return mypLastBlock->allocCount; }
};
static void accumulating_count() {
  occ::handle<CountedAccAllocator> owner = new CountedAccAllocator;
  auto* pointer = initialized(owner, false, 16);
  std::cout << "acc_count " << owner->count();
  { NCollection_Buffer buffer(owner, 4, pointer); std::cout << ' ' << owner->count(); }
  std::cout << ' ' << owner->count() << '\n';
}
static void incremental_cursor() {
  auto owner = allocator(2);
  auto* pointer = initialized(owner, false, 16);
  NCollection_Buffer buffer(owner, 4, pointer);
  auto* next = static_cast<uint8_t*>(owner->Allocate(1));
  std::cout << "inc_cursor " << (next - pointer) << '\n';
  owner->Free(next);
}
static void zero_allocation(int policy) {
  auto owner = allocator(policy);
  auto* pointer = static_cast<uint8_t*>(owner->Allocate(0));
  NCollection_Buffer buffer(owner, 0, pointer);
  std::cout << "zero " << policy << ' ' << buffer.Size() << ' ' << buffer.IsEmpty()
            << ' ' << (buffer.Data() == pointer) << '\n';
}
int main() {
  for (int policy = 0; policy != 5; ++policy) {
    for (bool optimal : {false, true}) for (size_t size : {0u, 1u, 4u, 16u}) prefix(policy, optimal, size);
    for (size_t size : {0u, 4u}) null_pointer(policy, size);
    replace(policy); allocate_again(policy);
    if (policy != 3) { free_buffer(policy); zero_allocation(policy); }
  }
  accumulating_count(); incremental_cursor();
  NCollection_Buffer invalid(new NCollection_AlignedAllocator(3), 4);
  std::cout << "invalid_alignment " << invalid.Size() << ' ' << invalid.IsEmpty() << '\n';
}
