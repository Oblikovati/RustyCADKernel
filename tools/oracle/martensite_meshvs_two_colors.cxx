// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0: actual byte packing, ordered equality, hash and linear output bits.
#include <MeshVS_TwoColors.hxx>
#include <cstdint>
#include <cstring>
#include <iostream>

static void ColorBits(const Quantity_Color& color) {
  for (double channel : {color.Red(), color.Green(), color.Blue()}) {
    float value = float(channel);
    std::uint32_t bits;
    std::memcpy(&bits, &value, sizeof(bits));
    std::cout << " " << bits;
  }
}
static void EmitPair(int mode, int index) {
  const auto space = mode == 0 ? Quantity_TOC_sRGB : Quantity_TOC_RGB;
  const double value = double(index) / 255.0;
  Quantity_Color first(value, double((index * 17) % 256) / 255.0,
                       double((index * 37) % 256) / 255.0, space);
  Quantity_Color second(1.0 - value, value, double((index * 73) % 256) / 255.0, space);
  auto pair = BindTwoColors(first, second);
  std::cout << mode << " " << index << " " << pair.r1 << " " << pair.g1 << " " << pair.b1
    << " " << pair.r2 << " " << pair.g2 << " " << pair.b2 << " " << std::hash<MeshVS_TwoColors>{}(pair);
  ColorBits(ExtractColor(pair, 1)); ColorBits(ExtractColor(pair, 2));
  Quantity_Color a, b; ExtractColors(pair, a, b); ColorBits(a); ColorBits(b);
  std::cout << " " << (pair == BindTwoColors(second, first)) << " " << (pair == pair) << "\n";
}
static void EmitRaw(int index) {
  MeshVS_TwoColors pair{unsigned(index), unsigned(255-index), unsigned((index*17)%256),
    unsigned((index*37)%256), unsigned((index*73)%256), unsigned(index)};
  std::cout << "2 " << index << " " << pair.r1 << " " << pair.g1 << " " << pair.b1
    << " " << pair.r2 << " " << pair.g2 << " " << pair.b2 << " " << std::hash<MeshVS_TwoColors>{}(pair);
  ColorBits(ExtractColor(pair, 1)); ColorBits(ExtractColor(pair, 2));
  Quantity_Color a, b; ExtractColors(pair, a, b); ColorBits(a); ColorBits(b);
  auto reversed = MeshVS_TwoColors{pair.r2,pair.g2,pair.b2,pair.r1,pair.g1,pair.b1};
  std::cout << " " << (pair == reversed) << " " << (pair == pair) << "\n";
}
int main() {
  for (int mode = 0; mode < 2; ++mode) {
    for (int index = 0; index < 256; ++index) EmitPair(mode, index);
  }
  for (int index = 0; index < 256; ++index) EmitRaw(index);
  MeshVS_TwoColors pair{1,2,3,4,5,6};
  for (int index : {-2147483647-1,-1,0,3,2147483647}) {
    std::cout << "invalid " << index; ColorBits(ExtractColor(pair,index)); std::cout << "\n";
  }
}
