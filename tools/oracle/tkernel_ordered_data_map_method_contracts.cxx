// Supplemental OrderedDataMap contracts from the pinned native library.
#include <NCollection_OrderedDataMap.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <string>
#include <utility>
struct TraceCopy {
  int number;
  std::string label;
  static std::string events;
  TraceCopy(int n, const char* text) : number(n), label(text) {}
  TraceCopy(const TraceCopy& other) : number(other.number), label(other.label) { events += label + ','; }
  TraceCopy(TraceCopy&& other) : number(other.number), label(std::move(other.label)) {}
  TraceCopy& operator=(TraceCopy&& other) { number=other.number; label=std::move(other.label); return *this; }
};
std::string TraceCopy::events;
struct TraceHash {
  size_t operator()(const TraceCopy& key) const { TraceCopy::events += "hash,"; return key.number; }
  bool operator()(const TraceCopy& a, const TraceCopy& b) const { return a.number == b.number; }
};
struct SaltHash {
  int salt;
  size_t operator()(int key) const { return static_cast<size_t>(key + salt); }
  bool operator()(int a, int b) const { return a==b; }
};
void borrowing() {
  NCollection_OrderedDataMap<TraceCopy,TraceCopy,TraceHash> map;
  TraceCopy key(1,"key"), item(10,"item");
  TraceCopy::events.clear(); bool added=map.Bind(key,item);
  std::cout << "bind_new " << added << ' ' << TraceCopy::events << '\n';
  TraceCopy::events.clear(); added=map.Bind(key,item);
  std::cout << "bind_hit " << added << ' ' << TraceCopy::events << '\n';
  const TraceCopy* stored=map.Seek(key);
  TraceCopy::events.clear(); const bool same=map.Bound(key,item)==stored;
  std::cout << "bound_hit " << same << ' ' << TraceCopy::events << '\n';
  TraceCopy::events.clear(); added=map.TryBind(key,item);
  const bool sameTry=&map.TryBound(key,item)==stored;
  std::cout << "try_hit " << added << ' ' << sameTry << ' ' << TraceCopy::events << '\n';
}
void copyOrder() {
  NCollection_OrderedDataMap<TraceCopy,TraceCopy,TraceHash> source;
  source.Bind(TraceCopy(1,"key1"),TraceCopy(10,"item1"));
  source.Bind(TraceCopy(2,"key2"),TraceCopy(20,"item2"));
  NCollection_OrderedDataMap<TraceCopy,TraceCopy,TraceHash> target;
  TraceCopy::events.clear(); target.Assign(source);
  std::cout << "assign_order " << TraceCopy::events << '\n';
  TraceCopy::events.clear(); const auto copy=source;
  std::cout << "clone_order " << TraceCopy::events << ' ' << copy.FirstValue().number << '\n';
}
void assignment() {
  occ::handle<NCollection_BaseAllocator> arena = new NCollection_IncAllocator(4096);
  NCollection_OrderedDataMap<int,int,SaltHash> target(SaltHash{90},1009,arena);
  NCollection_OrderedDataMap<int,int,SaltHash> source(SaltHash{7});
  target.Bind(9,90); source.Bind(1,10); source.Bind(2,20);
  const bool returned=&target.Assign(source)==&target;
  std::cout << "assign " << returned << ' ' << target.GetHasher().salt << ' '
    << (target.Allocator()==arena) << ' ' << target.NbBuckets() << ' '
    << target.First() << ' ' << target.Last() << '\n';
  const bool selfCopy=&target.Assign(target)==&target;
  const bool selfMove=&(target=std::move(target))==&target;
  const bool moved=&(target=std::move(source))==&target;
  std::cout << "move " << selfCopy << ' ' << selfMove << ' ' << moved << ' '
    << target.GetHasher().salt << ' ' << source.GetHasher().salt << '\n';
}
void identities() {
  using Map=NCollection_OrderedDataMap<int,int>;
  Map first,second; first.Bind(1,10); second.Bind(1,10);
  Map::Iterator left(first),right(second);
  const bool distinct=!left.IsEqual(right);
  left.Next(); right.Next();
  std::cout << "identity " << distinct << ' ' << left.IsEqual(right) << ' '
    << left.IsEqual(Map::Iterator()) << '\n';
}
void cursorOperations() {
  using Map=NCollection_OrderedDataMap<int,int>;
  Map map; map.Bind(1,10); map.Bind(2,20);
  Map::Iterator cursor(map),saved(cursor);
  const auto previous=cursor++;
  const int previousKey=previous.Key();
  const int nextValue=cursor.Value(); cursor.ChangeValue()=21;
  const bool prefix=&(++cursor)==static_cast<Map::iterator*>(&cursor);
  const bool exhausted=!cursor.More();
  const bool copied=&(cursor=saved)==&cursor;
  cursor.Reset();
  std::cout << "cursor " << previousKey << ' ' << nextValue << ' ' << prefix << ' '
    << exhausted << ' ' << copied << ' ' << cursor.Key() << '\n';
  map.UnBind(1); cursor.Reset();
  Map::Iterator detached; detached.Reset(); const bool stayed=!detached.More();
  detached.Initialize(map);
  std::cout << "reset " << cursor.Key() << ' ' << stayed << ' ' << detached.IsEqual(cursor) << '\n';
}
int main() { borrowing(); copyOrder(); assignment(); identities(); cursorOperations(); }
