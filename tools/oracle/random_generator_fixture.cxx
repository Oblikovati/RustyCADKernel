// Native MathUtils_Random stream including reals, seeded independently for each block.
#include <MathUtils_Random.hxx>
#include <cstdint>
#include <cstring>
#include <iostream>
int main(){
 for(std::uint64_t seed:{0ULL,1ULL,6ULL,42ULL,0xffffffffffffffffULL}){
  MathUtils::RandomGenerator rng(seed);
  for(int i=0;i<64;++i){
   const std::uint64_t integer=rng.NextInt();const double real=rng.NextReal();std::uint64_t bits;std::memcpy(&bits,&real,8);
   std::cout<<seed<<' '<<integer<<' '<<bits<<'\n';
  }
 }
}
