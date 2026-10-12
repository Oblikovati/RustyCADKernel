// True ToolRFace Edge references bind retained explorer slots, not detached shape copies; OCCT3d097a0.
#include "martensite_isoline_faces.hxx"
#include <StdPrs_ToolRFace.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <TopoDS_Edge.hxx>
#include <iostream>
#include <iomanip>
static void print_edge_reference(const TopoDS_Edge& edge) {
  std::cout<<' '<<edge.IsNull()<<' '<<int(edge.Orientation())<<' '<<edge.Location().IsIdentity();
  if(edge.IsNull()){std::cout<<" -1 -1";return;}
  std::cout<<' '<<int(edge.ShapeType())<<' '<<edge.Modified();
}
static void print_edge_state(StdPrs_ToolRFace& tool,const TopoDS_Edge& root,const TopoDS_Edge* slot,int scenario,int step) {
  int failed=0;bool same=false;try{const auto& current=tool.Edge();same=slot==&current;}catch(Standard_Failure const& failure){failed=std::string(failure.ExceptionType())=="Standard_TypeMismatch"?1:2;}
  std::cout<<scenario<<' '<<step<<' '<<tool.More()<<' '<<failed<<' '<<same;
  print_edge_reference(root);std::cout<<' '<<(slot!=nullptr);
  if(slot)print_edge_reference(*slot);
  if(tool.More()){const auto point=tool.Value().Value((tool.Value().FirstParameter()+tool.Value().LastParameter())*.5);std::cout<<' '<<point.X()<<' '<<point.Y();}
  else std::cout<<" 0 0";
  std::cout<<'\n';
}
static void run_edge_case(int scenario) {
  const occ::handle<BRepAdaptor_Surface> source=new BRepAdaptor_Surface(isoline_fixture_face(scenario==0?0:9));
  StdPrs_ToolRFace tool(source);const auto& root=tool.Edge();int step=0;
  print_edge_state(tool,root,nullptr,scenario,step++);
  tool.Init();const auto& slot=tool.Edge();print_edge_state(tool,root,&slot,scenario,step++);
  auto detached=tool.Edge();detached.Orientation(TopAbs_INTERNAL);detached.Modified(false);
  print_edge_state(tool,root,&slot,scenario,step++);
  while(tool.More()){tool.Next();print_edge_state(tool,root,&slot,scenario,step++);}
  tool.Init();print_edge_state(tool,root,&slot,scenario,step++);
  while(tool.More()){tool.Next();print_edge_state(tool,root,&slot,scenario,step++);}
}
int main() {
  std::cout<<std::setprecision(17);
  for(int scenario=0;scenario<2;++scenario)run_edge_case(scenario);
}
