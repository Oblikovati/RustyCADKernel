// Exact input generator for MathLin_EigenSearch_Test.RandomParityAndOrthogonality.
// Pinned source: 3d097a0328e71b826377d4814ab05ec3c3d23871; Linux libstdc++ profile.

#include <random>
#include <iostream>
#include <iomanip>
int main(){std::mt19937 gen(123456u);std::uniform_int_distribution<int> dim(2,32);std::uniform_real_distribution<double> val(-100,100);std::cout<<std::setprecision(17);for(int c=0;c<120;++c){int n=dim(gen);std::cout<<n;for(int i=0;i<n;++i){double d=val(gen),e=i==0?0:val(gen);std::cout<<' '<<d<<' '<<e;}std::cout<<'\n';}}
