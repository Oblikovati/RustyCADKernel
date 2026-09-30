// The BRepApprox multi-line, SvSurfaces, IntImp instances and surface tool probe
// operations, after tktopalgo_probe_brepapprox.cxx (its walk and SvSurfaces readers). The
// Rust twin is crates/tktopalgo/examples/tp_probe/brep_approx_parts.rs.
// The multipoint `i` of a multi-line (3d then 2d points, the tool's Value).
void tp_bra_ml_value(const BRepApprox_TheMultiLineOfApprox& m,int i){
 int n3=m.NbP3d(),n2=m.NbP2d();NCollection_Array1<gp_Pnt> p(1,std::max(1,n3));NCollection_Array1<gp_Pnt2d> q(1,std::max(1,n2));
 GA_TRY(if(n3!=0&&n2!=0)BRepApprox_TheMultiLineToolOfApprox::Value(m,i,p,q);else if(n2!=0)BRepApprox_TheMultiLineToolOfApprox::Value(m,i,q);
  else if(n3!=0)BRepApprox_TheMultiLineToolOfApprox::Value(m,i,p);for(const gp_Pnt& x:p)ga_pnt(x);for(const gp_Pnt2d& x:q)ga_pnt2d(x));}
// The tangents of the multipoint `i` (the tool's three Tangency members and Curvature).
void tp_bra_ml_tangency(const BRepApprox_TheMultiLineOfApprox& m,int i){
 int n3=std::max(1,m.NbP3d()),n2=std::max(1,m.NbP2d());
 NCollection_Array1<gp_Vec> v(1,n3);NCollection_Array1<gp_Vec2d> w(1,n2);
 GA_TRY(ga_flag(BRepApprox_TheMultiLineToolOfApprox::Tangency(m,i,v));for(const gp_Vec& x:v)ga_xyz(x.XYZ()));
 GA_TRY(ga_flag(BRepApprox_TheMultiLineToolOfApprox::Tangency(m,i,w));for(const gp_Vec2d& x:w)ga_xy(x.XY()));
 GA_TRY(ga_flag(BRepApprox_TheMultiLineToolOfApprox::Tangency(m,i,v,w));for(const gp_Vec& x:v)ga_xyz(x.XYZ());for(const gp_Vec2d& x:w)ga_xy(x.XY()));
 GA_TRY(ga_flag(BRepApprox_TheMultiLineToolOfApprox::Curvature(m,i,v)));}
// A multi-line: its range, sizes and status, and its multipoints (with one index outside at
// each end).
void tp_bra_ml_report(const BRepApprox_TheMultiLineOfApprox& m){
 int f=BRepApprox_TheMultiLineToolOfApprox::FirstPoint(m),l=BRepApprox_TheMultiLineToolOfApprox::LastPoint(m);
 ga_int(f);ga_int(l);ga_int(BRepApprox_TheMultiLineToolOfApprox::NbP3d(m));ga_int(BRepApprox_TheMultiLineToolOfApprox::NbP2d(m));
 ga_int(BRepApprox_TheMultiLineToolOfApprox::WhatStatus(m,f,l));for(int i=f-1;i<=l+1;++i)tp_bra_ml_value(m,i);}
// One operation on a multi-line: 1 MakeMLBetween `low high nb`, 2 MakeMLOneMorePoint `low
// high bad`, 3 Dump, 4 BRepApprox_Approx::Parameters of the parametrization `kind`.
void tp_bra_ml_op(GaReader& r,const BRepApprox_TheMultiLineOfApprox& m){
 int op=r.integer();
 if(op==3){ga_ip_captured([&]{BRepApprox_TheMultiLineToolOfApprox::Dump(m);});return;}
 if(op==4){int f=m.FirstPoint(),l=m.LastPoint();Approx_ParametrizationType t=Approx_ParametrizationType(r.integer());
  GA_TRY(math_Vector p(f,l);BRepApprox_Approx::Parameters(m,f,l,t,p);for(int i=p.Lower();i<=p.Upper();++i)ga_real(p(i)));return;}
 int a=r.integer(),b=r.integer(),c=r.integer();
 if(op==1){GA_TRY(tp_bra_ml_report(BRepApprox_TheMultiLineToolOfApprox::MakeMLBetween(m,a,b,c)));return;}
 BRepApprox_TheMultiLineOfApprox other;
 GA_TRY(bool made=BRepApprox_TheMultiLineToolOfApprox::MakeMLOneMorePoint(m,a,b,c,other);ga_flag(made);if(made)tp_bra_ml_report(other));}
