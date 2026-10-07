// GNU/Linux native oracle only: observe real TKernel free calls without allocating in the hook.
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <iostream>
extern "C" void __libc_free(void*) noexcept;
inline uintptr_t watchedNodes[4] = {};
inline size_t watchedClasses[4] = {};
inline size_t** watchedTable = nullptr;
inline bool observeNativeFree = false;
struct ReleaseEntry { size_t index; int node; int head; };
inline ReleaseEntry releases[8] = {};
inline size_t releaseCount = 0;

inline int node_id(uintptr_t address)
{
  for (int i = 0; i < 4; ++i) if (watchedNodes[i] == address) return i + 1;
  return 0;
}

extern "C" void free(void* address) noexcept
{
  if (observeNativeFree) {
    const int node = node_id(reinterpret_cast<uintptr_t>(address));
    if (node != 0) {
      const size_t index = watchedClasses[node - 1];
      uintptr_t head = 0;
      // Read live table bytes, never evaluate a pointer value whose allocation was just freed.
      std::memcpy(&head, watchedTable + index, sizeof(head));
      releases[releaseCount++] = {index, node, node_id(head)};
    }
  }
  __libc_free(address);
}

inline void print_releases(int freed)
{
  std::cout << "purge-order " << freed << ' ';
  for (size_t i = 0; i < releaseCount; ++i) {
    if (i != 0) std::cout << ',';
    std::cout << releases[i].index << ':' << releases[i].node << ':' << releases[i].head;
  }
  std::cout << '\n';
}
