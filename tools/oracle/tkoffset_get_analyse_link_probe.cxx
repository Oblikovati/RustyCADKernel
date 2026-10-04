#include <BRepOffset_MakeOffset.hxx>

int main() {
  BRepOffset_MakeOffset maker;
  return maker.GetAnalyse().IsDone() ? 1 : 0;
}
