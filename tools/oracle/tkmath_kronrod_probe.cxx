// Generated quadrature nodes and weights checked bit for bit.
#include <math_ComputeKronrodPointsAndWeights.hxx>
void kronrod_case(std::istream& input) {
  int order;input>>order;math_ComputeKronrodPointsAndWeights solver(order);
  std::cout<<solver.IsDone()<<' ';
  if(solver.IsDone()) {auto points=solver.Points();auto weights=solver.Weights();
    for(int index=1;index<=points.Length();++index)std::cout<<bits(points(index))<<' '<<bits(weights(index))<<' ';
  }
  std::cout<<'\n';
}
