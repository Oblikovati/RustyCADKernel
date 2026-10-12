// Null-surface no-write, infinite limits, negative counts and observable closure failure outputs.
#include "martensite_isoline_faces.hxx"
#include <StdPrs_Isolines.hxx>
#include <BRep_Builder.hxx>
#include <iostream>
#include <iomanip>
#include "martensite_isoline_alias_surface.hxx"
static TopoDS_Face parameter_edge_face(int scenario) {
  TopoDS_Face face;
  if(scenario==0){BRep_Builder().MakeFace(face);return face;}
  if(scenario==1)return BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3()));
  if(scenario==2)return isoline_fixture_face(0);
  const occ::handle<IsolineAliasSurface> source=new IsolineAliasSurface;source->failClosed=true;
  BRep_Builder().MakeFace(face,static_cast<Geom_Surface*>(source.get()),1e-7);
  return face;
}
static void parameter_edge_case(int scenario) {
  NCollection_Sequence<double> u,v;u.Append(91.);v.Append(92.);
  double bounds[4]={71.,72.,73.,74.};bool failed=false;
  try{StdPrs_Isolines::UVIsoParameters(parameter_edge_face(scenario),scenario==2?-1:2,3,100.,u,v,bounds[0],bounds[1],bounds[2],bounds[3]);}
  catch(Standard_Failure const&){failed=true;}
  std::cout<<scenario<<' '<<failed;for(double value:bounds)std::cout<<' '<<value;
  std::cout<<' '<<u.Length();for(double value:u)std::cout<<' '<<value;
  std::cout<<' '<<v.Length();for(double value:v)std::cout<<' '<<value;std::cout<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);
  for(int scenario=0;scenario<4;++scenario)parameter_edge_case(scenario);
}
