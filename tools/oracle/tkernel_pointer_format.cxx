// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871, Linux x86_64 pointer spelling.
#include <Standard_CString.hxx>
#include <cstdint>
#include <iomanip>
#include <iostream>

static void printCase(std::uintptr_t address, const char* format)
{
  char output[128];
  const int count = Sprintf(output, format, reinterpret_cast<void*>(address));
  std::cout << std::hex << address << '|' << format << '|' << std::dec << count << '|';
  for (int i = 0; i < count; ++i)
    std::cout << std::hex << std::setfill('0') << std::setw(2)
              << static_cast<unsigned>(static_cast<unsigned char>(output[i]));
  std::cout << '\n';
}

int main()
{
  const std::uintptr_t addresses[] = {0, 1, 42, 0x1234, 0x7fffffff, 0xffffffffffffffffULL};
  const char* formats[] = {"%p", "%+p", "% p", "%+020p", "%020p", "%.0p", "%.10p",
    "%020.10p", "%#p", "%-20p", "%#.20p", "%020.20p"};
  for (std::uintptr_t address : addresses)
    for (const char* format : formats)
      printCase(address, format);
}
