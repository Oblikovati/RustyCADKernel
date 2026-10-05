#include <NCollection_LinearVector.hxx>

#include <iostream>

struct RelocationCounts
{
  int copies = 0;
  int moves = 0;
};

struct MoveIfNoexceptProbe
{
  RelocationCounts* counts;
  int value;

  MoveIfNoexceptProbe(RelocationCounts& theCounts, int theValue)
      : counts(&theCounts), value(theValue)
  {
  }
  MoveIfNoexceptProbe(const MoveIfNoexceptProbe& theOther)
      : counts(theOther.counts), value(theOther.value)
  {
    ++counts->copies;
  }
  MoveIfNoexceptProbe(MoveIfNoexceptProbe&& theOther) noexcept(false)
      : counts(theOther.counts), value(theOther.value)
  {
    ++counts->moves;
    theOther.value = -1;
  }
};

int main()
{
  RelocationCounts aCounts;
  NCollection_LinearVector<MoveIfNoexceptProbe> aVector(1);
  aVector.EmplaceAppend(aCounts, 42);
  aVector.Reserve(2);
  std::cout << aCounts.copies << ' ' << aCounts.moves << ' ' << aVector.Size() << ' '
            << aVector.Value(0).value << '\n';
}
