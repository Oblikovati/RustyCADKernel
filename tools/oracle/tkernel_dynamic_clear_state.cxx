// Pinned OCCT 3d097a0 Clear calls throwing destructors directly and stops at the first throw.
#include <NCollection_DynamicArray.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>
struct ClearArena:NCollection_BaseAllocator {
  int live=0;
  void* AllocateOptimal(size_t bytes) override {++live;return NCollection_BaseAllocator::AllocateOptimal(bytes);}
  void Free(void* pointer) override {--live;NCollection_BaseAllocator::Free(pointer);}
};
struct ClearItem;
struct ClearProbe {
  NCollection_DynamicArray<ClearItem>* owner=nullptr;ClearArena* arena=nullptr;
  int fail=0;std::string events;std::vector<ClearItem*> slots;std::vector<int> ended;
};
struct ClearItem {
  int value;ClearProbe* probe;
  ClearItem(int v,ClearProbe& p):value(v),probe(&p){}
  ~ClearItem() noexcept(false) {
    const bool same=this==probe->slots.at(size_t(value-1));
    probe->events+="D"+std::to_string(value)+"@"+std::to_string(probe->owner->Size())
      +"#"+std::to_string(probe->arena->live)+":"+std::to_string(same)+";";
    probe->ended.push_back(value);if(value==probe->fail)throw std::runtime_error("clear destructor");
  }
};
static void repair(ClearProbe& probe) {
  for(int value:probe.ended)new(probe.slots.at(size_t(value-1)))ClearItem(value,probe);
  probe.ended.clear();probe.events.clear();probe.fail=0;
}
static void clearRow(bool release,int fail) {
  occ::handle<ClearArena> arena=new ClearArena;ClearProbe probe;probe.arena=arena.get();
  NCollection_DynamicArray<ClearItem> array(size_t(2),arena);probe.owner=&array;
  for(int value=1;value<=5;++value)probe.slots.push_back(&array.EmplaceAppend(value,probe));
  probe.fail=fail;bool caught=false;
  try{array.Clear(release);}catch(const std::runtime_error&){caught=true;}
  std::cout<<"clear release="<<release<<" fail="<<fail<<" caught="<<caught<<" size="<<array.Size()
    <<" blocks="<<arena->live<<" ended="<<probe.ended.size()<<" events="<<probe.events<<'\n';
  if(!caught)return;
  const int tail=fail<5?array.Value(fail).value:0;
  std::cout<<"tail fail="<<fail<<" value="<<tail<<'\n';repair(probe);array.Clear(true);
  std::cout<<"repair size="<<array.Size()<<" blocks="<<arena->live<<" events="<<probe.events<<'\n';
}
int main(){for(bool release:{false,true})for(int fail:{0,1,2,3,5})clearRow(release,fail);}
