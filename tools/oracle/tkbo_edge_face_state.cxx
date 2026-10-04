#define main edge_face_original_main
#include "tkbo_edge_face.cxx"
#undef main
#include <BRep_Builder.hxx>
static void state(IntTools_EdgeFace& solver,int stage)
{
 std::cout<<stage<<' '<<solver.IsDone()<<' '<<solver.ErrorStatus()<<' '
          <<solver.CommonParts().Length()<<' '<<solver.Context().IsNull()<<' '
          <<solver.FuzzyValue()<<' '<<solver.IsCoincidenceCheckedQuickly()<<'\n';
}
int main()
{
 std::cout<<std::scientific<<std::setprecision(17);
 IntTools_EdgeFace solver;state(solver,0);
 auto edge=BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,-1),gp_Pnt(0,0,1)).Edge();
 auto face=BRepBuilderAPI_MakeFace(gp_Pln(),-5.,5.,-5.,5.).Face();
 solver.SetEdge(edge);solver.SetFace(face);solver.SetRange(0.,1.);
 solver.Perform();state(solver,1);solver.Perform();state(solver,2);
 BRep_Builder builder;builder.Degenerated(edge,true);
 solver.Perform();state(solver,3);
 TopoDS_Edge empty;builder.MakeEdge(empty);solver.SetEdge(empty);
 solver.Perform();state(solver,4);
 solver.SetFuzzyValue(-1.);state(solver,5);
 solver.SetFuzzyValue(1e-4);solver.SetContext(occ::handle<IntTools_Context>());
 solver.UseQuickCoincidenceCheck(true);
 solver.SetEdge(BRepBuilderAPI_MakeEdge(gp_Pnt(-1,0,0),gp_Pnt(1,0,0)).Edge());
 solver.Perform();state(solver,6);
 return 0;
}
