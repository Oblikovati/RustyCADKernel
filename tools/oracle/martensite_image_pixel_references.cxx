// Native live pixel aliases distinguish reference behavior from copied values.
#include <Image_PixMap.hxx>
#include <Image_PixMapTypedData.hxx>
#include <NCollection_BaseAllocator.hxx>
#include <iostream>

int main()
{
  Image_PixMap image;
  if (!image.InitZero(Image_Format_Gray, 3, 2)) return 1;
  const auto& read = image.Value<unsigned char>(1, 2);
  auto& write = image.ChangeValueXY<unsigned char>(2, 1);
  write = 37;
  std::cout << "pixmap " << int(read) << ' ' << (&read == &write) << '\n';
  *image.ChangeRawValue(1, 2) = 81;
  std::cout << "pixmap_mutated " << int(read) << ' ' << int(write) << '\n';
  Image_PixMapTypedData<unsigned char> typed;
  if (!typed.Init(NCollection_BaseAllocator::CommonBaseAllocator(), 3, 2)) return 2;
  const auto& typedRead = typed.Value(1, 2);
  auto& typedWrite = typed.ChangeValue(1, 2);
  typedWrite = 52;
  std::cout << "typed " << int(typedRead) << ' ' << (&typedRead == &typedWrite) << '\n';
  typed.ChangeRow(1)[2] = 93;
  std::cout << "typed_mutated " << int(typedRead) << ' ' << int(typedWrite) << '\n';
}
