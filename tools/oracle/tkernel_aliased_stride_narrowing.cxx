#include <NCollection_AliasedArray.hxx>
#include <iostream>
#include <limits>

class StrideProbe : public NCollection_AliasedArray<> {
public:
  using NCollection_AliasedArray<>::checkedStride;
  using NCollection_AliasedArray<>::checkedSize;
};

int main() {
  const size_t wrap = size_t(std::numeric_limits<uint32_t>::max()) + 1;
  int rejected = 0;
  for (int length : {std::numeric_limits<int>::min(), -1, 0}) {
    try { StrideProbe::checkedSize(length); }
    catch (const Standard_RangeError&) { ++rejected; }
  }
  std::cout << "signed:" << rejected << ':' << StrideProbe::checkedSize(1) << ':'
            << StrideProbe::checkedSize(std::numeric_limits<int>::max()) << '\n';
  try { StrideProbe::checkedStride(0); }
  catch (const Standard_RangeError&) { std::cout << "zero:range\n"; }
  std::cout << "stride:" << StrideProbe::checkedStride(1) << ':'
            << StrideProbe::checkedStride(wrap-1) << ':'
            << StrideProbe::checkedStride(wrap) << ':'
            << StrideProbe::checkedStride(wrap+4) << '\n';
  NCollection_AliasedArray<> empty(wrap), copy(empty);
  std::cout << "empty:" << empty.Stride() << ':' << empty.SizeBytes() << ':'
            << copy.Stride() << ':' << copy.Size() << ':' << copy.IsAllocated() << '\n';
  NCollection_AliasedArray<> array(wrap+4, size_t(2));
  array.Init(uint32_t(17));
  array.Resize(3, true);
  NCollection_AliasedArray<> clone(array);
  std::cout << "allocation:" << array.Stride() << ':' << array.SizeBytes() << ':'
            << array.Value<uint32_t>(1) << ':' << clone.SizeBytes() << '\n';
  NCollection_AliasedArray<> zero(wrap, 2), zeroCopy(zero);
  std::cout << "zero-allocation:" << zero.Stride() << ':' << zero.Size() << ':'
            << zeroCopy.Size() << ':' << zero.IsAllocated() << ':' << zeroCopy.IsAllocated() << '\n';
  NCollection_AliasedArray<> signedArray(wrap+4, 2);
  std::cout << "signed-constructor:" << signedArray.Stride() << ':' << signedArray.Length() << '\n';
}
