#include <NCollection_DynamicArray.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct TableArena : NCollection_BaseAllocator {
  int allocated=0, freed=0;
  void* AllocateOptimal(size_t bytes) override { ++allocated; return NCollection_BaseAllocator::AllocateOptimal(bytes); }
  void Free(void* pointer) override { ++freed; NCollection_BaseAllocator::Free(pointer); }
  int live() const { return allocated-freed; }
};
template<class T> struct TableEngine : NCollection_DynamicArray<T> {
  using NCollection_DynamicArray<T>::NCollection_DynamicArray;
  using NCollection_DynamicArray<T>::blockCount;
  using NCollection_DynamicArray<T>::availableSize;
  using NCollection_DynamicArray<T>::ensureBlockCount;
  using NCollection_DynamicArray<T>::expandBlock;
  using NCollection_DynamicArray<T>::clearBlocks;
  // -fno-access-control exposes metadata only; all methods execute the unchanged native header.
  size_t descriptorCapacity() const { return this->myContainer.Capacity(); }
};
template<class T> static void tableRow(const char* name,const TableEngine<T>& target,const TableArena& arena) {
  std::cout<<name<<' '<<target.Size()<<' '<<target.blockCount()<<' '<<target.descriptorCapacity()<<' '
           <<target.availableSize()<<' '<<arena.live()<<'\n';
}
static void descriptorRows() {
  occ::handle<TableArena> arena=new TableArena;TableEngine<int> target(size_t(2),arena);
  tableRow("table_empty",target,*arena);
  for(int value=1;value<=5;++value) { target.Append(value);if(value!=4)tableRow(("table_append"+std::to_string(value)).c_str(),target,*arena); }
  target.ensureBlockCount(6,sizeof(int));tableRow("table_reserve6",target,*arena);
  target.ensureBlockCount(9,sizeof(int));tableRow("table_reserve9",target,*arena);
  target.Clear(false);tableRow("table_clear_retain",target,*arena);
  target.ensureBlockCount(5,sizeof(int));tableRow("table_smaller_reserve",target,*arena);
  target.Append(42);tableRow("table_reuse",target,*arena);
  target.Clear(true);tableRow("table_clear_release",target,*arena);
  target.expandBlock(sizeof(int));tableRow("table_expand_empty",target,*arena);
  target.clearBlocks();tableRow("table_clear_blocks",target,*arena);
  target.ensureBlockCount(6,sizeof(int));tableRow("table_initial_reserve6",target,*arena);
}
static void copyRows() {
  occ::handle<TableArena> arena=new TableArena;TableEngine<int> source(size_t(2),arena);
  source.ensureBlockCount(7,sizeof(int));source.Append(7);tableRow("copy_source",source,*arena);
  TableEngine<int> copied(source);tableRow("copy_constructed",copied,*arena);
  copied.Clear(true);tableRow("copy_cleared",copied,*arena);
  copied.ensureBlockCount(10,sizeof(int));tableRow("copy_destination_reserved",copied,*arena);
  copied.Assign(source);tableRow("copy_assigned",copied,*arena);
  TableEngine<int> moved(std::move(copied));tableRow("move_constructed",moved,*arena);
  tableRow("move_source",copied,*arena);
}
struct DestructionItem {
  int value;TableArena* arena;std::string* trace;
  DestructionItem(int input,TableArena* owner,std::string* events):value(input),arena(owner),trace(events) {}
  ~DestructionItem() { if(!trace->empty())*trace+=',';*trace+="d"+std::to_string(value)+"@"+std::to_string(arena->live()); }
};
static void clearCase(const char* name,bool release,bool scopedDrop) {
  occ::handle<TableArena> arena=new TableArena;std::string trace;
  {
    TableEngine<DestructionItem> target(size_t(2),arena);
    for(int value=1;value<=5;++value)target.EmplaceAppend(value,arena.get(),&trace);
    if(!scopedDrop) { target.Clear(release);std::cout<<name<<' '<<arena->live()<<' '<<trace<<'\n'; }
  }
  if(scopedDrop)std::cout<<name<<' '<<arena->live()<<' '<<trace<<'\n';
}
struct FailingItem {
  explicit FailingItem(bool fail) { if(fail)throw std::runtime_error("constructor"); }
};
static void failureRows() {
  occ::handle<TableArena> arena=new TableArena;TableEngine<FailingItem> target(size_t(2),arena);
  try { target.EmplaceAppend(true); }catch(const std::runtime_error&) {}
  tableRow("append_constructor_failed",target,*arena);
  target.EmplaceAppend(false);tableRow("append_constructor_retry",target,*arena);
  target.Clear(true);tableRow("append_constructor_cleared",target,*arena);
}
int main() {
  descriptorRows();copyRows();failureRows();
  clearCase("clear_retain_phases",false,false);clearCase("clear_release_phases",true,false);
  clearCase("destructor_phases",true,true);
}
