#include <NCollection_DataMap.hxx>

#include <iostream>
#include <string>

struct Counters
{
  int copies = 0;
  int keyCopies = 0;
  int copyAssignments = 0;
  int moveAssignments = 0;
  int destructions = 0;
  int hashes = 0;
  std::string events;
};

struct TrackedKey
{
  int value;
  Counters* counters;

  TrackedKey(int theValue, Counters& theCounters)
      : value(theValue), counters(&theCounters)
  {
  }
  TrackedKey(const TrackedKey& theOther)
      : value(theOther.value), counters(theOther.counters)
  {
    ++counters->keyCopies;
    counters->events += 'K';
  }
};

struct KeyHasher
{
  std::size_t operator()(const TrackedKey& theKey) const noexcept
  {
    ++theKey.counters->hashes;
    return static_cast<std::size_t>(theKey.value);
  }
  bool operator()(const TrackedKey& theFirst, const TrackedKey& theSecond) const noexcept
  {
    return theFirst.value == theSecond.value;
  }
};

struct TrackedItem
{
  int value;
  Counters* counters;

  TrackedItem(int theValue, Counters& theCounters)
      : value(theValue), counters(&theCounters)
  {
  }
  TrackedItem(const TrackedItem& theOther)
      : value(theOther.value), counters(theOther.counters)
  {
    ++counters->copies;
    counters->events += 'V';
  }
  TrackedItem(TrackedItem&& theOther) noexcept
      : value(theOther.value), counters(theOther.counters)
  {
  }
  TrackedItem& operator=(const TrackedItem& theOther)
  {
    ++counters->copyAssignments;
    counters->events += 'C';
    value = theOther.value;
    return *this;
  }
  TrackedItem& operator=(TrackedItem&& theOther) noexcept
  {
    ++counters->moveAssignments;
    counters->events += 'M';
    value = theOther.value;
    return *this;
  }
  ~TrackedItem()
  {
    ++counters->destructions;
    counters->events += 'D';
  }
};

static void reset(Counters& theCounters)
{
  theCounters = {};
}

static void printCounts(const Counters& theCounters)
{
  std::cout << " keycopies=" << theCounters.keyCopies
            << " valuecopies=" << theCounters.copies
            << " copyassigns=" << theCounters.copyAssignments
            << " moveassigns=" << theCounters.moveAssignments
            << " destructions=" << theCounters.destructions
            << " hashes=" << theCounters.hashes
            << " events=" << theCounters.events;
}

int main()
{
  Counters counters;
  TrackedItem first(7, counters);
  NCollection_DataMap<TrackedKey, TrackedItem, KeyHasher> map;
  TrackedKey key(4, counters);
  reset(counters);
  const bool inserted = map.Bind(key, first);
  std::cout << "inserted=" << inserted;
  printCounts(counters);
  std::cout << " value=" << map.Find(key).value << '\n';

  TrackedItem replacement(9, counters);
  TrackedItem* const originalSlot = &map.ChangeFind(key);
  reset(counters);
  const bool replaced = map.Bind(key, replacement);
  std::cout << "replaced=" << replaced;
  printCounts(counters);
  std::cout << " stable=" << (&map.ChangeFind(key) == originalSlot)
            << " value=" << map.Find(key).value << '\n';
}
