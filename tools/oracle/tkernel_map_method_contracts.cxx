// Native Map class facade, forwarding, and MapAlgo key-copy ordering.
#include <NCollection_Map.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <algorithm>
#include <vector>
#include <string>
#include <stdexcept>
using Map = NCollection_Map<int>;
static Map keys(std::initializer_list<int> values) {
  Map map; for (int value : values) { map.Add(value); } return map;
}
static void print(const char* name, const Map& map) {
  std::vector<int> values(map.begin(),map.end());
  std::sort(values.begin(),values.end());
  std::cout << name; for (int value : values) { std::cout << ' ' << value; } std::cout << '\n';
}
static void algebra() {
  Map a=keys({1,2}), b=keys({2,3}), subset=keys({1}), equal=keys({2,1}), out=keys({99});
  std::cout << "queries " << a.IsEqual(equal) << ' ' << a.IsEqual(subset) << ' ' << a.Contains(subset) << ' ' << subset.Contains(a) << ' ' << a.HasIntersection(subset) << '\n';
  out.Union(a,b); print("union",out);
  out.Intersection(a,b); print("intersection",out);
  out.Subtraction(a,b); print("subtraction",out);
  out.Difference(a,b); print("difference",out);
  out.Difference(a,a); print("difference_same",out);
  std::cout << "mutation " << a.Unite(b) << ' ' << a.Unite(b) << ' ' << a.Intersect(b) << ' ' << a.Intersect(b) << ' ' << a.Subtract(b) << ' ' << a.Subtract(b) << ' ' << a.Differ(b) << ' ' << a.Differ(b) << '\n';
  a=keys({1,2}); std::cout << "differ_equal_size " << a.Differ(b) << '\n'; print("differ_keys",a);
}
struct FixedPolicy {
  int salt;
  explicit FixedPolicy(int value=0):salt(value){}
  size_t operator()(int value) const { return value+salt; }
  bool operator()(int a,int b) const { return a==b; }
};
static void assignments() {
  using PolicyMap=NCollection_Map<int,FixedPolicy>;
  occ::handle<NCollection_IncAllocator> arena = new NCollection_IncAllocator;
  PolicyMap source(FixedPolicy(11),101), target(FixedPolicy(17),1009,arena);
  source.Add(1); target.Add(8);
  const bool same=&target.Assign(source)==&target;
  std::cout << "assignment " << same << ' ' << target.GetHasher().salt << ' ' << (target.Allocator()==arena) << ' ' << target.NbBuckets() << ' ' << target.Contains(1) << '\n';
  const int* node=&source.Added(1);
  const bool moved=&(target=std::move(source))==&target;
  std::cout << "move " << moved << ' ' << (&target.Added(1)==node) << ' ' << target.GetHasher().salt << ' ' << source.GetHasher().salt << '\n';
  std::cout << "self " << (&target.Assign(target)==&target) << '\n';
}
static void emplacementVariants() {
  Map map;
  const bool first=map.Emplace(42), second=map.Emplace(42);
  const int* node=&map.Added(42);
  const bool same=&map.Emplaced(42)==node, kept=!map.TryEmplace(42), tried=&map.TryEmplaced(42)==node;
  const bool newTry=map.TryEmplace(43);
  std::cout << "emplacement " << first << ' ' << second << ' ' << same << ' ' << kept << ' ' << tried << ' ' << newTry << ' ' << map.TryEmplaced(44) << ' ' << map.Emplaced(45) << '\n';
}
static void signedCountsAndCursor() {
  bool badCtor=false,badResize=false;
  try { Map bad(-1); } catch (const Standard_OutOfRange&) { badCtor=true; }
  Map map(0); map.Add(42);
  try { map.ReSize(-1); } catch (const Standard_OutOfRange&) { badResize=true; }
  map.ReSize(101);
  std::cout << "signed " << badCtor << ' ' << badResize << ' ' << map.Contains(42) << ' ' << map.NbBuckets() << '\n';
  Map::Iterator cursor(map),copy(cursor);
  const bool oldEqual=cursor++==copy;
  cursor.Reset(); const bool same=&(++cursor)==&cursor;
  std::cout << "cursor " << oldEqual << ' ' << same << ' ' << cursor.More() << '\n';
}
static std::string events;
struct TraceKey {
  int number;
  explicit TraceKey(int value):number(value){}
  TraceKey(const TraceKey& source):number(source.number){events+='K';}
  TraceKey(TraceKey&&)=default;
};
struct TracePolicy {
  size_t operator()(const TraceKey& key) const { events+='H'; return key.number; }
  bool operator()(const TraceKey& a,const TraceKey& b) const { events+='E'; return a.number==b.number; }
};
static void copyOrder() {
  using TraceMap=NCollection_Map<TraceKey,TracePolicy>;
  TraceMap left(101),right(101),out(101);
  TraceKey key(1); left.Add(key); right.Add(key); events.clear();
  out.Union(left,right); std::cout << "union_copies " << events << '\n'; events.clear();
  out.Unite(right); std::cout << "unite_copies " << events << '\n'; events.clear();
  out.TryEmplace(key); std::cout << "try_emplace_constructor " << events << '\n'; events.clear();
}
struct ThrowKey {
  explicit ThrowKey(bool fail) { if(fail) throw std::runtime_error("injected key constructor failure"); }
};
struct ThrowPolicy {
  size_t operator()(const ThrowKey&) const {return 0;}
  bool operator()(const ThrowKey&,const ThrowKey&) const {return true;}
};
static void failingConstructor() {
  NCollection_Map<ThrowKey,ThrowPolicy> map(0);
  try {map.Emplace(true);} catch(const std::runtime_error&){}
  std::cout << "constructor_failure " << map.IsEmpty() << ' ' << map.NbBuckets() << '\n';
}
int main(){algebra();assignments();emplacementVariants();signedCountsAndCursor();copyOrder();failingConstructor();}
