// Linux/GCC x87 sqrtl oracle for software extended precision.
// Build: c++ -std=c++17 -O2 extended_sqrt_fixture.cxx -o fixture
// Output: input exponent/significand and result exponent/significand in hex.
#include <cmath>
#include <cstdint>
#include <cstring>
#include <iomanip>
#include <iostream>
void emit(std::uint16_t exponent,std::uint64_t significand) {
  unsigned char input[sizeof(long double)] = {};
  std::memcpy(input,&significand,8);std::memcpy(input+8,&exponent,2);
  long double value;std::memcpy(&value,input,sizeof(value));
  const long double result=std::sqrt(value);
  unsigned char output[sizeof(long double)];std::memcpy(output,&result,sizeof(result));
  std::uint64_t outSignificand;std::uint16_t outExponent;
  std::memcpy(&outSignificand,output,8);std::memcpy(&outExponent,output+8,2);
  std::cout<<std::hex<<exponent<<' '<<significand<<' '<<outExponent<<' '<<outSignificand<<'\n';
}
int main() {
  static_assert(sizeof(long double)==16 && __LDBL_MANT_DIG__==64);
  std::uint64_t seed=20160607;
  for(int i=0;i<2048;++i){
    seed=seed*6364136223846793005ULL+1442695040888963407ULL;
    const std::uint16_t exponent=i%8==0?0:std::uint16_t((seed>>48)%32767);
    const std::uint64_t significand=exponent==0?(seed>>unsigned(i%63+1)):(seed|(1ULL<<63));
    emit(exponent,significand);
  }
}
