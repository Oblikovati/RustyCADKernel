// Precision conversion and borrowed storage oracle, pinned Poly_ArrayOfNodes.
#include <Poly_ArrayOfNodes.hxx>
#include <Poly_ArrayOfUVNodes.hxx>
void nodes_case(std::istream& input) {
  double v[6];for(auto& value:v)input>>value;
  gp_Pnt doubles[2]={{v[0],v[1],v[2]},{v[3],v[4],v[5]}};
  Poly_ArrayOfNodes source(doubles[0],2),target;
  target.SetDoublePrecision(false);target.Resize(size_t{2},false);target.Assign(source);
  xyz(target.Value(size_t{0}).XYZ());xyz(target.Value(size_t{1}).XYZ());
  source.Assign(target);xyz(doubles[0].XYZ());xyz(doubles[1].XYZ());
  target.SetValue(size_t{0},gp_Pnt(v[3],v[1],v[5]));
  Poly_ArrayOfNodes copy(target);copy.Resize(size_t{3},true);
  xyz(copy.Value(size_t{0}).XYZ());xyz(copy.Value(size_t{1}).XYZ());
  std::cout<<source.IsDeletable()<<' '<<target.IsDeletable()<<' '<<copy.IsDoublePrecision()<<' '<<copy.Length()<<' ';
  try {target.SetDoublePrecision(true);}catch(const Standard_Failure& failure){std::cout<<failure.ExceptionType()<<' ';}
  Poly_ArrayOfNodes small(1);
  try {small.Assign(target);}catch(const Standard_Failure& failure){std::cout<<failure.ExceptionType()<<' ';}
  gp_Pnt2d uv[2]={{v[0],v[1]},{v[3],v[4]}};
  Poly_ArrayOfUVNodes uvSource(uv[0],2),uvTarget;
  uvTarget.SetDoublePrecision(false);uvTarget.Resize(size_t{2},false);uvTarget.Assign(uvSource);
  xy(uvTarget.Value(size_t{0}).XY());xy(uvTarget.Value(size_t{1}).XY());
  uvTarget.SetValue(size_t{1},gp_Pnt2d(v[2],v[5]));uvSource.Assign(uvTarget);
  xy(uv[0].XY());xy(uv[1].XY());std::cout<<'\n';
}
