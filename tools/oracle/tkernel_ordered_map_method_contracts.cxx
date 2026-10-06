// OrderedMap source-level method contracts against pinned OCCT.
#include <NCollection_OrderedMap.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <string>
#include <utility>
struct TraceKey {
  int number;
  static std::string events;
  explicit TraceKey(int n):number(n) {}
  TraceKey(const TraceKey& other):number(other.number) { events+="copy,"; }
  TraceKey(TraceKey&& other):number(other.number) {}
  TraceKey& operator=(TraceKey&& other) { number=other.number; return *this; }
};
std::string TraceKey::events;
struct TraceHash {
  int modulus;
  size_t operator()(const TraceKey& key) const { TraceKey::events+="hash,"; return key.number%modulus; }
  bool operator()(const TraceKey& a,const TraceKey& b) const { return a.number%modulus==b.number%modulus; }
};
struct SaltHash {
  int salt;
  size_t operator()(int key) const { return static_cast<size_t>(key+salt); }
  bool operator()(int a,int b) const { return a==b; }
};
void borrowing() {
  NCollection_OrderedMap<TraceKey,TraceHash> map(TraceHash{100});
  TraceKey input(1);
  TraceKey::events.clear(); const bool added=map.Add(input);
  std::cout<<"add_new "<<added<<' '<<TraceKey::events<<'\n';
  const TraceKey* stored=&map.First();
  TraceKey::events.clear(); const bool duplicate=map.Add(input);
  const bool same=&map.Added(input)==stored;
  std::cout<<"add_hit "<<duplicate<<' '<<same<<' '<<TraceKey::events<<'\n';
  TraceKey second(2); TraceKey::events.clear(); const int last=map.Added(second).number;
  std::cout<<"added_new "<<last<<' '<<TraceKey::events<<'\n';
  TraceKey::events.clear(); map.Add(TraceKey(3));
  std::cout<<"owned_add "<<TraceKey::events<<'\n';
  TraceKey::events.clear(); map.Emplace(4);
  std::cout<<"owned_emplace "<<TraceKey::events<<'\n';
}
void copying() {
  NCollection_OrderedMap<TraceKey,TraceHash> source(TraceHash{100});
  source.Add(TraceKey(1)); source.Add(TraceKey(11));
  NCollection_OrderedMap<TraceKey,TraceHash> target(TraceHash{10});
  TraceKey::events.clear(); target.Assign(source);
  std::cout<<"assign_collapse "<<target.Extent()<<' '<<target.First().number<<' '<<TraceKey::events<<'\n';
  TraceKey::events.clear(); const auto copied=source;
  std::cout<<"clone "<<copied.Extent()<<' '<<TraceKey::events<<'\n';
}
void assignments() {
  occ::handle<NCollection_BaseAllocator> arena=new NCollection_IncAllocator(4096);
  NCollection_OrderedMap<int,SaltHash> source(SaltHash{7}); source.Add(3);
  NCollection_OrderedMap<int,SaltHash> target(SaltHash{90},1009,arena); target.Add(9);
  const bool returned=&target.Assign(source)==&target;
  std::cout<<"assign "<<returned<<' '<<target.GetHasher().salt<<' '<<target.NbBuckets()<<' '
    <<(target.Allocator()==arena)<<'\n';
  const int* stored=&target.First();
  const bool selfCopy=&target.Assign(target)==&target;
  const bool selfMove=&(target=std::move(target))==&target;
  const bool moved=&(target=std::move(source))==&target;
  std::cout<<"move "<<selfCopy<<' '<<selfMove<<' '<<moved<<' '<<target.GetHasher().salt<<' '
    <<source.GetHasher().salt<<' '<<(&source.First()==stored)<<'\n';
  NCollection_OrderedMap<int> full(1009),empty; full.Add(3); full.Assign(empty);
  std::cout<<"empty "<<full.IsEmpty()<<' '<<full.NbBuckets()<<' '<<full.Add(7)<<'\n';
}
void iterators() {
  using Map=NCollection_OrderedMap<int>;
  Map map; map.Add(3); map.Add(7);
  Map::Iterator cursor(map),saved(cursor);
  const auto previous=cursor++;
  const int oldKey=previous.Key(),nextKey=cursor.Value();
  const bool prefix=&(++cursor)==static_cast<Map::iterator*>(&cursor);
  const bool ended=cursor.IsEqual(Map::Iterator());
  const bool assigned=&(cursor=saved)==&cursor;
  const bool self=&(cursor=cursor)==&cursor;
  cursor.Next(); cursor.Reset();
  std::cout<<"cursor "<<oldKey<<' '<<nextKey<<' '<<prefix<<' '<<ended<<' '<<assigned<<' '
    <<self<<' '<<cursor.Key()<<'\n';
  Map other; other.Add(3);
  Map::Iterator different(other),detached;
  detached.Reset(); const bool stayed=!detached.More(); detached.Initialize(map);
  std::cout<<"identity "<<(!saved.IsEqual(different))<<' '<<stayed<<' '<<detached.IsEqual(saved)<<'\n';
}
int main() { borrowing(); copying(); assignments(); iterators(); }
