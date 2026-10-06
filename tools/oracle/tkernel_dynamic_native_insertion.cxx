#include <NCollection_DynamicArray.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct Arena : NCollection_BaseAllocator {
  std::string events;
  void* AllocateOptimal(size_t n) override { events += "A;"; return NCollection_BaseAllocator::AllocateOptimal(n); }
};
static Arena* arena;
static int failDefault, failMove, moveCalls, failCopy;
struct Item {
  int value;
  Item():value(0) { arena->events += "D;"; if(failDefault) throw std::runtime_error("default"); }
  explicit Item(int v):value(v) {}
  Item(const Item& x):value(x.value) { arena->events += "CC;"; }
  Item(Item&& x):value(x.value) { arena->events += "MC;"; x.value=-7; }
  Item& operator=(const Item& x) {
    arena->events += "C("+std::to_string(value)+","+std::to_string(x.value)+");";
    if(failCopy) throw std::runtime_error("copy");
    value=x.value; return *this;
  }
  Item& operator=(Item&& x) {
    arena->events += "M("+std::to_string(value)+","+std::to_string(x.value)+");";
    if(++moveCalls==failMove) throw std::runtime_error("move");
    value=x.value; x.value=-7; return *this;
  }
};
static void run(const char* name, bool after, bool moving, int index, int defaultFail, int moveFail, int copyFail) {
  occ::handle<Arena> allocator=new Arena; arena=allocator.get();
  NCollection_DynamicArray<Item> a(2,allocator);
  for(int i=1;i<=4;++i) a.Append(Item(i*10));
  Item input(99); arena->events.clear(); moveCalls=0;
  failDefault=defaultFail; failMove=moveFail; failCopy=copyFail;
  try {
    if(after) { if(moving) a.InsertAfter(index,std::move(input)); else a.InsertAfter(index,input); }
    else { if(moving) a.InsertBefore(index,std::move(input)); else a.InsertBefore(index,input); }
  } catch(...) { arena->events += "E;"; }
  std::cout<<name<<' '<<arena->events<<' '<<a.Size()<<' '<<input.value;
  for(size_t i=0;i<a.Size();++i) std::cout<<' '<<a[i].value;
  std::cout<<'\n';
}
int main() {
  run("before_copy",false,false,1,0,0,0);
  run("after_move",true,true,1,0,0,0);
  run("after_last_copy",true,false,3,0,0,0);
  run("before_negative",false,false,-1,0,0,0);
  run("after_end",true,true,4,0,0,0);
  run("default_failure",false,false,1,1,0,0);
  run("first_move_failure",false,false,1,0,1,0);
  run("second_move_failure",false,false,1,0,2,0);
  run("final_copy_failure",false,false,1,0,0,1);
  run("final_move_failure",false,true,1,0,4,0);
  run("before_first_move",false,true,0,0,0,0);
  run("after_negative",true,false,-1,0,0,0);
}
