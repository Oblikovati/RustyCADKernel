#include <NCollection_IndexedDataMap.hxx>

#include <iostream>
#include <string>

static std::string events;

struct Key
{
  int value;
  explicit Key(int theValue) : value(theValue) {}
  Key(const Key& theOther) : value(theOther.value) { events += 'K'; }
};

struct Item
{
  int value;
  explicit Item(int theValue) : value(theValue) {}
  Item(const Item& theOther) : value(theOther.value) { events += 'V'; }
};

struct Hasher
{
  std::size_t operator()(const Key& theKey) const
  {
    events += 'H';
    return static_cast<std::size_t>(theKey.value);
  }

  bool operator()(const Key& theLeft, const Key& theRight) const
  {
    events += 'E';
    return theLeft.value == theRight.value;
  }
};

int main()
{
  NCollection_IndexedDataMap<Key, Item, Hasher> aMap(Hasher{}, 101);
  Key aKey(7);
  Item anItem(42);
  const int anInserted = aMap.Add(aKey, anItem);
  std::cout << "inserted=" << anInserted << " events=" << events
            << " stored=" << aMap.FindFromIndex(1).value << '\n';
  events.clear();
  Item anIgnored(99);
  const int anExisting = aMap.Add(aKey, anIgnored);
  std::cout << "existing=" << anExisting << " events=" << events
            << " stored=" << aMap.FindFromIndex(1).value << '\n';
}
