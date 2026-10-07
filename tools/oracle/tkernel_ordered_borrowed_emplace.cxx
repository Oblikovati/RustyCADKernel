// OCCT 3d097a0: borrowed Emplace clones the key only after node allocation and value construction.
#include "ordered_borrowed_emplace/fixture.hxx"
int main() {
 for(int method=0;method<4;++method){for(const auto& test:borrowCases)borrowedCase<ScalarMap>("scalar",method,test);}
 for(int method=0;method<4;++method){for(int test:{0,1,2})borrowedCase<TrackedMap>("tracked",method,borrowCases[test]);}
 for(int method:{2,3}){for(int test:{4,5,6,7,8})borrowedCase<TrackedMap>("tracked",method,borrowCases[test]);}
}
