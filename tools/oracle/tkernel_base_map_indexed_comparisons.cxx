// Defined signed-distance domains and observable Size evaluation order.
#include <NCollection_BaseMap.hxx>
#include <iostream>
#include <limits>
#include <string>

static std::string calls;
struct ProbeMap {
  char tag;
  size_t extent;
  int value = 42;
  size_t Size() const
  {
    calls += tag;
    return extent;
  }
  const int& operator()(size_t) const
  {
    return value;
  }
  int& operator()(size_t)
  {
    return value;
  }
  const int& FindKey(size_t) const
  {
    return value;
  }
};
using Read = NCollection_BaseMap::IndexedIterator<ProbeMap, int, true>;
using Write = NCollection_BaseMap::IndexedIterator<ProbeMap, int, false>;

static std::string trace()
{
  return calls.empty() ? "-" : calls;
}
template<bool L, bool R>
static void equality(const char* label,
  const NCollection_BaseMap::IndexedIterator<ProbeMap, int, L>& left,
  const NCollection_BaseMap::IndexedIterator<ProbeMap, int, R>& right) {
  calls.clear();
  const bool equal = left == right;
  const std::string equal_calls = trace();
  calls.clear();
  const bool unequal = left != right;
  std::cout << label << ' ' << equal << ' ' << equal_calls << ' '
            << unequal << ' ' << trace() << '\n';
}

static void equality_rows() {
  ProbeMap first{'a', 3}, second{'b', 3};
  Write live(first, 1), end(first, 4), detached;
  Read same(first, 1), other(second, 1), other_end(second, 5), empty;
  equality("live_same", live, same);
  equality("live_foreign", live, other);
  equality("end_live", end, other);
  equality("both_end", end, other_end);
  equality("null_live", detached, other);
  equality("live_null", live, empty);
  equality("both_null", detached, empty);
  equality("same_end", end, Read(first, 4));
}

static void ordering(const char* label, size_t left_index, size_t right_index) {
  ProbeMap first{'a', 0}, second{'b', 0};
  Write left(first, left_index);
  Read right(second, right_index);
  calls.clear();
  const bool less = left < right, greater = left > right;
  const bool less_equal = left <= right, greater_equal = left >= right;
  std::cout << label << ' ' << less << ' ' << greater << ' ' << less_equal
            << ' ' << greater_equal << ' ' << trace() << '\n';
}

int main() {
  equality_rows();
  ordering("less", 1, 2);
  ordering("greater", 2, 1);
  ordering("same_position", 2, 2);
  ordering("signed_max_right", 0, std::numeric_limits<std::ptrdiff_t>::max());
  ordering("signed_max_left", std::numeric_limits<std::ptrdiff_t>::max(), 0);
}
