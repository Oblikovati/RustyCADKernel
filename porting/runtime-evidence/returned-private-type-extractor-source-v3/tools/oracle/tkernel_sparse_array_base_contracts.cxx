#include <NCollection_SparseArrayBase.hxx>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <cstring>

struct RawSparse : NCollection_SparseArrayBase {
  RawSparse(size_t stride, size_t count, DestroyItemFunc destroy)
      : NCollection_SparseArrayBase(stride, count, destroy) {}
  using NCollection_SparseArrayBase::setValue;
  using NCollection_SparseArrayBase::unsetValue;
  using NCollection_SparseArrayBase::assign;
  using NCollection_SparseArrayBase::exchange;
  using NCollection_SparseArrayBase::clearItems;
  using NCollection_SparseArrayBase::getValue;
  using NCollection_SparseArrayBase::getBlock;
  using NCollection_SparseArrayBase::getItem;
  void* block(size_t n) const { return myData[n]; }
  size_t slots() const { return myNbBlocks; }
  size_t stride() const { return myItemSize; }
  size_t increment() const { return myBlockSize; }
};
static RawSparse* observing = nullptr;
static std::ostringstream trace;
static int reject = -1;
static size_t index_of(void* item) {
  for (size_t number=0; number<observing->slots(); ++number) {
    if (!observing->block(number)) continue;
    auto block=observing->getBlock(observing->block(number));
    for(size_t slot=0; slot<observing->increment(); ++slot)
      if(observing->getItem(block, slot)==item) return number*observing->increment()+slot;
  }
  throw std::runtime_error("missing item");
}
static void state(char event, void* item) {
  size_t index=index_of(item);
  auto block=observing->getBlock(observing->block(index/observing->increment()));
  trace << event << index << ':' << observing->Size() << ':' << *block.Count << ':' << observing->HasValue(index) << ',';
}
static void create(void* item, void* source) {
  state('C',item);
  if(*static_cast<int*>(source)==reject) throw std::runtime_error("copy rejected");
  std::memcpy(item, source, sizeof(int));
}
static void copy(void* item, void* source) {
  state(item==source ? 'S' : 'A',item);
  std::memmove(item, source, sizeof(int));
}
static void destroy(void* item) { if(observing) state('D',item); }
static void reset(RawSparse& array) { observing=&array; trace.str(""); trace.clear(); }
static void add(RawSparse& array,size_t index,int value) {
  observing=&array; array.setValue(index,&value,create,copy);
}
static void layout() {
  RawSparse array(12,9,destroy); add(array,8,88); add(array,9,99); reset(array);
  auto block=array.getBlock(array.block(0));
  auto* bytes=static_cast<unsigned char*>(array.block(0));
  std::cout << "layout " << array.slots() << ' ' << array.Capacity() << ' '
            << (static_cast<char*>(block.Array)-reinterpret_cast<char*>(bytes)) << ' '
            << (block.Bits-bytes) << ' ' << *block.Count << ' '
            << int(block.Bits[0]) << ' ' << int(block.Bits[1]) << ' '
            << (static_cast<char*>(array.getItem(block,8))-static_cast<char*>(block.Array)) << '\n';
  observing=nullptr;
}
static void insertion_removal() {
  RawSparse array(sizeof(int),4,destroy); reset(array); int value=7;
  array.setValue(1,&value,create,copy);
  value=9; array.setValue(1,&value,create,copy);
  array.setValue(1,array.getValue(1),create,copy);
  array.unsetValue(1,destroy);
  std::cout << "set_unset_states " << trace.str() << ' ' << array.Size() << ' ' << (array.block(0)==nullptr) << '\n';
  observing=nullptr;
}
static void assign_removal() {
  RawSparse source(sizeof(int),4,destroy), target(sizeof(int),4,destroy);
  add(source,0,10); add(source,8,80);
  add(target,0,1); add(target,1,11); add(target,4,4); add(target,8,8); add(target,12,12);
  reset(target); target.assign(source,create,destroy,copy);
  std::cout << "assign_states " << trace.str() << ' ' << target.Size() << '\n';
  observing=nullptr;
}
static void clear_states() {
  RawSparse array(sizeof(int),4,destroy); add(array,0,1); add(array,1,2); add(array,5,5);
  reset(array); array.clearItems(destroy);
  std::cout << "clear_states " << trace.str() << ' ' << array.Size() << ' ' << array.Capacity() << '\n';
  observing=nullptr;
}
static void failed_set() {
  RawSparse array(sizeof(int),4,destroy); reset(array); int value=44; reject=44;
  try { array.setValue(9,&value,create,copy); } catch(const std::runtime_error&) {}
  auto block=array.getBlock(array.block(2));
  std::cout << "failed_set_state " << trace.str() << ' ' << array.Size() << ' ' << array.Capacity() << ' ' << *block.Count << ' ' << int(block.Bits[0]) << '\n';
  reject=-1; observing=nullptr;
}
static int destroyed_a=0,destroyed_b=0;
static void destroy_a(void*) { ++destroyed_a; }
static void destroy_b(void*) { ++destroyed_b; }
static void destructor_exchange() {
  { RawSparse a(sizeof(int),4,destroy_a), b(12,9,destroy_b);
    add(a,0,1); add(b,8,8); add(b,9,9); observing=nullptr;
    a.exchange(b);
    std::cout << "exchange_layout " << a.stride() << ' ' << a.increment() << ' ' << a.Size() << ' ' << b.stride() << ' ' << b.increment() << ' ' << b.Size() << '\n';
  }
  std::cout << "exchange_destructors " << destroyed_a << ' ' << destroyed_b << '\n';
}
int main() { layout(); insertion_removal(); assign_removal(); clear_states(); failed_set(); destructor_exchange(); }
