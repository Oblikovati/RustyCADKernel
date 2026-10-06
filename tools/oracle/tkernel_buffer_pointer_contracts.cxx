#include <NCollection_Buffer.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <algorithm>
#include <iostream>

void pointer_range(const char* name, NCollection_Buffer& buffer)
{
  const NCollection_Buffer& constant = buffer;
  const auto beginning = constant.Data();
  const auto ending = buffer.Size() == 0 ? beginning : beginning + buffer.Size();
  std::cout << name << ':' << (beginning != nullptr) << ':' << buffer.Size() << ':'
            << (buffer.begin() == beginning) << ':' << (constant.begin() == beginning) << ':'
            << (constant.cbegin() == beginning) << ':' << (buffer.end() == ending) << ':'
            << (constant.end() == ending) << ':' << (constant.cend() == ending) << ':'
            << (buffer.ChangeData() == beginning) << '\n';
}

void allocated_pointer_ranges(const occ::handle<NCollection_BaseAllocator>& allocator)
{
  NCollection_Buffer owned(allocator, 4);
  pointer_range("owned", owned);
  owned.Free();
  pointer_range("freed", owned);
  owned.Allocate(3);
  pointer_range("reallocated", owned);
  auto data = static_cast<uint8_t*>(allocator->Allocate(3));
  NCollection_Buffer adopted(allocator, 3, data);
  pointer_range("adopted", adopted);
}

void mutable_pointer_range(const occ::handle<NCollection_BaseAllocator>& allocator)
{
  NCollection_Buffer buffer(allocator, 4);
  auto beginning = buffer.begin();
  std::fill(beginning, buffer.end(), 'x');
  const bool contents = std::all_of(buffer.cbegin(), buffer.cend(), [](uint8_t value) { return value == 'x'; });
  std::cout << "write:" << contents << ':' << (buffer.end() == beginning + 4) << '\n';
}

int main()
{
  NCollection_Buffer unallocated(nullptr, 0);
  pointer_range("null", unallocated);
  auto allocator = NCollection_BaseAllocator::CommonBaseAllocator();
  NCollection_Buffer empty(allocator, 0);
  pointer_range("zero", empty);
  allocated_pointer_ranges(allocator);
  mutable_pointer_range(allocator);
}
