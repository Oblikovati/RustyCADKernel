#include <NCollection_LinearVector.hxx>

#include <iostream>
#include <stdexcept>

struct CopyControl
{
  int copies = 0;
  int assignments = 0;
  int moves = 0;
  int drops = 0;
  int throwAtCopy = 0;
};

struct CopyProbe
{
  CopyControl* control;
  int value;

  CopyProbe(CopyControl* theControl, int theValue) : control(theControl), value(theValue) {}
  CopyProbe(const CopyProbe& theOther) : control(theOther.control), value(theOther.value)
  {
    ++control->copies;
    if (control->copies == control->throwAtCopy)
    {
      throw std::runtime_error("copy failed");
    }
  }
  CopyProbe(CopyProbe&& theOther) noexcept : control(theOther.control), value(theOther.value)
  {
    ++control->moves;
  }
  CopyProbe& operator=(const CopyProbe& theOther)
  {
    ++control->assignments;
    value = theOther.value;
    return *this;
  }
  ~CopyProbe() { ++control->drops; }
};

void print_state(const NCollection_LinearVector<CopyProbe>& theVector,
                 const CopyControl& theControl)
{
  std::cout << "assignment_failure " << theVector.Capacity() << ' ' << theVector.Size() << ' '
            << theControl.copies << ' ' << theControl.assignments << ' ' << theControl.moves << ' '
            << theControl.drops;
  for (size_t anIndex = 0; anIndex < theVector.Size(); ++anIndex)
  {
    std::cout << ' ' << theVector.Value(anIndex).value;
  }
  std::cout << '\n';
}

int main()
{
  CopyControl aControl;
  aControl.throwAtCopy = 2;
  {
    NCollection_LinearVector<CopyProbe> aSource(3);
    aSource.EmplaceAppend(&aControl, 10);
    aSource.EmplaceAppend(&aControl, 20);
    aSource.EmplaceAppend(&aControl, 30);
    NCollection_LinearVector<CopyProbe> aDestination(4);
    aDestination.EmplaceAppend(&aControl, 1);
    try
    {
      aDestination = aSource;
    }
    catch (const std::runtime_error&)
    {
    }
    print_state(aDestination, aControl);
  }
  std::cout << "after_destruction " << aControl.copies << ' ' << aControl.assignments << ' '
            << aControl.moves << ' ' << aControl.drops << '\n';
}
