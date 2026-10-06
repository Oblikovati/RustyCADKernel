#include <NCollection_DataMap.hxx>
#include <NCollection_IncAllocator.hxx>
#include <functional>
#include <iostream>
#include <string>
struct ObservedItem {
  int value;
  static std::function<std::string()> observe;
  static std::string events;
  explicit ObservedItem(int v):value(v) {}
  ~ObservedItem() { if(observe) events += "D"+std::to_string(value)+":"+observe()+","; }
};
std::function<std::string()> ObservedItem::observe;
std::string ObservedItem::events;
int main() {
  occ::handle<NCollection_BaseAllocator> first=new NCollection_IncAllocator(1024);
  occ::handle<NCollection_BaseAllocator> second=new NCollection_IncAllocator(1024);
  NCollection_DataMap<int,ObservedItem> map(size_t(1),first);
  map.Bind(1,ObservedItem(10)); map.Bind(2,ObservedItem(20));
  ObservedItem::observe=[&] { return std::to_string(map.Size())+":"+std::to_string(map.Allocator()==first); };
  bool removed=map.UnBind(1);
  std::cout<<"unbind_observer "<<ObservedItem::events<<' '<<removed<<' '<<map.Size()<<'\n';
  ObservedItem::events.clear(); map.Clear(second);
  std::cout<<"allocator_clear_observer "<<ObservedItem::events<<' '<<map.Size()<<' '
           <<(map.Allocator()==second)<<' '<<(map.Buckets()==nullptr)<<'\n';
  ObservedItem::observe={};
}
