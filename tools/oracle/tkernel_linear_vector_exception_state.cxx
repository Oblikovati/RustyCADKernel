#include <NCollection_LinearVector.hxx>

#include <iostream>
#include <stdexcept>

struct CopyControl
{
  int attempts = 0;
  int assignments = 0;
  int throwAt = 0;
};

struct CopyProbe
{
  CopyControl* control;
  int value;

  CopyProbe(CopyControl& theControl, int theValue) : control(&theControl), value(theValue) {}
  CopyProbe(const CopyProbe& theOther) : control(theOther.control), value(theOther.value)
  {
    if (++control->attempts == control->throwAt)
    {
      throw std::runtime_error("injected copy failure");
    }
  }
  CopyProbe(CopyProbe&& theOther) noexcept : control(theOther.control), value(theOther.value)
  {
    theOther.control = nullptr;
  }
  CopyProbe& operator=(const CopyProbe& theOther)
  {
    control = theOther.control;
    value = theOther.value;
    ++control->assignments;
    return *this;
  }
  ~CopyProbe() {}
};

NCollection_LinearVector<CopyProbe> make_source(CopyControl& theControl)
{
  NCollection_LinearVector<CopyProbe> aSource;
  aSource.EmplaceAppend(theControl, 10);
  aSource.EmplaceAppend(theControl, 20);
  aSource.EmplaceAppend(theControl, 30);
  return aSource;
}

void probe_resize_exception()
{
  CopyControl aControl;
  NCollection_LinearVector<CopyProbe> aVector;
  aVector.EmplaceAppend(aControl, 42);
  CopyProbe aFill(aControl, 7);
  aControl.throwAt = 2;
  try
  {
    aVector.Resize(4, aFill);
  }
  catch (const std::runtime_error&)
  {
    std::cout << aVector.Size() << ' ' << aVector.Value(0).value << ' ' << aControl.attempts
              << '\n';
  }
}

void probe_assignment_exception()
{
  CopyControl aControl;
  NCollection_LinearVector<CopyProbe> aSource = make_source(aControl);
  NCollection_LinearVector<CopyProbe> aDestination(5);
  aDestination.EmplaceAppend(aControl, 1);
  aControl.attempts = 0;
  aControl.assignments = 0;
  aControl.throwAt = 2;
  try
  {
    aDestination = aSource;
  }
  catch (const std::runtime_error&)
  {
    std::cout << aDestination.Size() << ' ' << aDestination.Value(0).value << ' '
              << aControl.attempts << ' ' << aControl.assignments << '\n';
  }
}

int main()
{
  probe_resize_exception();
  probe_assignment_exception();
}
