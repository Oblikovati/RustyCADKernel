#include <NCollection_AccAllocator.hxx>
#include <iostream>

class AccAllocatorKeyProbe : public NCollection_AccAllocator
{
public:
  using NCollection_AccAllocator::Hasher;
  using NCollection_AccAllocator::Key;

  static size_t hash(const size_t value)
  {
    return Hasher{}(Key{value});
  }

  static bool equal(const size_t first, const size_t second)
  {
    return Hasher{}(Key{first}, Key{second});
  }
};

int main()
{
  const size_t values[] = {0, 1, 42, SIZE_MAX};
  for (const size_t value : values)
  {
    std::cout << AccAllocatorKeyProbe::hash(value) << ','
              << AccAllocatorKeyProbe::equal(value, value) << ','
              << AccAllocatorKeyProbe::equal(value, value + 1) << '\n';
  }
}
