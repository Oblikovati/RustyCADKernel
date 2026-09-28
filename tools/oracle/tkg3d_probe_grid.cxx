// TKG3d oracle: the GeomGridEval grid evaluators (tkg3d_probe.cxx).
// Parameters are read as `lower n u*n` into NCollection_Array1(lower, lower + n - 1), or
// the default empty array for n = 0. Every grid prints its bounds, then its values; a
// raised exception prints its type instead.
NCollection_Array1<double> g3_grid_params(G3Reader& r){
 int lower=r.integer();std::vector<double> u=r.reals();if(u.empty())return NCollection_Array1<double>();
 NCollection_Array1<double> p(lower,lower+int(u.size())-1);for(size_t i=0;i<u.size();++i)p(lower+int(i))=u[i];return p;
}
void g3_grid_value(const gp_Pnt& p){g3_pnt(p);}
void g3_grid_value(const gp_Vec& v){g3_vec(v);}
void g3_grid_value(const GeomGridEval::CurveD1& r){g3_pnt(r.Point);g3_vec(r.D1);}
void g3_grid_value(const GeomGridEval::CurveD2& r){g3_pnt(r.Point);g3_vec(r.D1);g3_vec(r.D2);}
void g3_grid_value(const GeomGridEval::CurveD3& r){g3_pnt(r.Point);g3_vec(r.D1);g3_vec(r.D2);g3_vec(r.D3);}
void g3_grid_value(const GeomGridEval::SurfD1& r){g3_pnt(r.Point);g3_vec(r.D1U);g3_vec(r.D1V);}
void g3_grid_value(const GeomGridEval::SurfD2& r){g3_pnt(r.Point);g3_vec(r.D1U);g3_vec(r.D1V);g3_vec(r.D2U);g3_vec(r.D2V);g3_vec(r.D2UV);}
void g3_grid_value(const GeomGridEval::SurfD3& r){
 g3_pnt(r.Point);g3_vec(r.D1U);g3_vec(r.D1V);g3_vec(r.D2U);g3_vec(r.D2V);g3_vec(r.D2UV);
 g3_vec(r.D3U);g3_vec(r.D3V);g3_vec(r.D3UUV);g3_vec(r.D3UVV);
}
template<class T> void g3_grid_array(const NCollection_Array1<T>& a){g3_int(a.Lower());g3_int(a.Length());for(const T& x:a)g3_grid_value(x);}
template<class T> void g3_grid_array(const NCollection_Array2<T>& a){
 g3_int(a.LowerRow());g3_int(a.NbRows());g3_int(a.LowerCol());g3_int(a.NbColumns());
 for(int i=a.LowerRow();i<=a.UpperRow();++i)for(int j=a.LowerCol();j<=a.UpperCol();++j)g3_grid_value(a(i,j));
}
// Every member of one curve evaluator: EvaluateGrid, D1, D2, D3, then DN for N = -1..7.
template<class E> void g3_grid_curve_all(const E& e,const NCollection_Array1<double>& p){
 G3_TRY(g3_grid_array(e.EvaluateGrid(p)));G3_TRY(g3_grid_array(e.EvaluateGridD1(p)));
 G3_TRY(g3_grid_array(e.EvaluateGridD2(p)));G3_TRY(g3_grid_array(e.EvaluateGridD3(p)));
 for(int n=-1;n<=7;++n)G3_TRY(g3_grid_array(e.EvaluateGridDN(p,n)));
}
// The DN orders of a surface evaluator: every (NU, NV) in -1..4 when full, a sample otherwise.
const int g3_grid_sample_orders[][2]={{1,0},{0,1},{1,1},{2,1},{0,3},{4,0},{-1,1},{0,0}};
template<class E> void g3_grid_surface_all(const E& e,const NCollection_Array1<double>& u,const NCollection_Array1<double>& v,bool full){
 G3_TRY(g3_grid_array(e.EvaluateGrid(u,v)));G3_TRY(g3_grid_array(e.EvaluateGridD1(u,v)));
 G3_TRY(g3_grid_array(e.EvaluateGridD2(u,v)));G3_TRY(g3_grid_array(e.EvaluateGridD3(u,v)));
 if(full){for(int nu=-1;nu<=4;++nu)for(int nv=-1;nv<=4;++nv)G3_TRY(g3_grid_array(e.EvaluateGridDN(u,v,nu,nv)));return;}
 for(const auto& o:g3_grid_sample_orders)G3_TRY(g3_grid_array(e.EvaluateGridDN(u,v,o[0],o[1])));
}
template<class G,class E> bool g3_grid_curve_direct_as(const occ::handle<Geom_Curve>& c,const NCollection_Array1<double>& p){
 occ::handle<G> g=occ::down_cast<G>(c);if(g.IsNull())return false;E e(g);g3_flag(e.Geometry().IsNull());g3_grid_curve_all(e,p);return true;
}
// The specialized evaluator of the curve's own class, when it has one.
void g3_grid_curve_direct(const occ::handle<Geom_Curve>& c,const NCollection_Array1<double>& p){
 g3_grid_curve_direct_as<Geom_Line,GeomGridEval_Line>(c,p)||g3_grid_curve_direct_as<Geom_Circle,GeomGridEval_Circle>(c,p)
 ||g3_grid_curve_direct_as<Geom_Ellipse,GeomGridEval_Ellipse>(c,p)||g3_grid_curve_direct_as<Geom_Hyperbola,GeomGridEval_Hyperbola>(c,p)
 ||g3_grid_curve_direct_as<Geom_Parabola,GeomGridEval_Parabola>(c,p)||g3_grid_curve_direct_as<Geom_BezierCurve,GeomGridEval_BezierCurve>(c,p)
 ||g3_grid_curve_direct_as<Geom_BSplineCurve,GeomGridEval_BSplineCurve>(c,p)||g3_grid_curve_direct_as<Geom_OffsetCurve,GeomGridEval_OffsetCurve>(c,p);
}
void g3_grid_curve_dispatch(const GeomGridEval_Curve& e,const NCollection_Array1<double>& p){g3_int(e.GetType());g3_grid_curve_all(e,p);}
// g3_grid <curve> lower n u*n: the dispatcher on the handle and on a GeomAdaptor_Curve, the
// specialized evaluator, and the adaptor fallback on the same adaptor.
void g3_grid_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Curve> c=g3_built_curve(r);if(c.IsNull())return;NCollection_Array1<double> p=g3_grid_params(r);
 g3_grid_curve_dispatch(GeomGridEval_Curve(c),p);GeomAdaptor_Curve a(c);g3_grid_curve_dispatch(GeomGridEval_Curve(a),p);
 g3_grid_curve_direct(c,p);g3_grid_curve_all(GeomGridEval_OtherCurve(a),p);std::cout<<'\n';
}
// A grid curve adaptor: 0 GeomAdaptor_Curve(C) <curve> | 1 u1 u2 <curve> (ranged) |
// 2 Adaptor3d_Curve (the base class) | 3 trsf <curve> (GeomAdaptor_TransformedCurve) |
// 4 iso param <a3 surface adaptor> (Adaptor3d_IsoCurve, 0 IsoU 1 IsoV) |
// 5 <a3 2D adaptor> <a3 surface adaptor> (Adaptor3d_CurveOnSurface).
occ::handle<Adaptor3d_Curve> g3_grid_curve_adaptor(G3Reader& r){
 int k=r.integer();
 if(k==0)return new GeomAdaptor_Curve(g3_curve(r));
 if(k==1){double u1=r.real(),u2=r.real();return new GeomAdaptor_Curve(g3_curve(r),u1,u2);}
 if(k==2)return new Adaptor3d_Curve();
 if(k==3){gp_Trsf t=g3_trsf(r);return new GeomAdaptor_TransformedCurve(g3_curve(r),t);}
 if(k==4){int iso=r.integer();double w=r.real();return new Adaptor3d_IsoCurve(a3_surface(r),GeomAbs_IsoType(iso),w);}
 if(k==5){occ::handle<Adaptor2d_Curve2d> c=a3_adaptor2d(r);return new Adaptor3d_CurveOnSurface(c,a3_surface(r));}
 throw Standard_ProgramError("unknown grid curve adaptor kind");
}
// g3_grid_adaptor <grid curve adaptor> lower n u*n: the dispatcher on the adaptor (GetType may
// raise), then the fallback on it.
void g3_grid_adaptor_case(std::istream& in){
 G3Reader r{in};occ::handle<Adaptor3d_Curve> a;
 try{a=g3_grid_curve_adaptor(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 NCollection_Array1<double> p=g3_grid_params(r);
 try{g3_grid_curve_dispatch(GeomGridEval_Curve(*a),p);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 g3_grid_curve_all(GeomGridEval_OtherCurve(*a),p);std::cout<<'\n';
}
template<class G,class E> bool g3_grid_surface_direct_as(const occ::handle<Geom_Surface>& s,const NCollection_Array1<double>& u,const NCollection_Array1<double>& v){
 occ::handle<G> g=occ::down_cast<G>(s);if(g.IsNull())return false;E e(g);g3_flag(e.Geometry().IsNull());g3_grid_surface_all(e,u,v,true);return true;
}
// The specialized evaluator of the surface's own class, when it has one.
void g3_grid_surface_direct(const occ::handle<Geom_Surface>& s,const NCollection_Array1<double>& u,const NCollection_Array1<double>& v){
 g3_grid_surface_direct_as<Geom_Plane,GeomGridEval_Plane>(s,u,v)||g3_grid_surface_direct_as<Geom_CylindricalSurface,GeomGridEval_Cylinder>(s,u,v)
 ||g3_grid_surface_direct_as<Geom_SphericalSurface,GeomGridEval_Sphere>(s,u,v)||g3_grid_surface_direct_as<Geom_ConicalSurface,GeomGridEval_Cone>(s,u,v)
 ||g3_grid_surface_direct_as<Geom_ToroidalSurface,GeomGridEval_Torus>(s,u,v)||g3_grid_surface_direct_as<Geom_BezierSurface,GeomGridEval_BezierSurface>(s,u,v)
 ||g3_grid_surface_direct_as<Geom_BSplineSurface,GeomGridEval_BSplineSurface>(s,u,v)
 ||g3_grid_surface_direct_as<Geom_OffsetSurface,GeomGridEval_OffsetSurface>(s,u,v)
 ||g3_grid_surface_direct_as<Geom_SurfaceOfRevolution,GeomGridEval_SurfaceOfRevolution>(s,u,v)
 ||g3_grid_surface_direct_as<Geom_SurfaceOfLinearExtrusion,GeomGridEval_SurfaceOfExtrusion>(s,u,v);
}
// The dispatcher: type, transformation (values and form), then its grids.
void g3_grid_surface_dispatch(const GeomGridEval_Surface& e,const NCollection_Array1<double>& u,const NCollection_Array1<double>& v,bool full){
 g3_int(e.GetType());g3_flag(e.HasTransformation());
 if(e.HasTransformation()){const gp_Trsf& t=*e.GetTransformation();for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)g3_real(t.Value(i,j));g3_int(t.Form());}
 g3_grid_surface_all(e,u,v,full);
}
// g3_grid_surface <surface> U V: the dispatcher on the handle and on a GeomAdaptor_Surface,
// the specialized evaluator, and the fallback on the handle and on the adaptor.
void g3_grid_surface_case(std::istream& in){
 G3Reader r{in};occ::handle<Geom_Surface> s=g3_built_surface(r);if(s.IsNull())return;
 NCollection_Array1<double> u=g3_grid_params(r),v=g3_grid_params(r);
 g3_grid_surface_dispatch(GeomGridEval_Surface(s),u,v,true);
 occ::handle<GeomAdaptor_Surface> a;
 try{a=new GeomAdaptor_Surface(s);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 if(!a.IsNull())G3_TRY(g3_grid_surface_dispatch(GeomGridEval_Surface(*a),u,v,false));
 g3_grid_surface_direct(s,u,v);g3_grid_surface_all(GeomGridEval_OtherSurface(s),u,v,false);
 if(!a.IsNull())g3_grid_surface_all(GeomGridEval_OtherSurface(a.get()),u,v,false);
 std::cout<<'\n';
}
// A grid surface adaptor: 0 <a3 surface adaptor> | 1 D <grid curve adaptor> (extrusion) |
// 2 P D <grid curve adaptor> (revolution) | 3 trsf <surface> (GeomAdaptor_TransformedSurface) |
// 4 u1 u2 v1 v2 tolu tolv trsf <surface> (the ranged transformed adaptor).
occ::handle<Adaptor3d_Surface> g3_grid_surface_adaptor(G3Reader& r){
 int k=r.integer();
 if(k==0)return a3_surface(r);
 if(k==1){gp_Dir d=r.dir();return new GeomAdaptor_SurfaceOfLinearExtrusion(g3_grid_curve_adaptor(r),d);}
 if(k==2){gp_Ax1 ax=r.ax1();return new GeomAdaptor_SurfaceOfRevolution(g3_grid_curve_adaptor(r),ax);}
 if(k==3){gp_Trsf t=g3_trsf(r);return new GeomAdaptor_TransformedSurface(g3_surface(r),t);}
 if(k==4){double b[6];for(double& x:b)x=r.real();gp_Trsf t=g3_trsf(r);
  return new GeomAdaptor_TransformedSurface(g3_surface(r),b[0],b[1],b[2],b[3],t,b[4],b[5]);}
 throw Standard_ProgramError("unknown grid surface adaptor kind");
}
// g3_grid_surface_adaptor <grid surface adaptor> U V: the dispatcher on the adaptor (its
// resolution may raise), then the fallback on it.
void g3_grid_surface_adaptor_case(std::istream& in){
 G3Reader r{in};occ::handle<Adaptor3d_Surface> a;
 try{a=g3_grid_surface_adaptor(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 NCollection_Array1<double> u=g3_grid_params(r),v=g3_grid_params(r);
 try{g3_grid_surface_dispatch(GeomGridEval_Surface(*a),u,v,true);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 g3_grid_surface_all(GeomGridEval_OtherSurface(a.get()),u,v,false);std::cout<<'\n';
}
template<class G,class E> void g3_grid_curve_null_as(const NCollection_Array1<double>& p){E e{occ::handle<G>()};g3_flag(e.Geometry().IsNull());g3_grid_curve_all(e,p);}
template<class G,class E> void g3_grid_surface_null_as(const NCollection_Array1<double>& u,const NCollection_Array1<double>& v){
 E e{occ::handle<G>()};g3_flag(e.Geometry().IsNull());g3_grid_surface_all(e,u,v,true);
}
// g3_grid_null U V: every evaluator on a null geometry, the dispatchers on null handles and
// on empty adaptors (the surface one is not evaluated: its fallback would dereference the
// missing surface).
void g3_grid_null_case(std::istream& in){
 G3Reader r{in};NCollection_Array1<double> u=g3_grid_params(r),v=g3_grid_params(r);
 g3_grid_curve_null_as<Geom_Line,GeomGridEval_Line>(u);g3_grid_curve_null_as<Geom_Circle,GeomGridEval_Circle>(u);
 g3_grid_curve_null_as<Geom_Ellipse,GeomGridEval_Ellipse>(u);g3_grid_curve_null_as<Geom_Hyperbola,GeomGridEval_Hyperbola>(u);
 g3_grid_curve_null_as<Geom_Parabola,GeomGridEval_Parabola>(u);g3_grid_curve_null_as<Geom_BezierCurve,GeomGridEval_BezierCurve>(u);
 g3_grid_curve_null_as<Geom_BSplineCurve,GeomGridEval_BSplineCurve>(u);g3_grid_curve_null_as<Geom_OffsetCurve,GeomGridEval_OffsetCurve>(u);
 g3_grid_curve_dispatch(GeomGridEval_Curve(occ::handle<Geom_Curve>()),u);GeomAdaptor_Curve ac;g3_grid_curve_dispatch(GeomGridEval_Curve(ac),u);
 g3_grid_surface_null_as<Geom_Plane,GeomGridEval_Plane>(u,v);g3_grid_surface_null_as<Geom_CylindricalSurface,GeomGridEval_Cylinder>(u,v);
 g3_grid_surface_null_as<Geom_SphericalSurface,GeomGridEval_Sphere>(u,v);g3_grid_surface_null_as<Geom_ConicalSurface,GeomGridEval_Cone>(u,v);
 g3_grid_surface_null_as<Geom_ToroidalSurface,GeomGridEval_Torus>(u,v);g3_grid_surface_null_as<Geom_BezierSurface,GeomGridEval_BezierSurface>(u,v);
 g3_grid_surface_null_as<Geom_BSplineSurface,GeomGridEval_BSplineSurface>(u,v);g3_grid_surface_null_as<Geom_OffsetSurface,GeomGridEval_OffsetSurface>(u,v);
 g3_grid_surface_null_as<Geom_SurfaceOfRevolution,GeomGridEval_SurfaceOfRevolution>(u,v);
 g3_grid_surface_null_as<Geom_SurfaceOfLinearExtrusion,GeomGridEval_SurfaceOfExtrusion>(u,v);
 g3_grid_surface_dispatch(GeomGridEval_Surface(occ::handle<Geom_Surface>()),u,v,true);
 GeomAdaptor_Surface as;GeomGridEval_Surface e(as);g3_int(e.GetType());g3_flag(e.HasTransformation());std::cout<<'\n';
}
// g3_grid_helpers U V: the GeomGridEval helper templates with fixed functors.
void g3_grid_helpers_case(std::istream& in){
 G3Reader r{in};NCollection_Array1<double> u=g3_grid_params(r),v=g3_grid_params(r);
 g3_grid_array(GeomGridEval::EvaluateGridHelper(u,v,[](double a,double b){return gp_Pnt(a+b,a*b,a-b);}));
 g3_grid_array(GeomGridEval::EvaluateGridD1Helper(u,v,[](double a,double b){
  return GeomGridEval::SurfD1{gp_Pnt(a,b,0.0),gp_Vec(1.0,0.0,b),gp_Vec(0.0,1.0,a)};}));
 g3_grid_array(GeomGridEval::EvaluateGridD2Helper(u,v,[](double a,double b){
  return GeomGridEval::SurfD2{gp_Pnt(a,b,a*b),gp_Vec(1.0,0.0,b),gp_Vec(0.0,1.0,a),gp_Vec(),gp_Vec(),gp_Vec(0.0,0.0,1.0)};}));
 g3_grid_array(GeomGridEval::EvaluateGridD3Helper(u,v,[](double a,double b){
  return GeomGridEval::SurfD3{gp_Pnt(a,b,a*b*b),gp_Vec(1.0,0.0,b*b),gp_Vec(0.0,1.0,2.0*a*b),gp_Vec(),gp_Vec(0.0,0.0,2.0*a),
                              gp_Vec(0.0,0.0,2.0*b),gp_Vec(),gp_Vec(),gp_Vec(),gp_Vec(0.0,0.0,2.0)};}));
 g3_grid_array(GeomGridEval::EvaluateGridDNHelper(u,v,[](double a,double b){return gp_Vec(a*a,b*b,a/(b+3.0));}));
 std::cout<<'\n';
}
