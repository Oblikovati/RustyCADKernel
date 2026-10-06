// Pinned SeqNode and public Sequence<T>::Node allocation and link contracts.
#include <NCollection_Sequence.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <string>
struct ProbeAllocator : NCollection_BaseAllocator {
  std::string events;
  void* Allocate(size_t n) override { events+='A';return NCollection_BaseAllocator::Allocate(n); }
  void Free(void* p) override { events+='F';NCollection_BaseAllocator::Free(p); }
};
struct Item {
  int value;std::string* events;
  Item(int v,std::string& e):value(v),events(&e){e+='V';}
  Item(const Item& other):value(other.value),events(other.events){*events+='C';}
  ~Item(){*events+='D';}
};
void links() {
  NCollection_SeqNode a,b,c;
  std::cout<<"default "<<(a.Next()==nullptr)<<' '<<(a.Previous()==nullptr)<<'\n';
  a.SetNext(&b);a.SetPrevious(&c);NCollection_SeqNode copied(a),assigned;assigned=a;
  std::cout<<"links "<<(a.Next()==&b)<<' '<<(a.Previous()==&c)<<' '
    <<(copied.Next()==&b)<<' '<<(assigned.Previous()==&c)<<'\n';
  a.SetNext(nullptr);a.SetPrevious(nullptr);
  std::cout<<"reset "<<(a.Next()==nullptr)<<' '<<(a.Previous()==nullptr)<<' '
    <<(copied.Next()==&b)<<'\n';
}
void values() {
  NCollection_Sequence<std::string>::Node a(std::in_place,3,'x');
  a.ChangeValue()+='y';NCollection_SeqNode next;a.SetNext(&next);
  NCollection_Sequence<std::string>::Node copied(a),assigned(std::string("old"));assigned=a;
  std::cout<<"values "<<a.Value()<<' '<<copied.Value()<<' '<<assigned.Value()<<' '
    <<(copied.Next()==&next)<<' '<<(assigned.Next()==&next)<<'\n';
}
void allocation() {
  occ::handle<ProbeAllocator> concrete=new ProbeAllocator;
  occ::handle<NCollection_BaseAllocator> arena=concrete;
  auto node=new(arena) NCollection_Sequence<Item>::Node(std::in_place,7,concrete->events);
  std::cout<<"allocate "<<concrete->events<<' '<<node->Value().value<<'\n';
  NCollection_SeqNode neighbor;node->SetNext(&neighbor);
  NCollection_Sequence<Item>::delNode(node,arena);
  std::cout<<"delete "<<concrete->events<<' '<<(neighbor.Next()==nullptr)<<'\n';
  Item source(8,concrete->events);concrete->events.clear();
  node=new(arena) NCollection_Sequence<Item>::Node(source);
  std::cout<<"borrowed "<<concrete->events<<' '<<(node->Next()==nullptr)<<'\n';
  NCollection_Sequence<Item>::delNode(node,arena);
  std::cout<<"borrowed_delete "<<concrete->events<<'\n';
}
int main(){links();values();allocation();}
