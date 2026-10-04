#include <NCollection_FlatMap.hxx>

using HiddenIterator = NCollection_FlatMap<int>::iterator_impl<true>;

int main()
{
  return sizeof(HiddenIterator);
}
