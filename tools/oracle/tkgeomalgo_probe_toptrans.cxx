// TopTrans probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/toptrans.rs. A request is a count of commands on
// one fresh transition; after each command the probe prints the raised exception type,
// if any, then the states before and after.
#include <TopTrans_CurveTransition.hxx>
#include <TopTrans_SurfaceTransition.hxx>
TopAbs_Orientation ga_orientation(GaReader& r){return TopAbs_Orientation(r.integer());}
// Commands: 0 tgt norm curv (Reset), 1 tgt (Reset of a line),
// 2 tole tang norm curv transition orientation (Compare).
void ga_tt_curve_case(std::istream& input){
 GaReader r{input};TopTrans_CurveTransition transition;int count=r.integer();
 for(int k=0;k<count;++k){
  int command=r.integer();
  if(command==0){gp_Dir t=r.dir(),n=r.dir();transition.Reset(t,n,r.real());}
  else if(command==1){transition.Reset(r.dir());}
  else{double tole=r.real();gp_Dir t=r.dir(),n=r.dir();double c=r.real();
   TopAbs_Orientation s=ga_orientation(r);transition.Compare(tole,t,n,c,s,ga_orientation(r));}
  ga_int(transition.StateBefore());ga_int(transition.StateAfter());}
 ga_end();
}
// Commands: 0 tgt norm (Reset of a line), 1 tgt norm maxd mind maxcurv mincurv (Reset),
// 2 tole norm transition orientation (Compare of a plane),
// 3 tole norm maxd mind maxcurv mincurv transition orientation (Compare).
void ga_tt_surface_command(GaReader& r,TopTrans_SurfaceTransition& transition,int command){
 if(command==0){gp_Dir t=r.dir(),n=r.dir();GA_TRY(transition.Reset(t,n));return;}
 if(command==1){gp_Dir t=r.dir(),n=r.dir(),mx=r.dir(),mn=r.dir();double cx=r.real(),cn=r.real();
  GA_TRY(transition.Reset(t,n,mx,mn,cx,cn));return;}
 double tole=r.real();gp_Dir n=r.dir();
 if(command==2){TopAbs_Orientation s=ga_orientation(r),o=ga_orientation(r);
  GA_TRY(transition.Compare(tole,n,s,o));return;}
 gp_Dir mx=r.dir(),mn=r.dir();double cx=r.real(),cn=r.real();
 TopAbs_Orientation s=ga_orientation(r),o=ga_orientation(r);
 GA_TRY(transition.Compare(tole,n,mx,mn,cx,cn,s,o));
}
void ga_tt_surface_case(std::istream& input){
 GaReader r{input};TopTrans_SurfaceTransition transition;int count=r.integer();
 for(int k=0;k<count;++k){
  ga_tt_surface_command(r,transition,r.integer());
  ga_int(transition.StateBefore());ga_int(transition.StateAfter());}
 ga_end();
}
// `orientation`: the static state tables of TopTrans_SurfaceTransition.
void ga_tt_states_case(std::istream& input){
 GaReader r{input};TopAbs_Orientation o=ga_orientation(r);
 ga_int(TopTrans_SurfaceTransition::GetBefore(o));ga_int(TopTrans_SurfaceTransition::GetAfter(o));ga_end();
}
