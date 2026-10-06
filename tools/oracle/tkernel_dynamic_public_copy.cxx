// OCCT 3d097a0: public copy/Assign lifetime and allocator selection before explicit leak recovery.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
struct Arena : NCollection_BaseAllocator {
  int live=0;
  bool recording=false;
  std::vector<void*> fresh;
  void* AllocateOptimal(size_t bytes) override {
    void* pointer=NCollection_BaseAllocator::AllocateOptimal(bytes);
    ++live; if(recording) fresh.push_back(pointer); return pointer;
  }
  void Free(void* pointer) override { --live; NCollection_BaseAllocator::Free(pointer); }
};
static Arena *sourceArena, *targetArena;
static std::string events;
static int failAt, attempts, liveItems;
static void event(char kind,int value) {
  events+=kind+std::to_string(value)+"@"+std::to_string(sourceArena->live)+","+std::to_string(targetArena->live)+";";
}
struct Item {
  int value;
  explicit Item(int number):value(number) { ++liveItems; }
  Item(const Item& source):value(source.value) {
    event('C',value); if(++attempts==failAt) throw std::runtime_error("copy failure"); ++liveItems;
  }
  Item& operator=(const Item&)=delete;
  ~Item() { event('X',value); --liveItems; }
};
template<class T> struct Engine : NCollection_DynamicArray<T> {
  using NCollection_DynamicArray<T>::NCollection_DynamicArray;
  using NCollection_DynamicArray<T>::blockCount;
  using NCollection_DynamicArray<T>::ensureBlockCount;
};
struct Fixture {
  occ::handle<Arena> sourceAllocator=new Arena;
  occ::handle<Arena> targetAllocator=new Arena;
  Engine<Item> source{2,sourceAllocator},target{4,targetAllocator};
  Fixture(int failure) {
    sourceArena=sourceAllocator.get();targetArena=targetAllocator.get();
    for(int value : {1,2,3}) source.EmplaceAppend(value);
    source.ensureBlockCount(3,sizeof(Item));
    for(int value : {90,91,92}) target.EmplaceAppend(value);
    failAt=failure;attempts=0;events.clear();
    sourceArena->recording=true;targetArena->recording=true;
  }
};
static std::string itemValues(const Item* items,size_t count) {
  std::string out;
  for(size_t i=0;i<count;++i) { if(i) out+=',';out+=std::to_string(items[i].value); }
  return out.empty()?"-":out;
}
static std::string publishedValues(const std::vector<void*>& blocks,size_t count) {
  std::string out;
  for(size_t i=0;i<count;++i) {
    if(i) out+=',';
    out+=std::to_string(static_cast<Item*>(blocks[i/2])[i%2].value);
  }
  return out.empty()?"-":out;
}
static void row(const std::string& name,Arena& chosen,bool failed,size_t published,size_t blocks,size_t partial) {
  auto* incomplete=failed?static_cast<Item*>(chosen.fresh.back()):nullptr;
  std::cout<<name<<' '<<(events.empty()?"none":events)<<" size="<<published<<" blocks="<<blocks
           <<" source="<<(&chosen==sourceArena)<<" failed="<<failed<<" partial="<<partial
           <<" live="<<liveItems<<" allocations="<<sourceArena->live<<','<<targetArena->live
           <<" values="<<publishedValues(chosen.fresh,published)<<" incomplete="<<itemValues(incomplete,partial)<<'\n';
}
static void destroyValues(Item* pointer,size_t count) {
  for(size_t i=0;i<count;++i) pointer[i].~Item();
}
static void constructorRecovery(Arena& arena,size_t successes) {
  for(size_t i=0;i<successes;++i) static_cast<Item*>(arena.fresh[i/2])[i%2].~Item();
  for(void* pointer : arena.fresh) arena.Free(pointer);
}
static void constructorCase(int failure) {
  Fixture fixture(failure);Engine<Item>* copied=nullptr;
  try { copied=new Engine<Item>(fixture.source); }
  catch(const std::runtime_error&) {}
  const size_t successes=copied?3:size_t(attempts-1);
  const size_t partial=copied?0:successes%2;
  row("constructor_"+std::to_string(failure),*sourceArena,!copied,successes-partial,
      copied?copied->blockCount():sourceArena->fresh.size()-1,partial);
  if(copied) delete copied;else constructorRecovery(*sourceArena,successes);
}
static void assignmentCase(int mode,int failure) {
  Fixture fixture(failure);bool failed=false;
  try {
    auto& result=mode==2?(fixture.target=fixture.source):fixture.target.Assign(fixture.source,mode==0);
    if(&result!=&fixture.target) throw std::logic_error("assignment: expected receiver");
  } catch(const std::runtime_error&) { failed=true; }
  Arena& chosen=mode==0?*targetArena:*sourceArena;
  const size_t partial=failed?size_t(attempts-1)-fixture.target.Size():0;
  row("assignment_"+std::to_string(mode)+"_"+std::to_string(failure),chosen,failed,
      fixture.target.Size(),fixture.target.blockCount(),partial);
  if(failed) { auto* tail=static_cast<Item*>(chosen.fresh.back());destroyValues(tail,partial);chosen.Free(tail); }
}
int main() {
  for(int failure : {0,1,2,3}) { constructorCase(failure); if(liveItems) return 2; }
  for(int mode=0;mode<3;++mode) for(int failure : {0,1,2,3}) {
    assignmentCase(mode,failure);if(liveItems) return 3;
  }
}
