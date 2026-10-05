#include <NCollection_LocalArray.hxx>

#include <iostream>
#include <new>
#include <stdexcept>

class ThrowingDefaultItem
{
public:
  ThrowingDefaultItem() { throw std::runtime_error("default construction failed"); }
  explicit ThrowingDefaultItem(const int theValue) : value(theValue) {}
  ~ThrowingDefaultItem() {}

  int value;
};

class SeededLocalArray : public NCollection_LocalArray<ThrowingDefaultItem, 2>
{
public:
  void seed(const int theValue)
  {
    new (inlinePtr()) ThrowingDefaultItem(theValue);
    mySize = 1;
  }
};

void probe_replacement_failure()
{
  SeededLocalArray anArray;
  anArray.seed(42);
  try
  {
    anArray.Reallocate(3, false);
  }
  catch (const std::runtime_error&)
  {
    std::cout << anArray.Size() << ' ' << static_cast<ThrowingDefaultItem*>(anArray)[0].value
              << '\n';
  }
}

int main()
{
  probe_replacement_failure();
}
