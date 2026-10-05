#include <NCollection_FlatDataMap.hxx>
#include <NCollection_FlatMap.hxx>
#include <iostream>
#include <string>
#include <utility>

struct MoveElement
{
  int number;
  char role;
  std::string* trace;
  MoveElement(int value, char kind, std::string& events)
      : number(value), role(kind), trace(&events) {}
  MoveElement(const MoveElement&) = delete;
  MoveElement(MoveElement&& source) noexcept
      : number(source.number), role(source.role), trace(source.trace)
  {
    source.trace = nullptr;
  }
  MoveElement& operator=(MoveElement&& source) noexcept
  {
    number = source.number;
    role = source.role;
    trace = source.trace;
    source.trace = nullptr;
    return *this;
  }
  ~MoveElement() { if (trace) trace->push_back(role); }
};

struct MovePolicy
{
  int salt;
  std::string* trace;
  MovePolicy(int value, std::string& events) : salt(value), trace(&events) {}
  MovePolicy(const MovePolicy& source) : salt(source.salt), trace(source.trace)
  {
    trace->push_back('C');
  }
  MovePolicy(MovePolicy&& source) noexcept : salt(source.salt), trace(source.trace)
  {
    trace->push_back('M');
    source.salt = -1;
  }
  MovePolicy& operator=(MovePolicy&& source) noexcept
  {
    source.trace->push_back('A');
    salt = source.salt;
    trace = source.trace;
    source.salt = -1;
    return *this;
  }
  ~MovePolicy() { trace->push_back('P'); }
  size_t operator()(const MoveElement& key) const
  {
    trace->push_back('H');
    return size_t(key.number + salt);
  }
  bool operator()(const MoveElement& left, const MoveElement& right) const
  {
    return left.number == right.number;
  }
};
using MoveSet = NCollection_FlatMap<MoveElement, MovePolicy>;
using MoveDictionary = NCollection_FlatDataMap<MoveElement, MoveElement, MovePolicy>;

static MoveSet build_set(std::string& trace, int number, int salt)
{
  MoveSet map(MovePolicy(salt, trace), 40);
  map.Add(MoveElement(number, 'k', trace));
  return map;
}
static MoveDictionary build_dictionary(std::string& trace, int number, int salt)
{
  MoveDictionary map(MovePolicy(salt, trace), 40);
  map.Bind(MoveElement(number, 'k', trace), MoveElement(number * 10, 'v', trace));
  return map;
}

template <typename Map> static void borrowed_policy(const char* label)
{
  std::string trace;
  const MovePolicy policy(7, trace);
  for (size_t hint : {0, 1, 13, 14, 40})
  {
    trace.clear();
    const Map map(policy, hint);
    std::cout << label << " copy " << hint << ' ' << trace << ' '
              << map.Size() << ' ' << map.Capacity() << ' '
              << map.GetHasher().salt << ' ' << policy.salt << '\n';
  }
}

template <typename Map> static void moved_policy(const char* label)
{
  std::string trace;
  for (size_t hint : {0, 1, 13, 14, 40})
  {
    MovePolicy policy(7, trace);
    trace.clear();
    const Map map(std::move(policy), hint);
    std::cout << label << " policy-move " << hint << ' ' << trace << ' '
              << map.Size() << ' ' << map.Capacity() << ' '
              << map.GetHasher().salt << ' ' << policy.salt << '\n';
  }
}

template <typename Build> static void custom_construction(const char* label, Build build, bool empty)
{
  std::string trace;
  auto source = build(trace, 4, 7);
  if (empty) source.Clear(false);
  trace.clear();
  decltype(source) moved(std::move(source));
  std::cout << label << " construct " << empty << ' ' << trace << ' '
            << source.Size() << ' ' << source.Capacity() << ' ' << source.GetHasher().salt << ' '
            << moved.Size() << ' ' << moved.Capacity() << ' ' << moved.GetHasher().salt << '\n';
}

template <typename Build> static void custom_assignment(const char* label, Build build, bool empty)
{
  std::string trace;
  auto source = build(trace, 4, 7), target = build(trace, 9, 2);
  if (empty) source.Clear(false);
  trace.clear();
  bool identity = &(target = std::move(source)) == &target;
  std::cout << label << " assign " << empty << ' ' << trace << ' ' << identity << ' '
            << source.Size() << ' ' << source.Capacity() << ' ' << source.GetHasher().salt << ' '
            << target.Size() << ' ' << target.Capacity() << ' ' << target.GetHasher().salt << '\n';
}

template <typename Build> static void self_assignment(const char* label, Build build)
{
  std::string trace;
  auto map = build(trace, 4, 7);
  trace.clear();
  bool identity = &(map = std::move(map)) == &map;
  std::cout << label << " self " << trace.empty() << ' ' << identity << ' '
            << map.Size() << ' ' << map.Capacity() << ' ' << map.GetHasher().salt << '\n';
}

static void default_set_assignment()
{
  NCollection_FlatMap<int> source(40), target(1);
  source.Add(4); target.Add(9);
  const int* address = &source.begin().Key();
  bool identity = &(target = std::move(source)) == &target;
  std::cout << "set default " << identity << ' ' << source.Size() << ' ' << source.Capacity() << ' '
            << target.Size() << ' ' << target.Capacity() << ' ' << (&target.begin().Key() == address) << '\n';
}
static void default_dictionary_assignment()
{
  NCollection_FlatDataMap<int, int> source(40), target(1);
  source.Bind(4, 40); target.Bind(9, 90);
  const int* address = &*source.begin();
  bool identity = &(target = std::move(source)) == &target;
  std::cout << "data default " << identity << ' ' << source.Size() << ' ' << source.Capacity() << ' '
            << target.Size() << ' ' << target.Capacity() << ' ' << (&*target.begin() == address) << '\n';
}

int main()
{
  borrowed_policy<MoveSet>("set");
  borrowed_policy<MoveDictionary>("data");
  moved_policy<MoveSet>("set");
  moved_policy<MoveDictionary>("data");
  for (bool empty : {false, true})
  {
    custom_construction("set", build_set, empty);
    custom_construction("data", build_dictionary, empty);
    custom_assignment("set", build_set, empty);
    custom_assignment("data", build_dictionary, empty);
  }
  self_assignment("set", build_set);
  self_assignment("data", build_dictionary);
  default_set_assignment();
  default_dictionary_assignment();
}
