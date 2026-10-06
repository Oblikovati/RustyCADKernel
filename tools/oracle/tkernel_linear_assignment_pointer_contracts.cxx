#include <NCollection_LinearVector.hxx>
#include <iostream>
#include <string>
#include <utility>

void linear_pointer_ranges()
{
  NCollection_LinearVector<int> array;
  std::cout << "null:" << (array.Data() == nullptr) << ':' << (array.begin() == nullptr)
            << ':' << (array.end() == nullptr) << '\n';
  array.Reserve(8);
  const int* beginning = array.Data();
  std::cout << "reserved:" << (beginning != nullptr) << ':' << (array.end() == beginning);
  array.Append(11); array.Append(22);
  const auto& constant = array;
  std::cout << ':' << (array.Data() == beginning) << ':' << (array.end() == beginning + 2)
            << ':' << (constant.begin() == beginning) << ':' << (constant.cbegin() == beginning)
            << ':' << (constant.end() == beginning + 2) << ':' << (constant.cend() == beginning + 2) << '\n';
  array.Clear(false);
  std::cout << "retain:" << (array.Data() == beginning) << ':' << (array.end() == beginning) << '\n';
  array.Clear(true);
  std::cout << "release:" << (array.Data() == nullptr) << '\n';
}

void linear_copy_assignment(size_t destination_size, size_t source_size)
{
  NCollection_LinearVector<int> destination(destination_size, 11), source(source_size, 22);
  const auto old_pointer = destination.Data();
  const auto old_capacity = destination.Capacity();
  auto& returned = (destination = source);
  std::cout << "copy:" << (&returned == &destination) << ':' << destination.Size()
            << ':' << (source_size <= old_capacity ? destination.Data() == old_pointer : destination.Capacity() == source_size)
            << ':' << (source_size == 0 || destination.First() == 22) << '\n';
}

void linear_owned_assignment()
{
  NCollection_LinearVector<std::string> destination(2, "old"), source(7, "source");
  auto& copied = (destination = source);
  const auto pointer = source.Data();
  auto& moved = (destination = std::move(source));
  auto& self = (destination = destination);
  std::cout << "owned:" << (&copied == &destination) << ':' << (&moved == &destination)
            << ':' << (destination.Data() == pointer) << ':' << (source.Data() == nullptr)
            << ':' << (&self == &destination) << ':' << (destination.First() == "source") << '\n';
}

void linear_copy_constructor()
{
  NCollection_LinearVector<int> source(8, 22);
  for (int i = 0; i < 5; ++i) source.EraseLast();
  const NCollection_LinearVector<int> copied(source);
  std::cout << "clone:" << copied.Size() << ':' << copied.Capacity()
            << ':' << (copied.First() == 22) << ':' << (copied.Data() != source.Data()) << '\n';
}

int main()
{
  linear_pointer_ranges();
  linear_copy_assignment(8, 3);
  linear_copy_assignment(1, 7);
  linear_copy_assignment(8, 0);
  linear_owned_assignment();
  linear_copy_constructor();
}
