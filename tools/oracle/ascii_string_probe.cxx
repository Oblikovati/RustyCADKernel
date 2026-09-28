// Exact-output probe for unchanged OCCT byte-string operations.
#include <TCollection_AsciiString.hxx>
#include <Standard_Failure.hxx>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>
#include <string_view>
static std::string decode(const std::string& hex) {
  std::string bytes;
  if (hex == "-") return bytes;
  for (std::size_t i=0;i<hex.size();i+=2)
    bytes.push_back(static_cast<char>(std::stoul(hex.substr(i,2),nullptr,16)));
  return bytes;
}
static void printBytes(const TCollection_AsciiString& value) {
  for(int i=0;i<value.Length();++i)
    std::cout<<std::hex<<std::setw(2)<<std::setfill('0')<<static_cast<unsigned>(static_cast<unsigned char>(value.ToCString()[i]));
  std::cout<<std::dec<<'\n';
}
static void inspect(const TCollection_AsciiString& value) {
  std::cout<<value.Length()<<' '<<value.UsefullLength()<<' '<<value.IsIntegerValue()<<' '<<value.IsRealValue(false)<<' '<<value.IsRealValue(true)<<' ';
  try { int integer=value.IntegerValue(); std::cout<<integer; } catch(const Standard_Failure&) { std::cout<<'x'; }
  std::cout<<' ';
  try { double real=value.RealValue(); std::uint64_t bits; std::memcpy(&bits,&real,8); std::cout<<std::hex<<std::setw(16)<<std::setfill('0')<<bits<<std::dec; } catch(const Standard_Failure&) { std::cout<<'x'; }
  std::cout<<'\n';
}
int main() {
  char mode; std::string encoded;
  while(std::cin>>mode>>encoded) {
    if(mode=='f') {
      const std::uint64_t bits=std::stoull(encoded,nullptr,16); double real; std::memcpy(&real,&bits,8);
      printBytes(TCollection_AsciiString(real)); continue;
    }
    const std::string bytes=decode(encoded);
    TCollection_AsciiString value{std::string_view(bytes)};
    if(mode=='r') { value.Copy(std::string_view(bytes)); }
    if(mode=='h') std::cout<<std::hex<<value.HashCode()<<std::dec<<'\n';
    else inspect(value);
  }
}
