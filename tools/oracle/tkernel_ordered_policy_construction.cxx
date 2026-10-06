#include <NCollection_OrderedMap.hxx>
#include <NCollection_OrderedDataMap.hxx>
#include <Standard_OutOfRange.hxx>
#include <iostream>
#include <stdexcept>
#include <string>

static std::string trace;
static bool failCopy = false;
static bool failMove = false;
struct Policy
{
  int state;
  explicit Policy(int value) : state(value) {}
  Policy(const Policy& source) : state(source.state)
  {
    trace += "C";
    if (failCopy) { trace += "X"; throw std::runtime_error("copy"); }
  }
  Policy(Policy&& source) : state(source.state)
  {
    trace += "M";
    source.state = -99;
    if (failMove) { trace += "X"; throw std::runtime_error("move"); }
  }
  size_t operator()(int key) const { return size_t(key % 3); }
  bool operator()(int a, int b) const { return a == b; }
};
struct MoveOnly
{
  int state;
  explicit MoveOnly(int value) : state(value) {}
  MoveOnly(const MoveOnly&) = delete;
  MoveOnly(MoveOnly&& source) : state(source.state) { trace += "M"; source.state = -99; }
  size_t operator()(int key) const { return size_t(key % 3); }
  bool operator()(int a, int b) const { return a == b; }
};
template<class H> void fill(NCollection_OrderedMap<int,H>& map) { map.Add(4); map.Add(1); }
template<class H> void fill(NCollection_OrderedDataMap<int,int,H>& map) { map.Bind(4,40); map.Bind(1,10); }
void start() { trace.clear(); failCopy = false; failMove = false; }
std::string events() { return trace.empty() ? "-" : trace; }
template<class Map, class Count> void borrowed(const char* prefix, const char* label, Count count, bool throwing)
{
  start(); Policy policy(7); failCopy = throwing;
  std::cout << prefix << '_' << label << ' ';
  try { Map map(policy,count); std::cout << "ok " << map.GetHasher().state << ' ' << map.NbBuckets(); }
  catch (const Standard_OutOfRange&) { std::cout << "range"; }
  catch (const std::runtime_error&) { std::cout << "throw"; }
  std::cout << ' ' << policy.state << ' ' << events() << '\n';
}
template<class Map, class Count> void moved(const char* prefix, const char* label, Count count, bool throwing)
{
  start(); Policy policy(7); failMove = throwing;
  std::cout << prefix << '_' << label << ' ';
  try { Map map(std::move(policy),count); std::cout << "ok " << map.GetHasher().state << ' ' << map.NbBuckets(); }
  catch (const Standard_OutOfRange&) { std::cout << "range"; }
  catch (const std::runtime_error&) { std::cout << "throw"; }
  std::cout << ' ' << policy.state << ' ' << events() << '\n';
}
template<class Map> void transfer(const char* prefix, bool populated, bool throwing)
{
  start(); Policy policy(7); Map source(policy,size_t(101));
  if (populated) fill(source);
  const auto first = source.FirstNode(); const auto buckets = source.Buckets();
  start(); failMove = throwing;
  std::cout << prefix << "_transfer_" << populated << '_' << throwing << ' ';
  try {
    Map target(std::move(source));
    std::cout << "ok " << target.GetHasher().state << ' ' << target.Size() << ' '
              << (target.FirstNode()==first) << ' ' << (target.Buckets()==buckets) << ' '
              << (target.Allocator()==source.Allocator()) << ' ';
  } catch (const std::runtime_error&) {
    std::cout << "throw " << (source.FirstNode()==first) << ' ' << (source.Buckets()==buckets) << ' ';
  }
  std::cout << source.GetHasher().state << ' ' << source.Size() << ' ' << source.NbBuckets() << ' '
            << (source.Buckets()==nullptr) << ' ' << events() << '\n';
}
template<class Map> void suite(const char* prefix)
{
  borrowed<Map>(prefix,"borrow_size",size_t(13),false);
  borrowed<Map>(prefix,"borrow_signed",0,false);
  borrowed<Map>(prefix,"borrow_negative",-7,false);
  borrowed<Map>(prefix,"borrow_throw",size_t(13),true);
  borrowed<Map>(prefix,"borrow_throw_signed",13,true);
  borrowed<Map>(prefix,"borrow_negative_throw",-7,true);
  moved<Map>(prefix,"move_size",size_t(13),false);
  moved<Map>(prefix,"move_signed",0,false);
  moved<Map>(prefix,"move_negative",-7,false);
  moved<Map>(prefix,"move_throw",size_t(13),true);
  moved<Map>(prefix,"move_throw_signed",13,true);
  moved<Map>(prefix,"move_negative_throw",-7,true);
  transfer<Map>(prefix,false,false);
  transfer<Map>(prefix,true,false);
  transfer<Map>(prefix,true,true);
}
template<class Map> void moveOnly(const char* prefix)
{
  start(); MoveOnly policy(19); Map source(std::move(policy),size_t(101)); fill(source);
  const auto first=source.FirstNode(); start(); Map target(std::move(source));
  std::cout << prefix << "_move_only " << target.GetHasher().state << ' ' << source.GetHasher().state << ' '
            << target.Size() << ' ' << source.Size() << ' ' << (target.FirstNode()==first) << ' ' << events() << '\n';
}
int main()
{
  suite<NCollection_OrderedMap<int,Policy>>("set");
  suite<NCollection_OrderedDataMap<int,int,Policy>>("data");
  moveOnly<NCollection_OrderedMap<int,MoveOnly>>("set");
  moveOnly<NCollection_OrderedDataMap<int,int,MoveOnly>>("data");
}
