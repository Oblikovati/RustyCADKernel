// Exhaustive UTF-16 code-unit conversion and hash probe against unchanged OCCT.
#include <TCollection_ExtendedString.hxx>
#include <TCollection_AsciiString.hxx>
#include <iomanip>
#include <iostream>
#include <string>
#include <vector>
static void printBytes(const char* bytes, int length) {
  for(int i=0;i<length;++i) std::cout<<std::hex<<std::setw(2)<<std::setfill('0')<<static_cast<unsigned>(static_cast<unsigned char>(bytes[i]));
  std::cout<<std::dec;
}
int main() {
  std::string encoded;
  while(std::cin>>encoded) {
    std::vector<char16_t> units;
    if(encoded!="-") for(std::size_t i=0;i<encoded.size();i+=4) units.push_back(static_cast<char16_t>(std::stoul(encoded.substr(i,4),nullptr,16)));
    const int length=static_cast<int>(units.size()); units.push_back(0);
    TCollection_ExtendedString value(units.data(),length);
    const int size=value.LengthOfCString(); std::vector<char> bytes(size+1);
    char* destination=bytes.data(); const int written=value.ToUTF8CString(destination);
    std::cout<<value.Length()<<' '<<value.IsAscii()<<' '<<std::hex<<value.HashCode()<<std::dec<<' '<<size<<' '<<written<<' ';
    printBytes(bytes.data(),written); std::cout<<' ';
    const TCollection_AsciiString ascii(value); printBytes(ascii.ToCString(),ascii.Length()); std::cout<<' ';
    const TCollection_AsciiString replaced(value,'?'); printBytes(replaced.ToCString(),replaced.Length()); std::cout<<'\n';
  }
}
