// Preserve the native value-proxy versus borrowed-pointer operator-> branches.
#include <NCollection_ForwardRange.hxx>
#include <iostream>
#include <string>
struct Item { int value; };
struct ValueHost {
  std::string* trace;
  bool More() const { *trace += "M"; return false; }
  void Next() { *trace += "N"; }
  Item Value() const { *trace += "V"; return Item{7}; }
};
struct ReferenceHost {
  const Item* item;
  std::string* trace;
  bool More() const { *trace += "M"; return false; }
  void Next() { *trace += "N"; }
  const Item& Value() const { *trace += "R"; return *item; }
};
static void valueArrow() {
  std::string trace;
  ValueHost host{&trace};
  NCollection_ForwardRangeIterator<ValueHost> iter(&host);
  auto proxy = iter.operator->();
  std::cout << "value_arrow " << proxy->value << ' '
            << (proxy.operator->() == &proxy.myValue) << ' ' << trace << '\n';
}
static void referenceArrow() {
  std::string trace;
  Item item{42};
  ReferenceHost host{&item, &trace};
  NCollection_ForwardRangeIterator<ReferenceHost> iter(&host);
  const Item* pointer = iter.operator->();
  std::cout << "reference_arrow " << pointer->value << ' '
            << (pointer == &item) << ' ' << trace << '\n';
}
int main() { valueArrow(); referenceArrow(); }
