// Probe OCCT's unchanged Strtod: record binary64 bits and consumed byte count.
#include <Standard_CString.hxx>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <string>
int main() {
  std::string input;
  while (std::getline(std::cin,input)) {
    char* end=nullptr;
    double value=Strtod(input.c_str(),&end);
    std::uint64_t bits;
    std::memcpy(&bits,&value,sizeof(bits));
    std::cout<<std::hex<<std::setw(16)<<std::setfill('0')<<bits<<std::dec<<' '<<(end-input.c_str())<<'\n';
  }
}
