#include <NCollection_LinearVector.hxx>

#include <iostream>
#include <stdexcept>

struct ThrowTag
{
};

struct ThrowingProbe
{
  int value;
  ThrowingProbe() : value(0) { throw std::runtime_error("default construction failed"); }
  explicit ThrowingProbe(int theValue) : value(theValue) {}
  explicit ThrowingProbe(ThrowTag) : value(0) { throw std::runtime_error("emplace failed"); }
};

struct DefaultProbe
{
  int value;
  DefaultProbe() : value(77) {}
  explicit DefaultProbe(int theValue) : value(theValue) {}
};

template <class Item>
void print_state(const char* theLabel, const NCollection_LinearVector<Item>& theVector)
{
  std::cout << theLabel << ' ' << theVector.Capacity() << ' ' << theVector.Size();
  for (size_t anIndex = 0; anIndex < theVector.Size(); ++anIndex)
  {
    std::cout << ' ' << theVector.Value(anIndex).value;
  }
  std::cout << '\n';
}

template <class Item, class Setup, class Append>
void probe(const char* theLabel, const size_t theCapacity, Setup theSetup, Append theAppend)
{
  NCollection_LinearVector<Item> aVector(theCapacity);
  theSetup(aVector);
  try
  {
    theAppend(aVector);
  }
  catch (const std::runtime_error&)
  {
  }
  print_state(theLabel, aVector);
}

void probe_appended_cases()
{
  probe<ThrowingProbe>("appended_empty_throw", 0, [](auto&) {}, [](auto& v) { v.Appended(); });
  probe<ThrowingProbe>("appended_full_throw", 1, [](auto& v) { v.EmplaceAppend(42); },
                       [](auto& v) { v.Appended(); });
  probe<ThrowingProbe>("appended_spare_throw", 2, [](auto& v) { v.EmplaceAppend(42); },
                       [](auto& v) { v.Appended(); });
  probe<DefaultProbe>("appended_empty_success", 0, [](auto&) {}, [](auto& v) { v.Appended(); });
  probe<DefaultProbe>("appended_full_success", 1, [](auto& v) { v.EmplaceAppend(42); },
                      [](auto& v) { v.Appended(); });
  probe<DefaultProbe>("appended_spare_success", 2, [](auto& v) { v.EmplaceAppend(42); },
                      [](auto& v) { v.Appended(); });
}

void probe_emplace_cases()
{
  probe<ThrowingProbe>("emplace_empty_throw", 0, [](auto&) {},
                       [](auto& v) { v.EmplaceAppend(ThrowTag{}); });
  probe<ThrowingProbe>("emplace_full_throw", 1, [](auto& v) { v.EmplaceAppend(42); },
                       [](auto& v) { v.EmplaceAppend(ThrowTag{}); });
  probe<ThrowingProbe>("emplace_spare_throw", 2, [](auto& v) { v.EmplaceAppend(42); },
                       [](auto& v) { v.EmplaceAppend(ThrowTag{}); });
  probe<DefaultProbe>("emplace_empty_success", 0, [](auto&) {},
                      [](auto& v) { v.EmplaceAppend(77); });
  probe<DefaultProbe>("emplace_full_success", 1, [](auto& v) { v.EmplaceAppend(42); },
                      [](auto& v) { v.EmplaceAppend(77); });
  probe<DefaultProbe>("emplace_spare_success", 2, [](auto& v) { v.EmplaceAppend(42); },
                      [](auto& v) { v.EmplaceAppend(77); });
}

int main()
{
  probe_appended_cases();
  probe_emplace_cases();
}
