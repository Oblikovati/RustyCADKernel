// Pinned OCCT canonical free-list slots, replacement lifetime and actual Purge release order.
#include <Standard_MMgrOpt.hxx>
#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <limits>
#include <vector>
#include "tkernel_mmgr_opt_free_list_table/free_observer.hxx"

class TableProbe : public Standard_MMgrOpt
{
public:
  explicit TableProbe(size_t threshold = 512)
    : Standard_MMgrOpt(true, false, 0, 100, threshold)
  {
    // No pools are needed by the table-only constructor probes.
  }
  void print_layout(size_t threshold)
  {
    const bool zero = std::all_of(myFreeList, myFreeList + myFreeListMax + 1, [](auto p) { return !p; });
    std::cout << "table-" << threshold << ' ' << myFreeListMax << ' '
              << (myFreeListMax + 1) * sizeof(size_t*) << ' ' << zero << ' '
              << (reinterpret_cast<uintptr_t>(myFreeList) % alignof(size_t*) == 0) << '\n';
  }
  size_t* head(size_t index)
  {
    return myFreeList[index];
  }
  void print_head_layout(void* first, void* second)
  {
    auto* left = static_cast<size_t*>(first) - 1;
    auto* right = static_cast<size_t*>(second) - 1;
    const auto offset = reinterpret_cast<char*>(&myFreeList[2]) - reinterpret_cast<char*>(myFreeList);
    std::cout << "slot-head " << offset << ' ' << (myFreeList[2] == right) << ' '
              << (*reinterpret_cast<size_t**>(right) == left) << '\n';
  }
  size_t** replace()
  {
    auto* previous = myFreeList;
    Initialize();
    return previous;
  }
  void failure_row(const char* label)
  {
    std::cout << label << ' ' << myFreeListMax << ' ' << (myFreeList == nullptr) << '\n';
    // Repair only after observing calloc refusal; the native destructor otherwise dereferences null.
    myFreeListMax = 0; myCellSize = 0;
    myFreeList = static_cast<size_t**>(calloc(1, sizeof(size_t*)));
  }
  int purge_observed()
  {
    watchedTable = myFreeList; observeNativeFree = true;
    int freed = Purge(false);
    observeNativeFree = false;
    return freed;
  }
};

void threshold_matrix()
{
  for (size_t threshold : {0, 1, 7, 8, 9, 511, 512, 513, 514}) {
    TableProbe allocator(threshold);
    allocator.print_layout(threshold);
  }
}

void head_transitions()
{
  TableProbe allocator;
  void* first = allocator.Allocate(9);
  void* second = allocator.Allocate(13);
  allocator.Free(first); allocator.Free(second);
  allocator.print_head_layout(first, second);
  void* reused = allocator.Allocate(16);
  std::cout << "slot-pop " << (reused == second) << ' '
            << (allocator.head(2) == static_cast<size_t*>(first) - 1) << ' '
            << (static_cast<size_t*>(reused)[-1] == 16) << '\n';
  allocator.Free(reused);
  std::cout << "slot-purge " << allocator.Purge(false) << ' ' << (allocator.head(2) == nullptr) << '\n';
}

void replacement_lifetime()
{
  TableProbe allocator;
  void* block = allocator.Allocate(16);
  allocator.Free(block);
  std::vector<size_t**> retired;
  for (int i = 0; i < 3; ++i) retired.push_back(allocator.replace());
  bool distinct = retired[0] != retired[1] && retired[0] != retired[2] && retired[1] != retired[2];
  std::cout << "retained-tables " << retired.size() << ' ' << distinct << ' '
            << (retired[0][2] == static_cast<size_t*>(block) - 1) << ' ' << (allocator.head(2) == nullptr) << '\n';
  for (auto* table : retired) std::free(table);
  std::free(static_cast<size_t*>(block) - 1); // Forgotten medium backing, after all observations.
}

void allocation_failures()
{
  TableProbe overflow(std::numeric_limits<size_t>::max() - 7);
  overflow.failure_row("table-overflow");
  TableProbe refused(static_cast<size_t>(std::numeric_limits<ptrdiff_t>::max()));
  refused.failure_row("table-refused");
}

void purge_order()
{
  TableProbe allocator;
  void* blocks[4] = {allocator.Allocate(8), allocator.Allocate(8), allocator.Allocate(16), allocator.Allocate(24)};
  for (int i = 0; i < 4; ++i) {
    watchedNodes[i] = reinterpret_cast<uintptr_t>(static_cast<size_t*>(blocks[i]) - 1);
    watchedClasses[i] = i < 2 ? 1 : static_cast<size_t>(i);
  }
  allocator.Free(blocks[0]); allocator.Free(blocks[1]);
  allocator.Free(blocks[3]); allocator.Free(blocks[2]);
  print_releases(allocator.purge_observed());
}

int main()
{
  threshold_matrix(); head_transitions(); replacement_lifetime();
  allocation_failures(); purge_order();
}
