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

template <class Item>
void print_state(const char* theLabel,
                 const NCollection_LinearVector<Item>& theVector,
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

void probe_successful_constructor(const char* theLabel, size_t theSize)
{
  CopyControl aControl;
  {
    CopyProbe aFill(&aControl, 9);
    {
      NCollection_LinearVector<CopyProbe> aVector(theSize, aFill);
      print_state(theLabel, aVector, aControl);
    }
    std::cout << "after_vector_" << theSize << ' ' << aControl.copies << ' '
              << aControl.moves << ' ' << aControl.drops << '\n';
  }
  std::cout << "after_fill_" << theSize << ' ' << aControl.drops << '\n';
}

void probe_throwing_constructor()
{
  CopyControl aControl;
  aControl.throwAtCopy = 2;
  {
    CopyProbe aFill(&aControl, 9);
    try
    {
      NCollection_LinearVector<CopyProbe> aVector(4, aFill);
      print_state("unexpected_success", aVector, aControl);
    }
    catch (const std::runtime_error&)
    {
      std::cout << "constructor_failure " << aControl.copies << ' ' << aControl.moves << ' '
                << aControl.drops << '\n';
    }
  }
  std::cout << "failure_after_fill " << aControl.drops << '\n';
}

int main()
{
  probe_successful_constructor("size_zero", 0);
  probe_successful_constructor("size_one", 1);
  probe_successful_constructor("size_many", 3);
  probe_throwing_constructor();
}
