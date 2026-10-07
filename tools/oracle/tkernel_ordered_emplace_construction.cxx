// OCCT 3d097a0: resize, conversion, lookup and retained temporary destruction order.
#include "ordered_emplace_construction/fixture.hxx"
int main(){
 for(int operation=0;operation<4;++operation){
  for(const auto& test:scenarios)runInteger(operation,test);
  for(const auto& test:{scenarios[5],scenarios[6],scenarios[7]})runTracked(operation,test);
  if(operation>=2)runTracked(operation,scenarios[4]);
 }
}
