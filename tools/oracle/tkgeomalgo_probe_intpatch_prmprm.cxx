// IntPatch_PrmPrmIntersection probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_patch_prmprm.rs. The grid members run on a
// command script over one bit map; PointDepart samples two surfaces (ga_ics_surface).
// Needs tkgeomalgo_probe_intwalk.cxx and tkgeomalgo_probe_intcurvesurface.cxx.
#include <IntPatch_PrmPrmIntersection.hxx>
#include <IntPatch_PrmPrmIntersection_T3Bits.hxx>
// The cells set in the map: their count, then the first `limit` of them.
void ga_pp_cells(const IntPatch_PrmPrmIntersection_T3Bits& m,int limit){
 int n=0;std::vector<int> first;
 for(int t=0;t<128*128*128;++t)if(m.Val(t)){++n;if(int(first.size())<limit)first.push_back(t);}
 ga_int(n);for(int t:first)ga_int(t);}
// `n (command args)*`: 1 GrilleInteger ix iy iz, 2 IntegerGrille t, 3 DansGrille t,
// 4 NbPointsGrille, 5 CodeReject x0 y0 z0 x1 y1 z1 x y z, 6 RemplitLin 6 ints,
// 7 RemplitTri 9 ints, 8 Remplit a b c; then the cells of the map.
void ga_ip_ppgrid_case(std::istream& input){
 GaReader r{input};IntPatch_PrmPrmIntersection pp;IntPatch_PrmPrmIntersection_T3Bits m(128);int n=r.integer();
 for(int k=0;k<n;++k){int c=r.integer();
  if(c==1){int a=r.integer(),b=r.integer();ga_int(pp.GrilleInteger(a,b,r.integer()));}
  else if(c==2){int a,b,d;pp.IntegerGrille(r.integer(),a,b,d);ga_int(a);ga_int(b);ga_int(d);}
  else if(c==3)ga_int(pp.DansGrille(r.integer()));
  else if(c==4)ga_int(pp.NbPointsGrille());
  else if(c==5){double v[9];for(double& x:v)x=r.real();ga_int(pp.CodeReject(v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7],v[8]));}
  else if(c==6){int v[6];for(int& x:v)x=r.integer();pp.RemplitLin(v[0],v[1],v[2],v[3],v[4],v[5],m);}
  else if(c==7){int v[9];for(int& x:v)x=r.integer();pp.RemplitTri(v[0],v[1],v[2],v[3],v[4],v[5],v[6],v[7],v[8],m);}
  else{int a=r.integer(),b=r.integer();pp.Remplit(a,b,r.integer(),m);}}
 ga_pp_cells(m,40);ga_flag(pp.IsDone());GA_TRY(ga_flag(pp.IsEmpty()));GA_TRY(ga_int(pp.NbLines()));GA_TRY(pp.Line(1));
 ga_end();
}
// `s1 su1 sv1 s2 su2 sv2`: the start points of PointDepart.
void ga_ip_ppdepart_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);int su1=r.integer(),sv1=r.integer();
  occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);int su2=r.integer(),sv2=r.integer();
  occ::handle<IntSurf_LineOn2S> l=new IntSurf_LineOn2S();IntPatch_PrmPrmIntersection pp;pp.PointDepart(l,s1,su1,sv1,s2,su2,sv2);
  ga_int(l->NbPoints());for(int i=1;i<=l->NbPoints();++i)ga_iw_pnt2s(l->Value(i)));
 ga_end();
}
