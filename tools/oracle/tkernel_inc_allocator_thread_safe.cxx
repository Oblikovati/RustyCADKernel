#include <NCollection_IncAllocator.hxx>
#include <algorithm>
#include <iostream>
#include <thread>
#include <vector>

void allocate_many(NCollection_IncAllocator& allocator, std::vector<void*>& result)
{
  for (int i = 0; i < 128; ++i)
  {
    result.push_back(allocator.Allocate(64));
  }
}

int main()
{
  NCollection_IncAllocator allocator(4096);
  allocator.SetThreadSafe(true);
  std::vector<std::vector<void*>> results(4);
  std::vector<std::thread> threads;
  for (auto& result : results) threads.emplace_back(allocate_many, std::ref(allocator), std::ref(result));
  for (auto& thread : threads) thread.join();
  std::vector<void*> addresses;
  for (auto& result : results) addresses.insert(addresses.end(), result.begin(), result.end());
  std::sort(addresses.begin(), addresses.end());
  const auto uniqueCount = std::unique(addresses.begin(), addresses.end()) - addresses.begin();
  allocator.SetThreadSafe(false);
  allocator.Reset(false);
  std::cout << addresses.size() << ' ' << uniqueCount << ' ' << (allocator.Allocate(1) != nullptr) << '\n';
}
