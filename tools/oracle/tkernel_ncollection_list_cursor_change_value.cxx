#include <NCollection_List.hxx>

#include <iostream>

int main()
{
  NCollection_List<int> aList;
  aList.Append(10);
  aList.Append(20);
  NCollection_List<int>::Iterator anIterator(aList);
  std::cout << anIterator.More() << '/' << anIterator.Value() << '/';
  anIterator.ChangeValue() = 11;
  std::cout << anIterator.More() << '/' << anIterator.Value() << '/';
  anIterator.Next();
  std::cout << anIterator.More() << '/' << anIterator.Value() << '/';
  anIterator.ChangeValue() = 21;
  std::cout << anIterator.Value() << '/';
  anIterator.Next();
  std::cout << anIterator.More() << "/[" << aList.First() << ',' << aList.Last() << "]\n";
  return 0;
}