// `walk svkind line nbp3d nbp2d u1v1 u2v2 xo yo zo u1o v1o u2o v2o p2donfirst range ntan
// (i)... nops (op args)...`: a multi-line on walking line `line` with the SvSurfaces of
// `svkind` (0 none), its tangents at the given indices, then the operations.
void tp_bra_mline_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpBraWalk w;tp_bra_walk(r,w);int kind=r.integer();occ::handle<BRepApprox_ApproxLine> l=tp_bra_line(w,r.integer());
  TpBraSv s;tp_bra_sv(w,kind,s);ApproxInt_SvSurfaces* sv=s.sv.get();
  int n3=r.integer(),n2=r.integer();bool a1=r.flag(),a2=r.flag();double o[7];for(double& x:o)x=r.real();bool first=r.flag();
  int lo=0,hi=0;if(!tp_bra_range(r,l,lo,hi)){ga_end();return;}
  BRepApprox_TheMultiLineOfApprox m=sv?BRepApprox_TheMultiLineOfApprox(l,sv,n3,n2,a1,a2,o[0],o[1],o[2],o[3],o[4],o[5],o[6],first,lo,hi)
   :BRepApprox_TheMultiLineOfApprox(l,n3,n2,a1,a2,o[0],o[1],o[2],o[3],o[4],o[5],o[6],first,lo,hi);
  tp_bra_ml_report(m);int nt=r.integer();for(int k=0;k<nt;++k)tp_bra_ml_tangency(m,r.integer());
  int nops=r.integer();for(int k=0;k<nops;++k)tp_bra_ml_op(r,m);
  if(sv)ga_flag(sv->GetUseSolver()));
 ga_end();}
// One SvSurfaces operation on `u1 v1 u2 v2`: 0 Compute, 1 Pnt, 2 SeekPoint, 3 Tangency,
// 4 TangencyOnSurf1, 5 TangencyOnSurf2, 7 FillInitialVectorOfSolution (ImpPrm only).
void tp_bra_sv_op(const TpBraWalk& w,int kind,ApproxInt_SvSurfaces* sv,int op,double u1,double v1,double u2,double v2){
 if(op==0){gp_Pnt p;gp_Vec t;gp_Vec2d a,b;ga_flag(sv->Compute(u1,v1,u2,v2,p,t,a,b));ga_real(u1);ga_real(v1);ga_real(u2);ga_real(v2);
  ga_pnt(p);ga_xyz(t.XYZ());ga_xy(a.XY());ga_xy(b.XY());}
 else if(op==1){gp_Pnt p;sv->Pnt(u1,v1,u2,v2,p);ga_pnt(p);}
 else if(op==2){IntSurf_PntOn2S p;bool ok=sv->SeekPoint(u1,v1,u2,v2,p);ga_flag(ok);if(ok)ga_iw_pnt2s(p);}
 else if(op==3){gp_Vec t;ga_flag(sv->Tangency(u1,v1,u2,v2,t));ga_xyz(t.XYZ());}
 else if(op==4){gp_Vec2d t;ga_flag(sv->TangencyOnSurf1(u1,v1,u2,v2,t));ga_xy(t.XY());}
 else if(op==5){gp_Vec2d t;ga_flag(sv->TangencyOnSurf2(u1,v1,u2,v2,t));ga_xy(t.XY());}
 else{BRepApprox_TheImpPrmSvSurfacesOfApprox* ip=dynamic_cast<BRepApprox_TheImpPrmSvSurfacesOfApprox*>(sv);
  const BRepAdaptor_Surface& ps=(kind==2)?*w.b.s:*w.a.s;math_Vector x(1,2,0.);double tu=0,tv=0;
  bool ok=ip->FillInitialVectorOfSolution(u1,v1,u2,v2,ps.FirstUParameter(),ps.LastUParameter(),ps.FirstVParameter(),ps.LastVParameter(),x,tu,tv);
  ga_flag(ok);if(ok){ga_real(x(1));ga_real(x(2));ga_real(tu);ga_real(tv);}}}
// `walk svkind nops (op args)...`: the SvSurfaces of `svkind` (1 to 3); op 6 SetUseSolver
// `flag`, the others on `u1 v1 u2 v2`.
void tp_bra_sv_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpBraWalk w;tp_bra_walk(r,w);int kind=r.integer();TpBraSv s;tp_bra_sv(w,kind,s);ApproxInt_SvSurfaces* sv=s.sv.get();
  ga_flag(sv->GetUseSolver());int nops=r.integer();
  for(int k=0;k<nops;++k){int op=r.integer();
   if(op==6){sv->SetUseSolver(r.flag());ga_flag(sv->GetUseSolver());continue;}
   double u1=r.real(),v1=r.real(),u2=r.real(),v2=r.real();GA_TRY(tp_bra_sv_op(w,kind,sv,op,u1,v1,u2,v2));});
 ga_end();}
