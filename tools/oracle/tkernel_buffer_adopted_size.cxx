#include <NCollection_Buffer.hxx>
#include <algorithm>
#include <iostream>
static void row(std::size_t size) {
  auto allocator = NCollection_BaseAllocator::CommonBaseAllocator();
  auto* bytes = static_cast<uint8_t*>(allocator->Allocate(16));
  std::fill_n(bytes, 16, uint8_t(7));
  NCollection_Buffer buffer(allocator, size, bytes);
  std::fill(buffer.begin(), buffer.end(), uint8_t(3));
  std::size_t sum = 0;
  for (auto byte : buffer) sum += byte;
  std::cout << size << ' ' << buffer.Size() << ' ' << buffer.IsEmpty() << ' '
            << (buffer.Data() == bytes) << ' ' << (buffer.end() - buffer.begin()) << ' ' << sum << '\n';
}
int main() { for (auto size : {0u, 1u, 4u, 16u}) row(size); }
