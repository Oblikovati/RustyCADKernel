#include <NCollection_IndexedMap.hxx>

#include <iostream>
#include <string>

static std::string events;

struct Key
{
  int value;
  explicit Key(int theValue) : value(theValue) {}
  Key(const Key& theOther) : value(theOther.value) { events += 'K'; }
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
  NCollection_IndexedMap<Key, Hasher> aMap(Hasher{}, 101);
  Key aKey(7);
  const int anInserted = aMap.Add(aKey);
  std::cout << "inserted=" << anInserted << " events=" << events
            << " stored=" << aMap.FindKey(1).value << '\n';
  events.clear();
  const int anExisting = aMap.Add(aKey);
  std::cout << "existing=" << anExisting << " events=" << events
            << " stored=" << aMap.FindKey(1).value << '\n';
}
