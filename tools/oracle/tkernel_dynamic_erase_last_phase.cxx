// Unmodified OCCT 3d097a0: observe Size from the actual destructor and preserve slot identity.
#include <NCollection_DynamicArray.hxx>
#include <iostream>
#include <string>
struct Item;
static NCollection_DynamicArray<Item>* owner;
static const Item* expected;
static std::string events;
static bool active;
struct Item {
  int value;
  explicit Item(int number) : value(number) {}
  ~Item();
};
Item::~Item() {
  if (active) events += "D" + std::to_string(value) + "@" + std::to_string(owner->Size())
                     + ":" + std::to_string(this == expected) + ";";
}
static void erase() {
  expected = owner->IsEmpty() ? nullptr : &owner->ChangeLast();
  const size_t before = owner->Size();
  owner->EraseLast();
  events += "S" + std::to_string(before) + "," + std::to_string(owner->Size()) + ";";
}
static void run(int count) {
  NCollection_DynamicArray<Item> array(2);
  owner = &array;
  for (int index = 0; index < count; ++index) array.EmplaceAppend(index + 1);
  events.clear(); active = true;
  erase();
  const Item* erased = expected;
  Item& appended = array.EmplaceAppend(77);
  const bool retained = count == 0 || erased == &appended;
  erase();
  while (!array.IsEmpty()) erase();
  erase();
  active = false;
  std::cout << "erase_" << count << ' ' << events << " retained=" << retained << '\n';
}
int main() { for (int count = 0; count <= 5; ++count) run(count); }
