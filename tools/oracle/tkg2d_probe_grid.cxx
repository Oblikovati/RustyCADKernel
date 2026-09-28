// TKG2d oracle: the Geom2dGridEval grid evaluators (tkg2d_probe.cxx).
// Parameters are read as `lower n u*n` into NCollection_Array1(lower, lower + n - 1),
// or the default empty array for n = 0. Every grid prints its bounds, then its values;
// a raised exception prints its type instead.
NCollection_Array1<double> g2_grid_params(G2Reader& r){
 int lower=r.integer();std::vector<double> u=r.reals();if(u.empty())return NCollection_Array1<double>();
 NCollection_Array1<double> p(lower,lower+int(u.size())-1);for(size_t i=0;i<u.size();++i)p(lower+int(i))=u[i];return p;
}
void g2_grid_value(const gp_Pnt2d& p){g2_pnt(p);}
void g2_grid_value(const gp_Vec2d& v){g2_vec(v);}
void g2_grid_value(const Geom2dGridEval::CurveD1& r){g2_pnt(r.Point);g2_vec(r.D1);}
void g2_grid_value(const Geom2dGridEval::CurveD2& r){g2_pnt(r.Point);g2_vec(r.D1);g2_vec(r.D2);}
void g2_grid_value(const Geom2dGridEval::CurveD3& r){g2_pnt(r.Point);g2_vec(r.D1);g2_vec(r.D2);g2_vec(r.D3);}
template<class T> void g2_grid_array(const NCollection_Array1<T>& a){g2_int(a.Lower());g2_int(a.Length());for(const T& x:a)g2_grid_value(x);}
// Every member of one evaluator: EvaluateGrid, D1, D2, D3, then DN for N = -1..7.
template<class E> void g2_grid_all(const E& e,const NCollection_Array1<double>& p){
 G2_TRY(g2_grid_array(e.EvaluateGrid(p)));G2_TRY(g2_grid_array(e.EvaluateGridD1(p)));
 G2_TRY(g2_grid_array(e.EvaluateGridD2(p)));G2_TRY(g2_grid_array(e.EvaluateGridD3(p)));
 for(int n=-1;n<=7;++n)G2_TRY(g2_grid_array(e.EvaluateGridDN(p,n)));
}
template<class G,class E> bool g2_grid_direct_as(const occ::handle<Geom2d_Curve>& c,const NCollection_Array1<double>& p){
 occ::handle<G> g=occ::down_cast<G>(c);if(g.IsNull())return false;E e(g);g2_flag(e.Geometry().IsNull());g2_grid_all(e,p);return true;
}
// The specialized evaluator of the curve's own class, when it has one.
void g2_grid_direct(const occ::handle<Geom2d_Curve>& c,const NCollection_Array1<double>& p){
 g2_grid_direct_as<Geom2d_Line,Geom2dGridEval_Line>(c,p)||g2_grid_direct_as<Geom2d_Circle,Geom2dGridEval_Circle>(c,p)
 ||g2_grid_direct_as<Geom2d_Ellipse,Geom2dGridEval_Ellipse>(c,p)||g2_grid_direct_as<Geom2d_Hyperbola,Geom2dGridEval_Hyperbola>(c,p)
 ||g2_grid_direct_as<Geom2d_Parabola,Geom2dGridEval_Parabola>(c,p)||g2_grid_direct_as<Geom2d_BezierCurve,Geom2dGridEval_BezierCurve>(c,p)
 ||g2_grid_direct_as<Geom2d_BSplineCurve,Geom2dGridEval_BSplineCurve>(c,p)||g2_grid_direct_as<Geom2d_OffsetCurve,Geom2dGridEval_OffsetCurve>(c,p);
}
void g2_grid_dispatch(const Geom2dGridEval_Curve& e,const NCollection_Array1<double>& p){g2_int(e.GetType());g2_grid_all(e,p);}
// g2_grid <curve> lower n u*n: the dispatcher on the handle and on a Geom2dAdaptor_Curve,
// the specialized evaluator, and the adaptor fallback on the same adaptor.
void g2_grid_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c=g2_built(r);if(c.IsNull())return;NCollection_Array1<double> p=g2_grid_params(r);
 g2_grid_dispatch(Geom2dGridEval_Curve(c),p);Geom2dAdaptor_Curve a(c);g2_grid_dispatch(Geom2dGridEval_Curve(a),p);
 g2_grid_direct(c,p);g2_grid_all(Geom2dGridEval_OtherCurve(a),p);std::cout<<'\n';
}
// g2_grid_adaptor <adaptor> lower n u*n: the dispatcher on any adaptor (GetType may raise),
// then the fallback on it.
void g2_grid_adaptor_case(std::istream& in){
 G2Reader r{in};occ::handle<Adaptor2d_Curve2d> a=a2_built(r);if(a.IsNull())return;NCollection_Array1<double> p=g2_grid_params(r);
 try{g2_grid_dispatch(Geom2dGridEval_Curve(*a),p);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 g2_grid_all(Geom2dGridEval_OtherCurve(*a),p);std::cout<<'\n';
}
// g2_grid_evalrep <curve> <rep curve> <desc> lower n u*n: grids of a curve that carries an
// evaluation representation, through the handle and through an adaptor.
void g2_grid_evalrep_case(std::istream& in){
 G2Reader r{in};occ::handle<Geom2d_Curve> c=g2_built(r);if(c.IsNull())return;occ::handle<Geom2d_Curve> rep=g2_curve(r);
 occ::handle<Geom2dEval_RepCurveDesc::Base> d=g2_desc(r,rep);NCollection_Array1<double> p=g2_grid_params(r);
 G2_TRY(a2_set_rep(c,d));g2_grid_dispatch(Geom2dGridEval_Curve(c),p);Geom2dAdaptor_Curve a(c);
 g2_grid_dispatch(Geom2dGridEval_Curve(a),p);std::cout<<'\n';
}
template<class G,class E> void g2_grid_null_as(const NCollection_Array1<double>& p){E e{occ::handle<G>()};g2_flag(e.Geometry().IsNull());g2_grid_all(e,p);}
// g2_grid_null lower n u*n: every evaluator on a null curve, and the dispatcher on a null
// handle and on an empty Geom2dAdaptor_Curve.
void g2_grid_null_case(std::istream& in){
 G2Reader r{in};NCollection_Array1<double> p=g2_grid_params(r);
 g2_grid_null_as<Geom2d_Line,Geom2dGridEval_Line>(p);g2_grid_null_as<Geom2d_Circle,Geom2dGridEval_Circle>(p);
 g2_grid_null_as<Geom2d_Ellipse,Geom2dGridEval_Ellipse>(p);g2_grid_null_as<Geom2d_Hyperbola,Geom2dGridEval_Hyperbola>(p);
 g2_grid_null_as<Geom2d_Parabola,Geom2dGridEval_Parabola>(p);g2_grid_null_as<Geom2d_BezierCurve,Geom2dGridEval_BezierCurve>(p);
 g2_grid_null_as<Geom2d_BSplineCurve,Geom2dGridEval_BSplineCurve>(p);g2_grid_null_as<Geom2d_OffsetCurve,Geom2dGridEval_OffsetCurve>(p);
 g2_grid_dispatch(Geom2dGridEval_Curve(occ::handle<Geom2d_Curve>()),p);Geom2dAdaptor_Curve a;g2_grid_dispatch(Geom2dGridEval_Curve(a),p);
 std::cout<<'\n';
}
