// Input generation copied from OSD_Parallel_Test.cxx at the pinned OCCT revision.
// Run the native generator once to retain GCC's exact std::mt19937(42) sequence.

#include <random>
#include <iostream>
int main() {
  std::mt19937 generator(42);
  for (int index=0; index<5000; ++index) {
    std::cout << generator()%1000 << ((index+1)%25==0 ? '\n' : ' ');
  }
}
