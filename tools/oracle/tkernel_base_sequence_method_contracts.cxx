// Native Sequence allocation/copy failures and BaseSequence iterator contracts.
#include <NCollection_Sequence.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>
struct ProbeAllocator : NCollection_BaseAllocator {
  std::string events;
  void* Allocate(size_t n) override { events+='A'; return NCollection_BaseAllocator::Allocate(n); }
  void Free(void* p) override { events+='F'; NCollection_BaseAllocator::Free(p); }
};
struct Item {
  int value; ProbeAllocator* arena; bool fail;
  Item(int v,ProbeAllocator* a,bool f=false):value(v),arena(a),fail(f) { arena->events+='V'; }
  Item(const Item& other):value(other.value),arena(other.arena),fail(other.fail) {
    arena->events+='C'; if(fail) throw std::runtime_error("copy failure");
  }
};
void allocation() {
  occ::handle<ProbeAllocator> arena=new ProbeAllocator;
  NCollection_Sequence<Item> sequence(arena);
  sequence.EmplaceAppend(1,arena.get()); std::cout<<"emplace_append "<<arena->events<<'\n';
  arena->events.clear(); sequence.EmplacePrepend(0,arena.get());
  std::cout<<"emplace_prepend "<<arena->events<<'\n';
  arena->events.clear(); sequence.EmplaceAfter(1,2,arena.get());
  std::cout<<"emplace_after "<<arena->events<<'\n';
  NCollection_Sequence<Item>::Iterator position(sequence);
  arena->events.clear();sequence.EmplaceAfter(position,3,arena.get());
  std::cout<<"emplace_cursor "<<arena->events<<'\n';
  Item item(4,arena.get());arena->events.clear();sequence.Append(item);
  std::cout<<"borrowed_append "<<arena->events<<'\n';
  Item bad(5,arena.get(),true);arena->events.clear();
  try { sequence.Append(bad); } catch(const std::runtime_error&) {}
  std::cout<<"failed_copy "<<arena->events<<' '<<sequence.Size()<<'\n';
}
void copyFailure(const std::string& mode) {
  occ::handle<ProbeAllocator> targetArena=new ProbeAllocator;
  occ::handle<ProbeAllocator> sourceArena=new ProbeAllocator;
  NCollection_Sequence<Item> source(sourceArena),target(targetArena);
  source.EmplaceAppend(1,targetArena.get());source.EmplaceAppend(2,targetArena.get(),true);
  source.EmplaceAppend(3,targetArena.get()); target.EmplaceAppend(9,targetArena.get());
  targetArena->events.clear();
  try {
    if(mode=="append") target.Append(source);
    else if(mode=="prepend") target.Prepend(source);
    else if(mode=="insert") target.InsertAfter(1,source);
    else target.Assign(source);
  } catch(const std::runtime_error&) {}
  std::cout<<mode<<"_failure "<<targetArena->events<<' '<<source.Size()<<' '<<target.Size();
  for(const Item& item:target) std::cout<<' '<<item.value;
  std::cout<<'\n';
}
void cursors() {
  NCollection_Sequence<int> sequence;sequence.Append(1);sequence.Append(2);sequence.Append(3);
  NCollection_Sequence<int>::iterator writable(sequence);auto old=writable++;
  NCollection_Sequence<int>::const_iterator readonly(writable);
  NCollection_Sequence<int>::iterator converted(readonly);
  NCollection_BaseSequence::Iterator base;readonly.CopyTo(base);
  NCollection_Sequence<int>::iterator restored(base);
  std::cout<<"cursor_conversion "<<*old<<' '<<*converted<<' '<<(converted==readonly)<<' '
    <<(base.CurrentNode()==writable.Node())<<' '<<(base.PreviousNode()==old.Node())<<' '<<*restored<<'\n';
  NCollection_Sequence<int>::iterator end(sequence,false);auto past=end--;*end=8;
  std::cout<<"cursor_end "<<past.More()<<' '<<*end<<' '<<sequence.Last()<<' '
    <<(end.MutableNode()==end.Node())<<' '<<(end.MutablePreviousNode()==writable.Node())<<'\n';
}
int main() {allocation();copyFailure("append");copyFailure("prepend");copyFailure("insert");copyFailure("assign");cursors();}
