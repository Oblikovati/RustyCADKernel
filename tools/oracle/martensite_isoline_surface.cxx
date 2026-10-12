// Real surface polyline/UV outputs, retained seeds and U/V aliases; Martensite OCCT3d097a0.
#include "martensite_isoline_faces.hxx"
#include <StdPrs_Isolines.hxx>
#include <Prs3d_Drawer.hxx>
#include <Prs3d_IsoAspect.hxx>
#include <NCollection_HSequence.hxx>
#include <iostream>
#include <iomanip>
using IsoList=NCollection_List<occ::handle<NCollection_HSequence<gp_Pnt>>>;
static void print_isoline_list(const IsoList& lines) {
  std::cout<<' '<<lines.Size();
  for(const auto& line:lines) {
    std::cout<<' '<<line->Length();
    for(int i=1;i<=line->Length();++i){const auto& p=line->Value(i);std::cout<<' '<<p.X()<<' '<<p.Y()<<' '<<p.Z();}
  }
}
static void seed_isoline_list(IsoList& list) {
  const occ::handle<NCollection_HSequence<gp_Pnt>> points=new NCollection_HSequence<gp_Pnt>;
  points->Append(gp_Pnt(42,43,44));list.Append(points);
}
static void print_isoline_parameters(const TopoDS_Face& face,int kind) {
  NCollection_Sequence<double> u,v;u.Append(91);v.Append(92);
  double bounds[]={71,72,73,74};
  StdPrs_Isolines::UVIsoParameters(face,2,3,100,u,v,bounds[0],bounds[1],bounds[2],bounds[3]);
  std::cout<<"params "<<kind;for(double value:bounds)std::cout<<' '<<value;
  std::cout<<' '<<u.Length();for(double value:u)std::cout<<' '<<value;
  std::cout<<' '<<v.Length();for(double value:v)std::cout<<' '<<value;std::cout<<'\n';
}
static void run_isoline_surface_case(const TopoDS_Face& face,int kind,int shared) {
  const occ::handle<Prs3d_Drawer> drawer=new Prs3d_Drawer;drawer->SetupOwnDefaults();
  drawer->UIsoAspect()->SetNumber(2);drawer->VIsoAspect()->SetNumber(3);
  IsoList u,v;seed_isoline_list(u);seed_isoline_list(v);
  bool failure=false;
  try {StdPrs_Isolines::AddOnSurface(face,drawer,.2,u,shared?u:v);}catch(Standard_Failure const&){failure=true;}
  std::cout<<"surface "<<kind<<' '<<shared<<' '<<failure;print_isoline_list(u);print_isoline_list(v);std::cout<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);
  for(int kind=0;kind<10;++kind){const auto face=isoline_fixture_face(kind);print_isoline_parameters(face,kind);run_isoline_surface_case(face,kind,0);run_isoline_surface_case(face,kind,1);}
}
