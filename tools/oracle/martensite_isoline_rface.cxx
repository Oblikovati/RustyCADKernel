// Actual ToolRFace borrowed adaptor reload/reset and sticky geometry flag, OCCT3d097a0.
#include "martensite_isoline_faces.hxx"
#include <StdPrs_ToolRFace.hxx>
#include <BRep_Tool.hxx>
#include <BRep_Builder.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <TopoDS.hxx>
#include <TopExp_Explorer.hxx>
#include <iostream>
#include <iomanip>
static void print_rface_state(const StdPrs_ToolRFace& tool,const Adaptor2d_Curve2d& alias,int scenario,int step) {
  std::cout<<scenario<<' '<<step<<' '<<tool.More()<<' '<<tool.IsOriented()<<' '<<tool.IsInvalidGeometry()<<' '<<(&alias==&tool.Value());
  std::cout<<' '<<int(alias.GetType())<<' '<<alias.FirstParameter()<<' '<<alias.LastParameter();
  if(tool.More()){const auto p=alias.Value((alias.FirstParameter()+alias.LastParameter())*.5);std::cout<<' '<<int(tool.Orientation())<<' '<<p.X()<<' '<<p.Y();}
  else std::cout<<" -1 0 0";
  std::cout<<'\n';
}
static void unoriented_rface_edge(const TopoDS_Face& face,const TopoDS_Edge& edge,int scenario) {
  TopExp_Explorer wires(face,TopAbs_WIRE);auto wire=TopoDS::Wire(wires.Current());
  wire.Free(true);BRep_Builder builder;builder.Remove(wire,edge);
  auto changed=edge;changed.Orientation(scenario==3?TopAbs_INTERNAL:TopAbs_EXTERNAL);
  builder.Add(wire,changed);
}
static void run_rface_case(int scenario) {
  auto face=isoline_fixture_face(1);if(scenario==1)face.Reverse();
  TopExp_Explorer edges(face,TopAbs_EDGE);const auto edge=TopoDS::Edge(edges.Current());
  double first,last;const auto original=BRep_Tool::CurveOnSurface(edge,face,first,last);
  BRep_Builder builder;if(scenario==2)builder.UpdateEdge(edge,occ::handle<Geom2d_Curve>(),face,1e-7);
  if(scenario>=3)unoriented_rface_edge(face,edge,scenario);
  const occ::handle<BRepAdaptor_Surface> surface=new BRepAdaptor_Surface(face);
  StdPrs_ToolRFace tool(surface);const auto& alias=tool.Value();int step=0;
  print_rface_state(tool,alias,scenario,step++);tool.Init();print_rface_state(tool,alias,scenario,step++);
  while(tool.More()){tool.Next();print_rface_state(tool,alias,scenario,step++);}
  if(scenario==2)builder.UpdateEdge(edge,original,face,1e-7);
  tool.Init();print_rface_state(tool,alias,scenario,step++);
  while(tool.More()){tool.Next();print_rface_state(tool,alias,scenario,step++);}
}
int main() {
  std::cout<<std::setprecision(17);
  StdPrs_ToolRFace empty;print_rface_state(empty,empty.Value(),-1,0);empty.Init();print_rface_state(empty,empty.Value(),-1,1);
  for(int scenario=0;scenario<5;++scenario)run_rface_case(scenario);
}
