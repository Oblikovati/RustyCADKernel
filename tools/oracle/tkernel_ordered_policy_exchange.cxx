#include <NCollection_OrderedMap.hxx>
#include <NCollection_OrderedDataMap.hxx>
#include <NCollection_IncAllocator.hxx>
#include <iostream>
#include <stdexcept>
#include <string>

static std::string trace;
static int phase = 0, rejectedPhase = 0, payloadMoves = 0, payloadDrops = 0;
struct Payload
{
  int key;
  explicit Payload(int value) : key(value) {}
  Payload(const Payload&) = delete;
  Payload(Payload&& other) : key(other.key) { ++payloadMoves; }
  Payload& operator=(Payload&& other) { key=other.key; ++payloadMoves; return *this; }
  ~Payload() { ++payloadDrops; }
};
void event(char tag, int state)
{
  trace += tag + std::to_string(state) + ',';
  if (++phase == rejectedPhase) { trace += "X,"; throw std::runtime_error("policy phase"); }
}
struct Policy
{
  int state;
  explicit Policy(int value) : state(value) {}
  Policy(const Policy&) = delete;
  Policy(Policy&& source) : state(source.state)
  {
    source.state = -99;
    event('M', state);
  }
  Policy& operator=(Policy&& source)
  {
    state = source.state; source.state = -99;
    event('A', state); return *this;
  }
  ~Policy() { trace += 'D' + std::to_string(state) + ','; }
  size_t operator()(const Payload& value) const { return size_t(value.key % 3); }
  bool operator()(const Payload& a, const Payload& b) const { return a.key == b.key; }
};
using Set=NCollection_OrderedMap<Payload,Policy>;
using Map=NCollection_OrderedDataMap<Payload,Payload,Policy>;
void add(Set& map,int key) { map.Add(Payload(key)); }
void add(Map& map,int key) { map.Bind(Payload(key),Payload(key*10)); }
void reset(int rejected)
{
  trace.clear(); phase=0; rejectedPhase=rejected; payloadMoves=0; payloadDrops=0;
}
template<class Table> void run(const char* prefix, bool assign, int rejected, bool empty)
{
  reset(0);
  occ::handle<NCollection_BaseAllocator> aArena=new NCollection_IncAllocator(1024);
  occ::handle<NCollection_BaseAllocator> bArena=new NCollection_IncAllocator(1024);
  Table a(Policy(7),size_t(101),aArena), b(Policy(19),size_t(13),bArena);
  if (!empty) { add(a,4); add(a,1); add(b,7); }
  const auto aFirst=a.FirstNode(), bFirst=b.FirstNode();
  const auto aBuckets=a.Buckets(), bBuckets=b.Buckets();
  reset(rejected); bool succeeded=false;
  try { if (assign) a=std::move(b); else a.Exchange(b); succeeded=true; }
  catch (const std::runtime_error&) {}
  std::cout << prefix << '_' << (assign?"assign":"exchange") << '_' << rejected << '_' << empty << ' '
    << (succeeded?"ok":"throw") << ' ' << a.GetHasher().state << ' ' << b.GetHasher().state << ' '
    << a.Size() << ' ' << b.Size() << ' ' << a.NbBuckets() << ' ' << b.NbBuckets() << ' '
    << (a.FirstNode()==(succeeded?bFirst:aFirst)) << ' ' << (b.FirstNode()==(succeeded?aFirst:bFirst)) << ' '
    << (a.Buckets()==(succeeded?bBuckets:aBuckets)) << ' ' << (b.Buckets()==(succeeded?aBuckets:bBuckets)) << ' '
    << (a.Allocator()==(succeeded?bArena:aArena)) << ' ' << (b.Allocator()==(succeeded?aArena:bArena)) << ' '
    << payloadMoves << ' ' << payloadDrops << ' ' << trace << '\n';
  rejectedPhase=0;
}
template<class Table> void self(const char* prefix)
{
  reset(0); Table map(Policy(7),size_t(101)); add(map,4);
  const auto node=map.FirstNode(); reset(1); map=std::move(map);
  std::cout << prefix << "_self " << map.GetHasher().state << ' ' << map.Size() << ' '
            << (node==map.FirstNode()) << ' ' << phase << ' ' << payloadMoves << ' ' << payloadDrops << '\n';
  rejectedPhase=0;
}
template<class Table> void selfExchange(const char* prefix,int rejected)
{
  reset(0); Table map(Policy(7),size_t(101)); add(map,4);
  const auto node=map.FirstNode(); reset(rejected); bool success=false;
  try { map.Exchange(map); success=true; } catch (const std::runtime_error&) {}
  std::cout << prefix << "_self_exchange_" << rejected << ' ' << (success?"ok":"throw") << ' '
    << map.GetHasher().state << ' ' << map.Size() << ' ' << (node==map.FirstNode()) << ' '
    << phase << ' ' << payloadMoves << ' ' << payloadDrops << ' ' << trace << '\n';
  rejectedPhase=0;
}
template<class Table> void suite(const char* prefix)
{
  for (int rejected=0;rejected!=4;++rejected) { run<Table>(prefix,false,rejected,false); run<Table>(prefix,true,rejected,false); }
  for (int rejected=0;rejected!=4;++rejected) selfExchange<Table>(prefix,rejected);
  run<Table>(prefix,false,0,true); run<Table>(prefix,true,0,true); self<Table>(prefix);
}
int main() { suite<Set>("set"); suite<Map>("data"); }
