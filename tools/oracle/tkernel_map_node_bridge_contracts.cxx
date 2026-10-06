// OCCT 3d097a: MapNode is a real TListNode<K>, and Map iteration dereferences its key.
#include <NCollection_Map.hxx>
#include <NCollection_DataMap.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct Arena:NCollection_BaseAllocator {
  int live=0;
  void* Allocate(size_t bytes) override {++live;return NCollection_BaseAllocator::Allocate(bytes);}
  void Free(void* address) override {--live;NCollection_BaseAllocator::Free(address);}
};
struct Key {
  int value;
  static std::string events;
  static Arena* arena;
  static bool allocated,reject;
  explicit Key(int v):value(v) {}
  Key(const Key& other):value(other.value) {
    events+="C,";allocated=arena&&arena->live==1;
    if(reject) throw std::runtime_error("copy rejected");
  }
  Key(Key&& other) noexcept:value(other.value) {other.value=-1;}
  ~Key() {if(value!=-1)events+="D,";}
};
std::string Key::events;Arena* Key::arena=nullptr;bool Key::allocated=false,Key::reject=false;
using Node=NCollection_Map<Key>::MapNode;
void constructors() {
  Key key(7);Key::events.clear();{Node node(key,nullptr);}
  std::cout<<"borrowed "<<Key::events<<'\n';
  Key::events.clear();{Node node(Key(7),nullptr);}
  std::cout<<"owned "<<Key::events<<'\n';
  Key::events.clear();{Node node(std::in_place,nullptr,7);}
  std::cout<<"in_place "<<Key::events<<'\n';
}
void prefix() {
  NCollection_ListNode tail(nullptr);NCollection_Map<int>::MapNode node(7,&tail);
  const auto& readonly=node;auto* base=static_cast<NCollection_TListNode<int>*>(&node);
  std::cout<<"prefix "<<(static_cast<void*>(base)==static_cast<void*>(&node))<<' '
    <<(node.Next()==&tail)<<' '<<(&node.Key()==&node.Value())<<' '
    <<(&readonly.Key()==&readonly.Value())<<' ';
  node.ChangeValue()=9;node.Next()=nullptr;
  std::cout<<node.Key()<<' '<<(base->Next()==nullptr)<<'\n';
}
void allocation() {
  occ::handle<Arena> concrete=new Arena;occ::handle<NCollection_BaseAllocator> arena=concrete;
  Key::arena=concrete.get();Key key(7);Key::events.clear();
  Node* node=new(arena) Node(key,nullptr);
  std::cout<<"allocate "<<Key::events<<' '<<Key::allocated<<' '<<concrete->live<<'\n';
  Key::events.clear();Node::delNode(node,arena);
  std::cout<<"delete "<<Key::events<<' '<<concrete->live<<'\n';
  Key::events.clear();Key::reject=true;
  try {new(arena) Node(key,nullptr);}catch(const std::runtime_error&){}
  std::cout<<"copy_failure "<<Key::events<<' '<<concrete->live<<'\n';
  Key::reject=false;Key::arena=nullptr;
}
struct Collision {
  size_t operator()(int) const {return 0;}
  bool operator()(int a,int b) const {return a==b;}
};
void key_iterator() {
  NCollection_Map<int,Collision> map;map.Add(1);map.Add(2);
  auto cursor=map.begin();auto old=cursor++;
  std::cout<<"key_iterator "<<*old<<' '<<old.Key()<<' '<<*cursor<<' '
    <<(old.operator->()==&*old)<<' ';
  ++cursor;std::cout<<(cursor==map.end())<<'\n';
}
void data_iterator() {
  NCollection_DataMap<int,int,Collision> map;map.Bind(1,10);map.Bind(2,20);
  auto cursor=map.begin();*cursor=21;
  auto old=cursor++;*cursor=11;
  std::cout<<"data_iterator "<<old.Key()<<' '<<*old<<' '<<cursor.Key()<<' '<<*cursor<<' '
    <<(old.operator->()==&map.ChangeFind(2))<<'\n';
  const auto& readonly=map;auto first=readonly.cbegin();
  std::cout<<"const_iterator "<<first.Key()<<' '<<*first<<' '<<(first.Node()==old.Node())<<'\n';
}
int main(){constructors();prefix();allocation();key_iterator();data_iterator();}
