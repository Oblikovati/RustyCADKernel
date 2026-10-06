// Deliberate compile probe: pinned cross-specialization assignment accesses a private handle.
#include <NCollection_OccAllocator.hxx>
int main() {
  NCollection_OccAllocator<int> target;
  NCollection_OccAllocator<char> source;
  target = source;
}
