// Pinned OCCT 3d097a03: thread-safe fast allocations deliberately skip list repair.
#include <NCollection_IncAllocator.hxx>
#include <cstdint>
#include <iostream>
#include <string>
#include <vector>

struct Trace
{
  NCollection_IncAllocator Arena{2048};
  std::vector<std::pair<std::uintptr_t, size_t>> Blocks;
  void Allocate(size_t size, bool optimal = false);
  void Reset(bool release);
};

void Trace::Allocate(size_t size, bool optimal)
{
  void* pointer = optimal ? Arena.AllocateOptimal(size) : Arena.Allocate(size);
  const auto address = reinterpret_cast<std::uintptr_t>(pointer);
  auto* head = Arena.myOrderedBlocks;
  const auto start = reinterpret_cast<std::uintptr_t>(head) + sizeof(*head);
  bool known = false;
  for (const auto& block : Blocks) known |= block.first == start;
  if (!known) Blocks.push_back({start, Arena.myBlockSize});
  for (size_t i = 0; i < Blocks.size(); ++i)
    if (address >= Blocks[i].first && address <= Blocks[i].first + Blocks[i].second)
    { std::cout << ' ' << i << ':' << address - Blocks[i].first; return; }
  std::abort();
}

void Trace::Reset(bool release)
{
  Arena.Reset(release);
  if (release) Blocks.clear();
  std::cout << (release ? " release" : " rewind");
}

void threshold(bool enabled)
{
  Trace trace;
  trace.Arena.SetThreadSafe(enabled);
  std::cout << "threshold-" << enabled;
  for (size_t size : {1024, 1016, 8, 0, 1}) trace.Allocate(size, true);
  std::cout << '\n';
}

void sorted(bool enabled)
{
  Trace trace;
  trace.Arena.SetThreadSafe(enabled);
  std::cout << "sorted-" << enabled;
  for (size_t size : {1600, 1700, 64, 64, 64}) trace.Allocate(size);
  std::cout << '\n';
}

void reset(bool enabled, bool release)
{
  Trace trace;
  trace.Arena.SetThreadSafe(enabled);
  std::cout << (release ? "release-" : "rewind-") << enabled;
  trace.Allocate(2048); trace.Allocate(2048); trace.Reset(release);
  if (release) { trace.Allocate(1024); trace.Allocate(11256); }
  trace.Allocate(8); trace.Allocate(8);
  std::cout << '\n';
}

void toggle()
{
  Trace trace;
  trace.Arena.SetThreadSafe(true);
  std::cout << "toggle";
  for (size_t size : {1600, 1700, 64, 64}) trace.Allocate(size);
  trace.Arena.SetThreadSafe(false);
  trace.Allocate(8); trace.Allocate(8);
  trace.Arena.SetThreadSafe(true);
  trace.Allocate(8); trace.Allocate(8);
  std::cout << '\n';
}

void growth(bool enabled)
{
  Trace trace;
  trace.Arena.SetThreadSafe(enabled);
  std::cout << "growth-" << enabled;
  for (int i = 0; i < 5; ++i) trace.Allocate(2048);
  trace.Allocate(8); trace.Allocate(8);
  std::cout << '\n';
}

int main()
{
  for (bool enabled : {false, true})
  {
    threshold(enabled); sorted(enabled); reset(enabled, false);
    reset(enabled, true); growth(enabled);
  }
  toggle();
}
