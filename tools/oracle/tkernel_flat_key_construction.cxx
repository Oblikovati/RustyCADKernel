#include <NCollection_FlatMap.hxx>
#include <iostream>
#include <string>
#include <utility>

struct ConstructorFailure {};
struct ConstructedKey
{
  int number;
  int serial;
  ConstructedKey(int value, int identity, std::string& trace, bool fail = false)
      : number(value), serial(identity)
  {
    trace.push_back('C');
    if (fail) throw ConstructorFailure{};
  }
};
struct ConstructionPolicy
{
  std::string* trace;
  size_t operator()(const ConstructedKey& key) const
  {
    trace->push_back('H');
    return size_t(key.number);
  }
  bool operator()(const ConstructedKey& left, const ConstructedKey& right) const
  {
    trace->push_back('E');
    return left.number == right.number;
  }
};
using ConstructionSet = NCollection_FlatMap<ConstructedKey, ConstructionPolicy>;
static int outcome(bool result) { return result; }
static int outcome(const ConstructedKey& result) { return result.serial; }

template <typename Invoke> static void construction_contract(const char* name, Invoke invoke)
{
  std::string trace;
  ConstructionSet map(ConstructionPolicy{&trace});
  int miss = outcome(invoke(map, trace, 1, false));
  std::string missEvents = std::exchange(trace, "");
  int hit = outcome(invoke(map, trace, 2, false));
  std::cout << name << " regular " << miss << ' ' << missEvents << ' ' << hit << ' '
            << trace << ' ' << map.Size() << ' ' << map.Capacity() << ' '
            << map.begin()->serial << '\n';
}
template <typename Invoke> static void failing_contract(const char* name, Invoke invoke)
{
  std::string trace;
  ConstructionSet map(ConstructionPolicy{&trace});
  bool failed = false;
  try { invoke(map, trace, -1, true); }
  catch (const ConstructorFailure&) { failed = true; }
  std::cout << name << " failure " << failed << ' ' << trace << ' '
            << map.Size() << ' ' << map.Capacity() << '\n';
}
#define CONSTRUCTION_CASE(Name) \
  construction_contract(#Name, [](auto& map, auto& trace, int serial, bool fail) -> decltype(auto) \
    { return map.Name(4, serial, trace, fail); }); \
  failing_contract(#Name, [](auto& map, auto& trace, int serial, bool fail) -> decltype(auto) \
    { return map.Name(4, serial, trace, fail); })

int main()
{
  CONSTRUCTION_CASE(Emplace);
  CONSTRUCTION_CASE(Emplaced);
  CONSTRUCTION_CASE(TryEmplace);
  CONSTRUCTION_CASE(TryEmplaced);
  NCollection_FlatMap<int> map;
  map.Add(0); map.Add(1);
  std::cout << "Emplaced displacement " << map.Emplaced(8) << ' ' << map.Contains(8) << ' ' << map.Size() << '\n';
  NCollection_FlatMap<int> tried;
  tried.Add(0); tried.Add(1);
  std::cout << "TryEmplaced displacement " << tried.TryEmplaced(8) << ' ' << tried.Contains(8) << ' ' << tried.Size() << '\n';
}
