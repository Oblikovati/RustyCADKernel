// Equality must use Size alone even when dereference returns a noncopy owned value.
#include <NCollection_BaseMap.hxx>
#include <iostream>
#include <string>
#include <utility>

static std::string comparison_calls;
struct ComparisonValue {
  int number;
  explicit ComparisonValue(int value) : number(value) {}
  ComparisonValue(const ComparisonValue&) = delete;
  ComparisonValue(ComparisonValue&&) = delete;
};
struct ComparisonOwner {
  char tag;
  size_t extent = 2;
  size_t Size() const
  {
    comparison_calls += tag;
    return extent;
  }
  size_t Size()
  {
    comparison_calls += static_cast<char>(tag - 'a' + 'A');
    return extent;
  }
  ComparisonValue operator()(size_t) const
  {
    comparison_calls += 'v';
    return ComparisonValue(42);
  }
  int FindKey(size_t) const
  {
    comparison_calls += 'k';
    return 7;
  }
};
template<bool Constant>
using ComparisonCursor = NCollection_BaseMap::IndexedIterator<ComparisonOwner, int, Constant>;

static std::string take_comparison_calls()
{
  std::string previous = std::exchange(comparison_calls, {});
  return previous.empty() ? "-" : previous;
}
template<bool Left, bool Right>
static void comparison_row(const char* name, const ComparisonCursor<Left>& left,
  const ComparisonCursor<Right>& right)
{
  const bool equal = left == right;
  const std::string equal_calls = take_comparison_calls();
  const bool unequal = left != right;
  std::cout << name << ' ' << equal << ' ' << equal_calls << ' '
            << unequal << ' ' << take_comparison_calls() << '\n';
}
static void attached_rows()
{
  ComparisonOwner first{'a'}, second{'b'};
  comparison_row("live_same", ComparisonCursor<false>(first), ComparisonCursor<true>(first));
  comparison_row("live_other", ComparisonCursor<false>(first), ComparisonCursor<true>(second));
  comparison_row("end_live", ComparisonCursor<false>(first, 3), ComparisonCursor<true>(second));
  comparison_row("both_end", ComparisonCursor<false>(first, 3), ComparisonCursor<true>(second, 4));
  comparison_row("same_end", ComparisonCursor<false>(first, 3), ComparisonCursor<true>(first, 3));
  comparison_row("zero_live", ComparisonCursor<true>(first, 0), ComparisonCursor<false>(second, 0));
  comparison_row("readonly_first", ComparisonCursor<true>(first, 3), ComparisonCursor<false>(second));
  first.extent = 0;
  second.extent = 0;
  comparison_row("size_changed", ComparisonCursor<true>(first), ComparisonCursor<false>(second));
}
static void detached_rows()
{
  ComparisonOwner first{'a'}, second{'b'};
  comparison_row("null_live", ComparisonCursor<false>(), ComparisonCursor<true>(second));
  comparison_row("live_null", ComparisonCursor<false>(first), ComparisonCursor<true>());
  comparison_row("both_null", ComparisonCursor<false>(), ComparisonCursor<true>());
  comparison_row("null_end", ComparisonCursor<false>(), ComparisonCursor<true>(second, 3));
}
int main()
{
  attached_rows();
  detached_rows();
}
