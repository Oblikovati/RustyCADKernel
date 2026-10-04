#include <NCollection_AccAllocator.hxx>
#include <iostream>

class AccAllocatorKeyProbe : public NCollection_AccAllocator
{
public:
  using NCollection_AccAllocator::Hasher;
  using NCollection_AccAllocator::Key;

  static size_t hash(const size_t value)
  {
    return hashKey(Key{value});
  }

  static bool equal(const size_t first, const size_t second)
  {
    return equalKeys(Key{first}, Key{second});
  }

  static size_t hashKey(const Key key)
  {
    return Hasher{}(key);
  }

  static bool equalKeys(const Key first, const Key second)
  {
    return Hasher{}(first, second);
  }

  static void mutableValue()
  {
    Key key{41};
    std::cout << key.Value << ',' << hashKey(key) << ',';
    key.Value = 42;
    std::cout << key.Value << ',' << hashKey(key) << ',' << equalKeys(key, Key{42}) << ','
              << equalKeys(key, Key{41}) << '\n';
  }

  static void borrowedValue()
  {
    Key key{41};
    size_t& value = key.Value;
    value += 1;
    const size_t* address = &value;
    std::cout << *address << ',' << (address == &key.Value) << ',' << hashKey(key) << ','
              << equalKeys(key, Key{42}) << '\n';
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
  AccAllocatorKeyProbe::mutableValue();
  AccAllocatorKeyProbe::borrowedValue();
}
