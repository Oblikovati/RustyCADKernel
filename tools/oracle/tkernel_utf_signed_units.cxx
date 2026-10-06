#include <NCollection_UtfIterator.hxx>
#include <cstdint>
#include <string>
#include <iostream>

void signedInput(const char* label, const signed char* bytes, int count)
{
  NCollection_UtfIterator<signed char> it(bytes);
  std::cout << label;
  for (int i = 0; i < count; ++i, ++it)
    std::cout << ' ' << static_cast<std::uint32_t>(*it) << ':' << it.Index()
              << ':' << it.BufferHere() - bytes << ':' << it.BufferNext() - bytes;
  std::cout << '\n';
}
void signedOutput(char32_t code)
{
  const char32_t input[] = {code, 0};
  NCollection_UtfIterator<char32_t> it(input);
  signed char output[] = {17, 17, 17, 17, 17};
  signed char* end = it.GetUtf(output);
  std::cout << "write " << static_cast<std::uint32_t>(code) << ' '
            << it.AdvanceBytesUtf<signed char>() << ' ' << end - output;
  for (signed char unit : output) std::cout << ' ' << int(unit);
  std::cout << '\n';
}
int main()
{
  const signed char text[] = {65, -61, -87, -16, -97, -104, -128, 0, 0};
  const signed char invalid[] = {-128, -64, -128, -4, -128, -128, -128, -128, -128, 0, 0};
  signedInput("input", text, 4);
  signedInput("invalid", invalid, 4);
  for (int lead = 0; lead <= 255; ++lead)
  {
    const signed char padded[] = {static_cast<signed char>(lead), -128, -128, -128, -128, -128, 0, 0};
    signedInput(("lead" + std::to_string(lead)).c_str(), padded, 1);
  }
  for (char32_t code : {0, 65, 127, 128, 2047, 2048, 0xd800, 0xffff, 0x10000, 0x1f600, 0x10ffff, 0x110000})
    signedOutput(code);
}
