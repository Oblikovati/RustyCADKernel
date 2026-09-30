// The BRepApprox_SurfaceTool probe operation, after tktopalgo_probe_brepapprox.cxx (its
// face reader). The Rust twin is crates/tktopalgo/examples/tp_probe/brep_approx_tool.rs.
void tp_bra_ax3(const gp_Ax3& a){ga_pnt(a.Location());ga_dir(a.Direction());ga_dir(a.XDirection());}
// The intervals of each continuity C0 to CN in U, then in V.
void tp_bra_intervals(const BRepAdaptor_Surface& s){
 for(int k=0;k<=6;++k){GeomAbs_Shape c=GeomAbs_Shape(k);
  GA_TRY(int n=BRepApprox_SurfaceTool::NbUIntervals(s,c);ga_int(n);NCollection_Array1<double> t(1,n+1);BRepApprox_SurfaceTool::UIntervals(s,t,c);for(double x:t)ga_real(x));
  GA_TRY(int n=BRepApprox_SurfaceTool::NbVIntervals(s,c);ga_int(n);NCollection_Array1<double> t(1,n+1);BRepApprox_SurfaceTool::VIntervals(s,t,c);for(double x:t)ga_real(x));}}
// The trimmed adaptors of UTrim and VTrim: their parameter ranges.
void tp_bra_trims(const BRepAdaptor_Surface& s,double u1,double u2,double v1,double v2,double tol){
 GA_TRY(occ::handle<Adaptor3d_Surface> t=BRepApprox_SurfaceTool::UTrim(s,u1,u2,tol);ga_real(t->FirstUParameter());ga_real(t->LastUParameter());ga_real(t->FirstVParameter());ga_real(t->LastVParameter()));
 GA_TRY(occ::handle<Adaptor3d_Surface> t=BRepApprox_SurfaceTool::VTrim(s,v1,v2,tol);ga_real(t->FirstUParameter());ga_real(t->LastUParameter());ga_real(t->FirstVParameter());ga_real(t->LastVParameter()));}
// The closure and the periods.
void tp_bra_periods(const BRepAdaptor_Surface& s){
 ga_flag(BRepApprox_SurfaceTool::IsUClosed(s));ga_flag(BRepApprox_SurfaceTool::IsVClosed(s));
 ga_flag(BRepApprox_SurfaceTool::IsUPeriodic(s));GA_TRY(ga_real(BRepApprox_SurfaceTool::UPeriod(s)));
 ga_flag(BRepApprox_SurfaceTool::IsVPeriodic(s));GA_TRY(ga_real(BRepApprox_SurfaceTool::VPeriod(s)));}
// The evaluations at (u, v): Value, D0 to D3 and DN of orders (1 0), (0 1), (1 1), (2 0).
void tp_bra_evaluate(const BRepAdaptor_Surface& s,double u,double v){
 GA_TRY(ga_pnt(BRepApprox_SurfaceTool::Value(s,u,v)));
 GA_TRY(gp_Pnt p;BRepApprox_SurfaceTool::D0(s,u,v,p);ga_pnt(p));
 GA_TRY(gp_Pnt p;gp_Vec a,b;BRepApprox_SurfaceTool::D1(s,u,v,p,a,b);ga_pnt(p);ga_xyz(a.XYZ());ga_xyz(b.XYZ()));
 GA_TRY(gp_Pnt p;gp_Vec a,b,c,d,e;BRepApprox_SurfaceTool::D2(s,u,v,p,a,b,c,d,e);ga_pnt(p);for(const gp_Vec* x:{&a,&b,&c,&d,&e})ga_xyz(x->XYZ()));
 GA_TRY(gp_Pnt p;gp_Vec a,b,c,d,e,f,g,h,i;BRepApprox_SurfaceTool::D3(s,u,v,p,a,b,c,d,e,f,g,h,i);ga_pnt(p);
  for(const gp_Vec* x:{&a,&b,&c,&d,&e,&f,&g,&h,&i})ga_xyz(x->XYZ()));
 const int orders[4][2]={{1,0},{0,1},{1,1},{2,0}};
 for(const auto& o:orders)GA_TRY(ga_xyz(BRepApprox_SurfaceTool::DN(s,u,v,o[0],o[1]).XYZ()));}
