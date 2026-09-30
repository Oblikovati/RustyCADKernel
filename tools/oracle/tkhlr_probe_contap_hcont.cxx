// Arc members use real OCCT adaptors and vertices.
#include <Adaptor3d_HVertex.hxx>
void ct_arc_constants(const occ::handle<Adaptor2d_Curve2d>& arc){
 typedef Contap_HContTool T;
 ga_flag(T::HasBeenSeen(arc));ga_int(T::NbPoints(arc));
 GA_TRY(gp_Pnt p;double tol,u;T::Value(arc,1,p,tol,u));
 ga_flag(T::IsVertex(arc,1));GA_TRY(occ::handle<Adaptor3d_HVertex> v;T::Vertex(arc,1,v));
 ga_int(T::NbSegments(arc));GA_TRY(int i;T::HasFirstPoint(arc,1,i));
 GA_TRY(int i;T::HasLastPoint(arc,1,i));ga_flag(T::IsAllSolution(arc));
}
void ct_hcont_arc_case(std::istream& input){
 GaReader r{input};typedef Contap_HContTool T;
 GA_TRY(auto arc=ct_arc(r);gp_Pnt2d p=r.pnt2d();
  occ::handle<Adaptor3d_HVertex> vertex=new Adaptor3d_HVertex(p,TopAbs_FORWARD,r.real());
  GA_TRY(ga_int(T::NbSamplesOnArc(arc)));
  GA_TRY(double first,last;T::Bounds(arc,first,last);ga_real(first);ga_real(last));
  GA_TRY(double u;gp_Pnt2d result;bool found=T::Project(arc,p,u,result);ga_flag(found);if(found){ga_real(u);ga_pnt2d(result);});
  GA_TRY(ga_real(T::Tolerance(vertex,arc)));GA_TRY(ga_real(T::Parameter(vertex,arc)));
  ct_arc_constants(arc));
 ga_end();
}
