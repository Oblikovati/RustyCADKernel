#include <NCollection_LinearVector.hxx>

#include <iostream>
#include <stdexcept>

struct CopyControl
{
  int copies = 0;
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
  ~CopyProbe() { ++control->drops; }
};

void print_state(const char* theLabel,
                 const NCollection_LinearVector<CopyProbe>& theVector,
                 const CopyControl& theControl)
{
  std::cout << theLabel << ' ' << theVector.Capacity() << ' ' << theVector.Size() << ' '
            << theControl.copies << ' ' << theControl.moves << ' ' << theControl.drops;
  for (size_t anIndex = 0; anIndex < theVector.Size(); ++anIndex)
  {
    std::cout << ' ' << theVector.Value(anIndex).value;
  }
  std::cout << '\n';
}

void probe_reserved_success()
{
  CopyControl aControl;
  NCollection_LinearVector<CopyProbe> aVector(4);
  aVector.EmplaceAppend(&aControl, 1);
  CopyProbe aFill(&aControl, 9);
  aVector.Resize(3, aFill);
  print_state("reserved_growth", aVector, aControl);
  aVector.Resize(1, aFill);
  print_state("shrink", aVector, aControl);
}

void probe_reallocation_success()
{
  CopyControl aControl;
  NCollection_LinearVector<CopyProbe> aVector(1);
  aVector.EmplaceAppend(&aControl, 1);
  CopyProbe aFill(&aControl, 9);
  aVector.Resize(3, aFill);
  print_state("reallocated_growth", aVector, aControl);
}

void probe_copy_failure()
{
  CopyControl aControl;
  aControl.throwAtCopy = 2;
  {
    NCollection_LinearVector<CopyProbe> aVector(4);
    aVector.EmplaceAppend(&aControl, 1);
    CopyProbe aFill(&aControl, 9);
    try
    {
      aVector.Resize(4, aFill);
    }
    catch (const std::runtime_error&)
    {
    }
    print_state("copy_failure", aVector, aControl);
  }
  std::cout << "copy_failure_after_destruction " << aControl.copies << ' ' << aControl.moves << ' '
            << aControl.drops << '\n';
}

int main()
{
  probe_reserved_success();
  probe_reallocation_success();
  probe_copy_failure();
}
