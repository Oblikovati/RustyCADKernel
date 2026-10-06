// Source oracle for shared const cursors and unconditional prefix delegation.
#include <NCollection_ForwardRange.hxx>
#include <iostream>
#include <string>

struct CountingHost {
  int position = 0;
  bool More() const { return position < 3; }
  void Next() { ++position; }
  int Value() const { return position; }
};

static void sharedCursor() {
  const NCollection_ForwardRange<CountingHost> range(CountingHost{});
  auto first = range.begin();
  auto second = range.cbegin();
  const int initial = *first++;
  const int observed = *second;
  const int previous = *first++;
  const int current = *second;
  ++first;
  std::cout << "shared " << initial << ' ' << observed << ' ' << previous << ' '
            << current << ' ' << (first == range.end()) << ' '
            << (range.cend() == second) << '\n';
}

static void prefixDelegation() {
  CountingHost host{2};
  NCollection_ForwardRangeIterator<CountingHost> iter(&host);
  const bool returned = &++iter == &iter;
  ++iter;
  std::cout << "prefix " << returned << ' ' << host.position << '\n';
}

static void copiedMovedRange() {
  CountingHost host;
  NCollection_ForwardRange<CountingHost> range(host);
  const int first = *range.cbegin()++;
  auto moved = std::move(range);
  const int second = *moved.begin()++;
  const int third = *moved.begin()++;
  std::cout << "move " << first << ' ' << second << ' ' << third << ' '
            << (moved.cbegin() == moved.cend()) << ' ' << host.position << '\n';
}

struct BorrowedHost {
  const std::string* values;
  int position = 0;
  bool More() const { return position < 2; }
  void Next() { ++position; }
  const std::string& Value() const { return values[position]; }
};

static void referenceSnapshot() {
  const std::string values[] = {"first", "second"};
  const NCollection_ForwardRange<BorrowedHost> range(BorrowedHost{values});
  auto first = range.begin();
  const std::string& old = *first;
  const auto snapshot = first++;
  std::cout << "reference " << old << ' ' << *snapshot << ' ' << *range.cbegin()
            << ' ' << (&old != &*snapshot) << '\n';
}

int main() {
  sharedCursor();
  prefixDelegation();
  copiedMovedRange();
  referenceSnapshot();
}
