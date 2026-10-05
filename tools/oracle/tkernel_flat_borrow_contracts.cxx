#include <NCollection_FlatMap.hxx>
#include <NCollection_FlatDataMap.hxx>
#include <iostream>
#include <string>
#include <utility>
struct BorrowElement
{
  int number;
  char role;
  std::string* trace;
  BorrowElement(int value, char kind, std::string& events) : number(value), role(kind), trace(&events) {}
  BorrowElement(const BorrowElement& source) : number(source.number), role(source.role), trace(source.trace)
  {
    trace->push_back(role);
  }
  BorrowElement(BorrowElement&& source) noexcept : number(source.number), role(source.role), trace(source.trace) {}
  BorrowElement& operator=(const BorrowElement&) = default;
  BorrowElement& operator=(BorrowElement&&) = default;
};
struct BorrowHash
{
  std::string* trace;
  size_t operator()(const BorrowElement& key) const { trace->push_back('H'); return size_t(key.number); }
  bool operator()(const BorrowElement& left, const BorrowElement& right) const
  {
    trace->push_back('E');
    return left.number == right.number;
  }
};
template <typename Action> static void set_borrow(const char* label, Action action)
{
  std::string trace;
  NCollection_FlatMap<BorrowElement, BorrowHash> map(BorrowHash{&trace});
  map.Add(BorrowElement(1, 'K', trace));
  const BorrowElement key(2, 'K', trace);
  trace.clear();
  action(map, key);
  const std::string miss = trace;
  trace.clear();
  action(map, key);
  std::cout << "set " << label << ' ' << miss << ' ' << trace << '\n';
}
template <typename Action> static void dictionary_borrow(const char* label, Action action)
{
  std::string trace;
  NCollection_FlatDataMap<BorrowElement, BorrowElement, BorrowHash> map(BorrowHash{&trace});
  map.Bind(BorrowElement(1, 'K', trace), BorrowElement(10, 'V', trace));
  BorrowElement key(2, 'K', trace), first(20, 'V', trace), second(21, 'V', trace);
  trace.clear();
  action(map, key, first);
  const std::string miss = trace;
  trace.clear();
  action(map, key, second);
  const std::string hit = trace;
  std::cout << "data " << label << ' ' << miss << ' ' << hit << ' ' << map.Find(key).number << '\n';
}
#define PAIR_CASE(Method) dictionary_borrow(#Method " pair", [](auto& map, auto& key, auto& value) { (void)map.Method(key, value); })
#define KEY_CASE(Method) dictionary_borrow(#Method " key", [](auto& map, auto& key, auto& value) { (void)map.Method(key, std::move(value)); })
#define VALUE_CASE(Method) dictionary_borrow(#Method " value", [](auto& map, auto& key, auto& value) { (void)map.Method(std::move(key), value); })
int main()
{
  set_borrow("Add", [](auto& map, const auto& key) { (void)map.Add(key); });
  set_borrow("Added", [](auto& map, const auto& key) { (void)map.Added(key); });
  PAIR_CASE(Bind); PAIR_CASE(Bound); PAIR_CASE(TryBind); PAIR_CASE(TryBound);
  KEY_CASE(Bind); KEY_CASE(Bound); KEY_CASE(TryBind); KEY_CASE(TryBound);
  VALUE_CASE(Bind); VALUE_CASE(Bound); VALUE_CASE(TryBind); VALUE_CASE(TryBound);
}
