#include <NCollection_FlatMap.hxx>
#include <NCollection_FlatDataMap.hxx>
#include <iostream>
#include <string>

struct InsertionHash
{
  std::string* trace;
  size_t operator()(int key) const { trace->push_back('H'); return size_t(key); }
  bool operator()(int left, int right) const { return left == right; }
};
struct ConstructedItem
{
  int number;
  ConstructedItem(int value, std::string* trace) : number(value)
  {
    if (trace != nullptr) trace->push_back('V');
  }
};

template <typename Action> static void set_single_hash(const char* label, Action action)
{
  std::string trace;
  NCollection_FlatMap<int, InsertionHash> map(InsertionHash{&trace});
  map.Add(1);
  trace.clear();
  action(map, 2);
  const std::string miss = trace;
  trace.clear();
  action(map, 2);
  std::cout << "set " << label << ' ' << miss << ' ' << trace << '\n';
}

template <typename Action> static void dictionary_single_hash(const char* label, Action action)
{
  std::string trace;
  NCollection_FlatDataMap<int, int, InsertionHash> map(InsertionHash{&trace});
  map.Bind(1, 10);
  trace.clear();
  action(map, 2, 20);
  const std::string miss = trace;
  trace.clear();
  action(map, 2, 21);
  const std::string hit = trace;
  std::cout << "data " << label << ' ' << miss << ' ' << hit << ' ' << map.Find(2) << '\n';
}

template <typename Action> static void lazy_dictionary_hash(const char* label, Action action)
{
  std::string trace;
  NCollection_FlatDataMap<int, ConstructedItem, InsertionHash> map(InsertionHash{&trace});
  map.Bind(1, ConstructedItem(10, nullptr));
  trace.clear();
  action(map, 2, 20, &trace);
  const std::string miss = trace;
  trace.clear();
  action(map, 2, 21, &trace);
  const std::string hit = trace;
  std::cout << "lazy " << label << ' ' << miss << ' ' << hit << ' ' << map.Find(2).number << '\n';
}

template <typename Action> static void empty_lazy_hash(const char* label, Action action)
{
  std::string trace;
  NCollection_FlatDataMap<int, ConstructedItem, InsertionHash> map(InsertionHash{&trace});
  action(map, 2, 20, &trace);
  std::cout << "empty " << label << ' ' << trace << '\n';
}

static void empty_queries()
{
  std::string trace;
  NCollection_FlatDataMap<int, int, InsertionHash> map(InsertionHash{&trace});
  (void)map.Seek(2);
  (void)map.IsBound(2);
  std::cout << "empty-query " << trace.empty() << '\n';
}

#define SET_CASE(Method) set_single_hash(#Method, [](auto& map, int key) { (void)map.Method(key); })
#define DATA_CASE(Method) dictionary_single_hash(#Method, [](auto& map, int key, int value) { (void)map.Method(key, value); })
#define LAZY_CASE(Method) lazy_dictionary_hash(#Method, [](auto& map, int key, int value, auto* trace) { (void)map.Method(key, value, trace); })
#define EMPTY_CASE(Method) empty_lazy_hash(#Method, [](auto& map, int key, int value, auto* trace) { (void)map.Method(key, value, trace); })

int main()
{
  SET_CASE(Add); SET_CASE(Added); SET_CASE(Emplace); SET_CASE(Emplaced);
  SET_CASE(TryEmplace); SET_CASE(TryEmplaced);
  DATA_CASE(Bind); DATA_CASE(Bound); DATA_CASE(TryBind); DATA_CASE(TryBound);
  LAZY_CASE(Emplace); LAZY_CASE(Emplaced); LAZY_CASE(TryEmplace); LAZY_CASE(TryEmplaced);
  EMPTY_CASE(Emplace); EMPTY_CASE(Emplaced); EMPTY_CASE(TryEmplace); EMPTY_CASE(TryEmplaced);
  empty_queries();
}
