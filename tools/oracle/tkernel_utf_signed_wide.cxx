#include <NCollection_UtfIterator.hxx>
#include <cstdint>
#include <iostream>

template<class Unit> void readUnits(const char* label, const Unit* source, int count)
{
  NCollection_UtfIterator<Unit> cursor(source);
  std::cout << label;
  for (int step=0; step<count; ++step, ++cursor)
    std::cout << ' ' << static_cast<std::uint32_t>(*cursor) << ':' << cursor.IsValid()
              << ':' << cursor.BufferNext()-source << ':' << cursor.AdvanceBytesUtf8();
  std::cout << '\n';
}
template<class Unit> void encodeInput(const char* label, Unit value)
{
  const Unit input[] = {value, 0, 0};
  NCollection_UtfIterator<Unit> cursor(input);
  unsigned char bytes[] = {17,17,17,17};
  char16_t halves[] = {17,17};
  char32_t word[] = {17};
  std::cout << label << ' ' << static_cast<std::int64_t>(value) << ' '
            << static_cast<std::uint32_t>(*cursor) << ' ' << cursor.IsValid() << ' '
            << cursor.GetUtf8(bytes)-bytes << ' ' << cursor.GetUtf16(halves)-halves << ' '
            << cursor.GetUtf32(word)-word;
  for (auto unit : bytes) std::cout << ' ' << unsigned(unit);
  for (auto unit : halves) std::cout << ' ' << unsigned(unit);
  std::cout << ' ' << static_cast<std::uint32_t>(word[0]) << '\n';
}
int main()
{
  const std::int16_t signed16[] = {65, -10179, -8704, -1, 0, 0};
  const std::uint16_t unsigned16[] = {65, 0xd83d, 0xde00, 0xffff, 0, 0};
  const std::int32_t signed32[] = {65, 0x1f600, -1, 0, 0};
  readUnits("i16", signed16, 5);
  readUnits("u16", unsigned16, 4);
  readUnits("i32", signed32, 4);
  for (std::int16_t value : {-32768,-32767,-10179,-8704,-1,0,1,127,128,255,256,2047,2048,32767})
    encodeInput("encode16", value);
  for (std::int32_t value : {-2147483647-1,-1,0,127,128,2047,2048,0xd800,0xffff,0x10000,0x10ffff,0x110000,2147483647})
    encodeInput("encode32", value);
}