// The kind, the quadric accessors (each raises for another kind), the spline surfaces,
// the swept-surface members and the sample counts.
void tp_bra_kind(const BRepAdaptor_Surface& s,double u1,double u2){
 ga_int(int(BRepApprox_SurfaceTool::GetType(s)));
 GA_TRY(tp_bra_ax3(BRepApprox_SurfaceTool::Plane(s).Position()));
 GA_TRY(gp_Cylinder c=BRepApprox_SurfaceTool::Cylinder(s);tp_bra_ax3(c.Position());ga_real(c.Radius()));
 GA_TRY(gp_Cone c=BRepApprox_SurfaceTool::Cone(s);tp_bra_ax3(c.Position());ga_real(c.RefRadius());ga_real(c.SemiAngle()));
 GA_TRY(gp_Torus c=BRepApprox_SurfaceTool::Torus(s);tp_bra_ax3(c.Position());ga_real(c.MajorRadius());ga_real(c.MinorRadius()));
 GA_TRY(gp_Sphere c=BRepApprox_SurfaceTool::Sphere(s);tp_bra_ax3(c.Position());ga_real(c.Radius()));
 GA_TRY(occ::handle<Geom_BezierSurface> b=BRepApprox_SurfaceTool::Bezier(s);ga_flag(b.IsNull());if(!b.IsNull()){ga_int(b->NbUPoles());ga_int(b->NbVPoles());});
 GA_TRY(occ::handle<Geom_BSplineSurface> b=BRepApprox_SurfaceTool::BSpline(s);ga_flag(b.IsNull());if(!b.IsNull()){ga_int(b->NbUPoles());ga_int(b->NbVPoles());});
 GA_TRY(gp_Ax1 a=BRepApprox_SurfaceTool::AxeOfRevolution(s);ga_pnt(a.Location());ga_dir(a.Direction()));
 GA_TRY(ga_dir(BRepApprox_SurfaceTool::Direction(s)));
 GA_TRY(ga_flag(BRepApprox_SurfaceTool::BasisCurve(s).IsNull()));
 ga_int(BRepApprox_SurfaceTool::NbSamplesU(s));ga_int(BRepApprox_SurfaceTool::NbSamplesV(s));
 ga_int(BRepApprox_SurfaceTool::NbSamplesU(s,u1,u2));ga_int(BRepApprox_SurfaceTool::NbSamplesV(s,u1,u2));}
// `face restr u1 u2 v1 v2 tol r3d n (u v)...`: every BRepApprox_SurfaceTool member on the
// adaptor of the face.
void tp_bra_tool_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpBraFace f=tp_bra_face(r);const BRepAdaptor_Surface& s=*f.s;double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real(),tol=r.real(),r3d=r.real();
  ga_real(BRepApprox_SurfaceTool::FirstUParameter(s));ga_real(BRepApprox_SurfaceTool::LastUParameter(s));
  ga_real(BRepApprox_SurfaceTool::FirstVParameter(s));ga_real(BRepApprox_SurfaceTool::LastVParameter(s));
  tp_bra_intervals(s);tp_bra_trims(s,u1,u2,v1,v2,tol);tp_bra_periods(s);
  GA_TRY(ga_real(BRepApprox_SurfaceTool::UResolution(s,r3d)));GA_TRY(ga_real(BRepApprox_SurfaceTool::VResolution(s,r3d)));
  tp_bra_kind(s,u1,u2);int n=r.integer();for(int k=0;k<n;++k){double u=r.real(),v=r.real();tp_bra_evaluate(s,u,v);});
 ga_end();}
