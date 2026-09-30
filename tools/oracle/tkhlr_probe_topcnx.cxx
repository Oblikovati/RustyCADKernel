// TopBas and TopCnx, the Rust twin of crates/tkhlr/examples/hl_probe/top_cnx.rs: the
// interference record and the edge-face transition.
#include <TopAbs_Orientation.hxx>
#include <TopBas_TestInterference.hxx>
#include <TopCnx_EdgeFaceTransition.hxx>
TopAbs_Orientation hl_orientation(GaReader& r){return TopAbs_Orientation(r.integer());}
// `mode I B O T BT [x n]`: mode 0 sets the values with the setters on a default record
// (reading a default record is undefined natively: its fields are uninitialized), mode 1
// uses the constructor, mode 2 the constructor then ChangeIntersection() = x and
// ChangeBoundary() += n; then the five values.
void hl_tb_interference_case(std::istream& input){
 GaReader r{input};int mode=r.integer();double inter=r.real();int bound=r.integer();
 TopAbs_Orientation o=hl_orientation(r),t=hl_orientation(r),bt=hl_orientation(r);
 TopBas_TestInterference record;
 if(mode==0){record.Intersection(inter);record.Boundary(bound);record.Orientation(o);record.Transition(t);record.BoundaryTransition(bt);}
 else record=TopBas_TestInterference(inter,bound,o,t,bt);
 if(mode==2){double x=r.real();int n=r.integer();record.ChangeIntersection()=x;record.ChangeBoundary()+=n;}
 ga_real(record.Intersection());ga_int(record.Boundary());ga_int(record.Orientation());
 ga_int(record.Transition());ga_int(record.BoundaryTransition());ga_end();}
// `n commands`: 0 Reset(T, N, C), 1 Reset(T), 2 AddInterference(tol, T, N, C, Or, Tr, BTr);
// Transition and BoundaryTransition before the first command and after each.
void hl_tc_transition_case(std::istream& input){
 GaReader r{input};TopCnx_EdgeFaceTransition transition;
 ga_int(transition.Transition());ga_int(transition.BoundaryTransition());
 for(int n=r.integer();n>0;--n){
  int command=r.integer();
  if(command==0){gp_Dir t=r.dir();gp_Dir nn=r.dir();transition.Reset(t,nn,r.real());}
  else if(command==1)transition.Reset(r.dir());
  else{double tole=r.real();gp_Dir t=r.dir();gp_Dir nn=r.dir();double c=r.real();
   TopAbs_Orientation o=hl_orientation(r),tr=hl_orientation(r),bt=hl_orientation(r);
   transition.AddInterference(tole,t,nn,c,o,tr,bt);}
  ga_int(transition.Transition());ga_int(transition.BoundaryTransition());}
 ga_end();}
