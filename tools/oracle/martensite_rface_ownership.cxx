// ToolRFace copies the nonvirtual parent Face, retaining topology while releasing source ownership.
#include "martensite_isoline_faces.hxx"
#include <StdPrs_ToolRFace.hxx>
#include <TopoDS_Edge.hxx>
#include <iostream>
#include <iomanip>
class RFaceLifetimeSource:public BRepAdaptor_Surface {
  TopoDS_Face decoy=isoline_fixture_face(1);
public:
  static int destroyed,hiddenFaceCalls;
  RFaceLifetimeSource():BRepAdaptor_Surface(isoline_fixture_face(0)) {
    // The actual parent core differs from the deliberately hidden, nonvirtual same-name member.
    // Final derivative evaluation remains the real inherited implementation.
  }
  ~RFaceLifetimeSource()override {
    // Lifetime observation is independent of copied topology still owned by the tool.
    ++destroyed;
  }
  const TopoDS_Face& Face()const {
    ++hiddenFaceCalls;
    return decoy;
  }
};
int RFaceLifetimeSource::destroyed=0;int RFaceLifetimeSource::hiddenFaceCalls=0;
static void run_source_ownership_case() {
  occ::handle<RFaceLifetimeSource> source=new RFaceLifetimeSource;
  source->Face();
  occ::handle<BRepAdaptor_Surface> parent=static_cast<BRepAdaptor_Surface*>(source.get());
  const int kind=int(source->GetType());
  StdPrs_ToolRFace tool(parent);source.Nullify();parent.Nullify();tool.Init();
  std::cout<<"0 "<<RFaceLifetimeSource::destroyed<<' '<<RFaceLifetimeSource::hiddenFaceCalls<<' '<<kind;
  int count=0;
  while(tool.More()) {
    const auto p=tool.Value().Value((tool.Value().FirstParameter()+tool.Value().LastParameter())*.5);
    std::cout<<' '<<tool.Edge().IsNull()<<' '<<int(tool.Edge().Orientation())<<' '<<p.X()<<' '<<p.Y();
    ++count;tool.Next();
  }
  std::cout<<' '<<count<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);
  run_source_ownership_case();
}