void tp_bra_vector(const math_Vector& v){for(int i=v.Lower();i<=v.Upper();i++)ga_real(v(i));}
void tp_bra_matrix(const math_Matrix& m){for(int i=m.LowerRow();i<=m.UpperRow();i++)for(int j=m.LowerCol();j<=m.UpperCol();j++)ga_real(m(i,j));}
void tp_bra_dir2d(const gp_Dir2d& d){ga_real(d.X());ga_real(d.Y());}
// `face restr quadric tol u v`: BRepApprox_TheZerImpFuncOfTheImpPrmSvSurfacesOfApprox
// evaluated at (u, v) and a second point, with its tangency and directions.
void tp_bra_zerimp_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpBraFace f=tp_bra_face(r);IntSurf_Quadric q=ga_intsurf_quadric(r);double tol=r.real();math_Vector x(1,2);x(1)=r.real();x(2)=r.real();
  BRepApprox_TheZerImpFuncOfTheImpPrmSvSurfacesOfApprox z(*f.s,q);z.Set(tol);ga_real(z.Tolerance());ga_int(z.NbVariables());ga_int(z.NbEquations());
  math_Vector fv(1,1,0.0);math_Matrix d(1,1,1,2,0.0);ga_flag(z.Value(x,fv));ga_real(fv(1));ga_real(z.Root());ga_pnt(z.Point());
  GA_TRY(ga_flag(z.IsTangent()));ga_flag(z.Derivatives(x,d));tp_bra_matrix(d);GA_TRY(ga_flag(z.IsTangent()));
  math_Vector x2(1,2);x2(1)=x(1)+0.02;x2(2)=x(2)-0.01;ga_flag(z.Values(x2,fv,d));ga_real(fv(1));tp_bra_matrix(d);
  GA_TRY(ga_flag(z.IsTangent()));GA_TRY(ga_xyz(z.Direction3d().XYZ()));GA_TRY(tp_bra_dir2d(z.Direction2d()));ga_pnt(z.Point()));
 ga_end();}
void tp_bra_int2s_report(const BRepApprox_TheInt2SOfThePrmPrmSvSurfacesOfApprox& s){
 ga_flag(s.IsDone());GA_TRY(ga_flag(s.IsEmpty()));
 GA_TRY(const IntSurf_PntOn2S& p=s.Point();ga_iw_pnt2s(p));
 GA_TRY(ga_flag(s.IsTangent()));GA_TRY(ga_dir(s.Direction()));
 GA_TRY(tp_bra_dir2d(s.DirectionOnS1()));GA_TRY(tp_bra_dir2d(s.DirectionOnS2()));}
// `face1 restr1 face2 restr2 tol mode iso u1 v1 u2 v2`:
// BRepApprox_TheInt2SOfThePrmPrmSvSurfacesOfApprox at a start point (mode 0), or Perform
// on the isoparametric `iso` (mode 1), with the values of its function at the start.
void tp_bra_int2s_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpBraFace a=tp_bra_face(r);TpBraFace b=tp_bra_face(r);double tol=r.real();int mode=r.integer(),iso=r.integer();
  NCollection_Array1<double> param(1,4);for(int i=1;i<=4;i++)param(i)=r.real();
  if(mode==0){BRepApprox_TheInt2SOfThePrmPrmSvSurfacesOfApprox s(param,*a.s,*b.s,tol);tp_bra_int2s_report(s);}
  else{BRepApprox_TheInt2SOfThePrmPrmSvSurfacesOfApprox s(*a.s,*b.s,tol);math_FunctionSetRoot rs(s.Function(),15);
   ga_int(int(s.Perform(param,rs,IntImp_ConstIsoparametric(iso))));tp_bra_int2s_report(s);
   BRepApprox_TheFunctionOfTheInt2SOfThePrmPrmSvSurfacesOfApprox& f=s.Function();math_Vector uv(1,3,0.0),inf(1,3,0.0),sup(1,3,0.0),t(1,3,0.0);
   f.ComputeParameters(IntImp_ConstIsoparametric(iso),param,uv,inf,sup,t);tp_bra_vector(uv);tp_bra_vector(inf);tp_bra_vector(sup);tp_bra_vector(t);
   math_Vector fv(1,3,0.0);math_Matrix d(1,3,1,3,0.0);ga_flag(f.Values(uv,fv,d));tp_bra_vector(fv);tp_bra_matrix(d);ga_real(f.Root());ga_pnt(f.Point());});
 ga_end();}
