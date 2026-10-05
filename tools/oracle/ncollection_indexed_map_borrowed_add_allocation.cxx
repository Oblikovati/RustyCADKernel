#include <NCollection_BaseAllocator.hxx>
#include <NCollection_IndexedMap.hxx>
#include <Standard.hxx>

#include <iostream>
#include <stdexcept>
#include <string>

static std::string events;
static bool throwOnCopy = false;

class TracingAllocator final : public NCollection_BaseAllocator
{
public:
  void* Allocate(const size_t size) override
  {
    events += 'A';
    return Standard::Allocate(size);
  }

  void Free(void* address) override
  {
    events += 'F';
    Standard::Free(address);
  }
};

struct Key
{
  int value;
  explicit Key(int theValue) : value(theValue) {}
  Key(const Key& theOther) : value(theOther.value)
  {
    events += 'K';
    if (throwOnCopy)
      throw std::runtime_error("injected key copy failure");
  }
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
  occ::handle<NCollection_BaseAllocator> allocator = new TracingAllocator;
  NCollection_IndexedMap<Key, Hasher> map(Hasher{}, 101, allocator);
  Key key(7);
  events.clear();
  const int inserted = map.Add(key);
  std::cout << "inserted=" << inserted << " events=" << events
            << " stored=" << map.FindKey(1).value << '\n';
  events.clear();
  const int existing = map.Add(key);
  std::cout << "existing=" << existing << " events=" << events
            << " stored=" << map.FindKey(1).value << '\n';
  map.Clear(false);
  events.clear();
  throwOnCopy = true;
  try
  {
    map.Add(key);
  }
  catch (const std::runtime_error&)
  {
    std::cout << "failed=1 events=" << events << " extent=" << map.Extent() << '\n';
  }
}
