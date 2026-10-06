// Supplemental method contracts compiled against the pinned OCCT library.
#include <NCollection_IndexedDataMap.hxx>
#include <NCollection_IncAllocator.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <string>
#include <utility>
struct SaltPolicy {
  int salt;
  size_t operator()(int key) const { return static_cast<size_t>(key + salt); }
  bool operator()(int a, int b) const { return a == b; }
};
struct CopyTrace {
  int number;
  std::string label;
  static std::string events;
  CopyTrace(int n, const char* text) : number(n), label(text) {}
  CopyTrace(const CopyTrace& other) : number(other.number), label(other.label) { events += label + ','; }
  CopyTrace(CopyTrace&& other) : number(other.number), label(std::move(other.label)) {}
  CopyTrace& operator=(const CopyTrace& other) { events += "assign,"; number=other.number; return *this; }
};
std::string CopyTrace::events;
struct TracePolicy {
  size_t operator()(const CopyTrace& key) const { return key.number; }
  bool operator()(const CopyTrace& a, const CopyTrace& b) const { return a.number == b.number; }
};
void assignment() {
  occ::handle<NCollection_BaseAllocator> arena = new NCollection_IncAllocator(4096);
  NCollection_IndexedDataMap<int,int,SaltPolicy> target(SaltPolicy{90},1009,arena);
  NCollection_IndexedDataMap<int,int,SaltPolicy> source(SaltPolicy{7});
  target.Add(99,990); source.Add(3,30); source.Add(4,40);
  const bool returned = &target.Assign(source) == &target;
  std::cout << "assign " << returned << ' ' << target.GetHasher().salt << ' '
    << (target.Allocator()==arena) << ' ' << target.NbBuckets() << ' '
    << target.FindKey(1) << ' ' << target.FindFromIndex(1) << ' '
    << target.FindKey(2) << ' ' << target.FindFromIndex(2) << '\n';
  NCollection_IndexedDataMap<int,int,SaltPolicy> empty(SaltPolicy{0});
  target.Assign(empty);
  std::cout << "empty " << target.IsEmpty() << ' ' << target.NbBuckets() << '\n';
}
void copyOrder() {
  NCollection_IndexedDataMap<CopyTrace,CopyTrace,TracePolicy> source;
  source.Add(CopyTrace(1,"key1"),CopyTrace(10,"item1"));
  source.Add(CopyTrace(2,"key2"),CopyTrace(20,"item2"));
  NCollection_IndexedDataMap<CopyTrace,CopyTrace,TracePolicy> target;
  CopyTrace::events.clear(); target.Assign(source);
  std::cout << "order " << CopyTrace::events << '\n';
}
void borrowed() {
  NCollection_IndexedDataMap<CopyTrace,CopyTrace,TracePolicy> map;
  CopyTrace key(1,"key"), item(10,"item"), replacement(20,"replacement");
  CopyTrace::events.clear(); bool inserted = map.Bind(key,item);
  std::cout << "bind_new " << inserted << ' ' << CopyTrace::events << '\n';
  const CopyTrace* stored = &map.FindFromIndex(1);
  CopyTrace::events.clear(); inserted = map.Bind(key,replacement);
  std::cout << "bind_hit " << inserted << ' ' << CopyTrace::events << ' ' << map.FindFromIndex(1).number << '\n';
  CopyTrace::events.clear(); const bool same = map.Bound(key,item) == stored;
  std::cout << "bound " << same << ' ' << CopyTrace::events << ' ' << map.FindFromIndex(1).number << '\n';
  CopyTrace::events.clear(); inserted = map.TryBind(key,replacement);
  const CopyTrace& found = map.TryBound(key,replacement);
  std::cout << "try_hit " << inserted << ' ' << found.number << ' ' << CopyTrace::events.empty() << '\n';
}
void moving() {
  occ::handle<NCollection_BaseAllocator> arena = new NCollection_IncAllocator(4096);
  NCollection_IndexedDataMap<int,int,SaltPolicy> source(SaltPolicy{7},1,arena);
  NCollection_IndexedDataMap<int,int,SaltPolicy> target(SaltPolicy{90},1009);
  source.Add(3,30); target.Add(99,990);
  const int* original = &source.FindFromIndex(1);
  const int* replaced = &target.FindFromIndex(1);
  const bool returned = &(target = std::move(source)) == &target;
  std::cout << "move " << returned << ' ' << (&target.FindFromIndex(1)==original) << ' '
    << (&source.FindFromIndex(1)==replaced) << ' ' << target.GetHasher().salt << ' '
    << source.GetHasher().salt << ' ' << (target.Allocator()==arena) << '\n';
  const bool selfCopy = &target.Assign(target) == &target;
  const bool selfMove = &(target=std::move(target)) == &target;
  std::cout << "self " << selfCopy << ' ' << selfMove << ' ' << (&target.FindFromIndex(1)==original) << '\n';
}
void signedCounts() {
  int rejected = 0;
  try { NCollection_IndexedDataMap<int,int> invalid(-7); }
  catch(const Standard_OutOfRange&) { ++rejected; }
  NCollection_IndexedDataMap<int,int> map(0);
  try { map.ReSize(-7); } catch(const Standard_OutOfRange&) { ++rejected; }
  const size_t before = map.NbBuckets(); map.ReSize(0);
  std::cout << "counts " << rejected << ' ' << before << ' ' << map.NbBuckets() << '\n';
}
void signedIndices() {
  NCollection_IndexedDataMap<int,int> map;
  map.Add(10,100); int rejected = 0;
  for(int index : {-7,0,2}) {
    try { map.FindKey(index); } catch(const Standard_OutOfRange&) { ++rejected; }
    try { map.FindFromIndex(index); } catch(const Standard_OutOfRange&) { ++rejected; }
    try { map.ChangeFromIndex(index); } catch(const Standard_OutOfRange&) { ++rejected; }
    try { map.Substitute(index,20,200); } catch(const Standard_OutOfRange&) { ++rejected; }
    try { map.Swap(index,1); } catch(const Standard_OutOfRange&) { ++rejected; }
    try { map.Swap(1,index); } catch(const Standard_OutOfRange&) { ++rejected; }
    try { map.RemoveFromIndex(index); } catch(const Standard_OutOfRange&) { ++rejected; }
  }
  map.ChangeFromIndex(1) = 101;
  std::cout << "indices " << rejected << ' ' << map.FindKey(1) << ' ' << map.FindFromIndex(1) << '\n';
}
void mutations() {
  NCollection_IndexedDataMap<int,int> map;
  map.Add(10,100); map.Add(20,200); map.Add(30,300); map.Substitute(1,11,110);
  bool rejected = false;
  try { map.Substitute(1,20,201); } catch(const Standard_DomainError&) { rejected = true; }
  map.Swap(1,3); map.RemoveFromIndex(1);
  std::cout << "mutate " << rejected << ' ' << map.FindKey(1) << ' ' << map.FindFromIndex(1)
    << ' ' << map.FindKey(2) << ' ' << map.FindFromIndex(2) << '\n';
}
int main() { assignment(); copyOrder(); borrowed(); moving(); signedCounts(); signedIndices(); mutations(); }
