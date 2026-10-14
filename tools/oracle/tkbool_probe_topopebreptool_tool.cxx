// TopOpeBRepTool_TOOL, TOPOLOGY, makeTransition and mkTondgE probe operations on the
// edges and faces of primitive solids; the Rust twin is
// crates/tkbool/examples/bl_probe/top_ope_brep_tool_tool.rs.
//
// A solid record is `kind a b c d tx ty tz`: 0 box (a, b, c), 1 cylinder (a, b), 2 cone
// (a, b, c), 3 sphere (a), 4 torus (a, b), 5 the L-shaped prism (legs a and b, thickness
// d, height c), each translated by (tx, ty, tz). States and orientations print as their
// TopAbs values.
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <TopOpeBRepTool_C2DF.hxx>
#include <TopOpeBRepTool_PROJECT.hxx>
#include <TopOpeBRepTool_TOOL.hxx>
#include <TopOpeBRepTool_TOPOLOGY.hxx>
#include <TopOpeBRepTool_makeTransition.hxx>
#include <TopOpeBRepTool_mkTondgE.hxx>
#include <gp_Trsf.hxx>
Standard_EXPORT bool FUN_tool_ClosedW(const TopoDS_Wire& W);

TopoDS_Shape bl_tt_solid(GaReader& r){
 int kind=r.integer();double a=r.real(),b=r.real(),c=r.real(),d=r.real();gp_Vec t(r.xyz());
 TopoDS_Shape s;
 if(kind==0)s=BRepPrimAPI_MakeBox(a,b,c).Shape();
 else if(kind==1)s=BRepPrimAPI_MakeCylinder(a,b).Shape();
 else if(kind==2)s=BRepPrimAPI_MakeCone(a,b,c).Shape();
 else if(kind==3)s=BRepPrimAPI_MakeSphere(a).Shape();
 else if(kind==4)s=BRepPrimAPI_MakeTorus(a,b).Shape();
 else{BRepBuilderAPI_MakePolygon poly;double xy[6][2]={{0,0},{a,0},{a,d},{d,d},{d,b},{0,b}};
  for(auto& p:xy)poly.Add(gp_Pnt(p[0],p[1],0));poly.Close();
  s=BRepPrimAPI_MakePrism(BRepBuilderAPI_MakeFace(poly.Wire()).Face(),gp_Vec(0,0,c)).Shape();}
 gp_Trsf tr;tr.SetTranslation(t);return s.Moved(TopLoc_Location(tr));}

// The edge `ie` of the edge/face map of a solid, its face `k` (1 or 2, the last one when
// it has a single face) and the other face, the parameter at `t` of its range and the
// UV of that point on the face (0, 0 without p-curve).
struct BlTtEdgeFace{TopoDS_Shape s;TopoDS_Edge e;TopoDS_Face f,f2;double par;gp_Pnt2d uv;};
BlTtEdgeFace bl_tt_edge_face(GaReader& r){
 BlTtEdgeFace x;x.s=bl_tt_solid(r);int ie=r.integer(),k=r.integer();double t=r.real();
 NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> m;
 TopExp::MapShapesAndAncestors(x.s,TopAbs_EDGE,TopAbs_FACE,m);
 x.e=TopoDS::Edge(m.FindKey(ie));const NCollection_List<TopoDS_Shape>& l=m(ie);
 x.f=TopoDS::Face(k==1?l.First():l.Last());x.f2=TopoDS::Face(k==1?l.Last():l.First());
 double a,b;BRep_Tool::Range(x.e,a,b);x.par=a+t*(b-a);
 double pf,pl;occ::handle<Geom2d_Curve> pc=BRep_Tool::CurveOnSurface(x.e,x.f,pf,pl);
 x.uv=pc.IsNull()?gp_Pnt2d(0,0):pc->Value(x.par);return x;}

// The first edge of the face other than `e` (`e` itself when there is none).
TopoDS_Edge bl_tt_other_edge(const TopoDS_Face& f,const TopoDS_Edge& e){
 for(TopExp_Explorer ex(f,TopAbs_EDGE);ex.More();ex.Next())if(!ex.Current().IsSame(e))return TopoDS::Edge(ex.Current());
 return e;}

// `tt_tool solid ie k t`: the TOPOLOGY-free TOOL members on the edge and the face.
void bl_tt_tool_case(std::istream& input){
 GaReader r{input};BlTtEdgeFace x;
 try{x=bl_tt_edge_face(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Edge& e=x.e;const TopoDS_Face& f=x.f;double par=x.par;gp_Pnt2d uv=x.uv;
 GA_TRY(ga_int(TopOpeBRepTool_TOOL::OriinSor(e,f,true));ga_int(TopOpeBRepTool_TOOL::OriinSor(e,f,false));ga_int(TopOpeBRepTool_TOOL::OriinSorclosed(e,f)))
 GA_TRY(TopoDS_Vertex vc;ga_flag(TopOpeBRepTool_TOOL::ClosedE(e,vc));ga_flag(TopOpeBRepTool_TOOL::ClosedS(f));
  ga_flag(TopOpeBRepTool_TOOL::IsClosingE(e,f));ga_flag(TopOpeBRepTool_TOOL::IsClosingE(e,f,f)))
 GA_TRY(ga_real(TopOpeBRepTool_TOOL::ParE(1,e));ga_real(TopOpeBRepTool_TOOL::ParE(2,e));
  ga_int(TopOpeBRepTool_TOOL::OriinSor(TopOpeBRepTool_TOOL::Vertex(1,e),e,true)))
 GA_TRY(ga_int(TopOpeBRepTool_TOOL::OnBoundary(par,e));ga_int(TopOpeBRepTool_TOOL::OnBoundary(TopOpeBRepTool_TOOL::ParE(1,e),e)))
 GA_TRY(gp_Vec tg(0,0,0);int ov=-1;ga_flag(TopOpeBRepTool_TOOL::TgINSIDE(TopOpeBRepTool_TOOL::Vertex(1,e),e,tg,ov));ga_xyz(tg.XYZ());ga_int(ov))
 GA_TRY(ga_int(TopOpeBRepTool_TOOL::tryOriEinF(par,e,f)))
 GA_TRY(gp_Dir n(gp_Dir::D::Z);ga_flag(TopOpeBRepTool_TOOL::NgApp(par,e,f,1.e-6,n));ga_dir(n))
 GA_TRY(gp_Dir n(gp_Dir::D::Z);ga_flag(TopOpeBRepTool_TOOL::tryNgApp(par,e,f,1.e-6,n));ga_dir(n))
 GA_TRY(ga_flag(TopOpeBRepTool_TOOL::IsQuad(e));ga_flag(TopOpeBRepTool_TOOL::IsQuad(f)))
 GA_TRY(gp_Dir n(gp_Dir::D::Z);ga_flag(TopOpeBRepTool_TOOL::Nt(uv,f,n));ga_dir(n);
  double cv=-1;ga_flag(TopOpeBRepTool_TOOL::CurvE(e,par,n,cv));ga_real(cv))
 GA_TRY(gp_Vec tg;ga_flag(TopOpeBRepTool_TOOL::TggeomE(par,e,tg));gp_Dir d(tg);double cv=-1;bool direct=false;
  ga_flag(TopOpeBRepTool_TOOL::CurvF(f,uv,d,cv,direct));ga_real(cv);ga_flag(direct))
 GA_TRY(gp_Dir xx(gp_Dir::D::Z);ga_flag(TopOpeBRepTool_TOOL::XX(uv,f,par,e,xx));ga_dir(xx))
 GA_TRY(double ang=-1;ga_flag(TopOpeBRepTool_TOOL::Matter(f,x.f2,e,par,1.e-6,ang));ga_real(ang))
 GA_TRY(double ang=-1;ga_flag(TopOpeBRepTool_TOOL::MatterKPtg(f,x.f2,e,ang));ga_real(ang))
 GA_TRY(gp_Pnt p=BRepAdaptor_Curve(e).Value(par).Translated(gp_Vec(0.1,0.2,0.3));gp_Pnt2d puv(-1,-1);TopAbs_State st=TopAbs_UNKNOWN;
  ga_flag(TopOpeBRepTool_TOOL::Getstp3dF(p,f,puv,st));ga_pnt2d(puv);ga_int(st))
 GA_TRY(int ou=-9,ov=-9;TopOpeBRepTool_TOOL::stuvF(uv,f,ou,ov);ga_int(ou);ga_int(ov);ga_flag(TopOpeBRepTool_TOOL::outUVbounds(uv,f));
  ga_real(TopOpeBRepTool_TOOL::TolUV(f,1.e-7));ga_real(TopOpeBRepTool_TOOL::TolP(e,f)))
 GA_TRY(bool on=false;ga_flag(TopOpeBRepTool_TOOL::EdgeONFace(par,e,uv,f,on));ga_flag(on))
 GA_TRY(gp_Pnt2d a(-1,-1);ga_flag(TopOpeBRepTool_TOOL::uvApp(f,e,par,0.1,a));ga_pnt2d(a))
 GA_TRY(gp_Vec tg;TopOpeBRepTool_TOOL::TggeomE(par,e,tg);gp_Dir2d duv(gp_Dir2d::D::X);ga_flag(TopOpeBRepTool_TOOL::Getduv(f,uv,tg,0.1,duv));ga_xy(duv.XY()))
 GA_TRY(double p=-1;ga_flag(TopOpeBRepTool_TOOL::ParISO(uv,e,f,p));ga_real(p))
 GA_TRY(double p=-1,d=-1;ga_flag(TopOpeBRepTool_TOOL::ParE2d(uv,e,f,p,d));ga_real(p);ga_real(d))
 GA_TRY(double pf,pl;occ::handle<Geom2d_Curve> pc=BRep_Tool::CurveOnSurface(e,f,pf,pl);TopOpeBRepTool_C2DF c(pc,pf,pl,1.e-7,f);
  ga_xy(TopOpeBRepTool_TOOL::Tg2d(1,e,c).XY());ga_xy(TopOpeBRepTool_TOOL::Tg2dApp(1,e,c,0.2).XY());
  ga_xy(TopOpeBRepTool_TOOL::tryTg2dApp(2,e,c,0.2).XY());ga_flag(TopOpeBRepTool_TOOL::IsonCLO(c,true,0.,2.*M_PI,1.e-7));
  ga_pnt2d(TopOpeBRepTool_TOOL::UVF(par,c)))
 GA_TRY(NCollection_List<TopoDS_Shape> l;ga_flag(TopOpeBRepTool_TOOL::SplitE(e,l));ga_int(l.Extent()))
 GA_TRY(gp_Vec tg;TopOpeBRepTool_TOOL::TggeomE(par,e,tg);gp_Dir n(gp_Dir::D::Z);TopOpeBRepTool_TOOL::Nt(uv,f,n);
  ga_real(TopOpeBRepTool_TOOL::Matter(tg,gp_Vec(n),gp_Vec(n).Crossed(tg)));
  ga_real(TopOpeBRepTool_TOOL::Matter(gp_Vec2d(1,0),gp_Vec2d(std::cos(par),std::sin(par)))))
 ga_end();}

// `tt_topology solid ie k t`: the TOPOLOGY functions on the edge and the face.
void bl_tt_topology_case(std::istream& input){
 GaReader r{input};BlTtEdgeFace x;
 try{x=bl_tt_edge_face(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Edge& e=x.e;const TopoDS_Face& f=x.f;double par=x.par;gp_Pnt2d uv=x.uv;const TopoDS_Edge e2=bl_tt_other_edge(f,e);
 const TopoDS_Vertex v1=TopExp::FirstVertex(e);
 GA_TRY(double a=-1,b=-1;FUN_tool_tolUV(f,a,b);ga_real(a);ga_real(b))
 GA_TRY(bool d=false;ga_flag(FUN_tool_direct(f,d));ga_flag(d))
 GA_TRY(double a=-1,b=-1,c=-1,d=-1;ga_flag(FUN_tool_bounds(f,a,b,c,d));ga_real(a);ga_real(b);ga_real(c);ga_real(d))
 GA_TRY(double a=-1,b=-1,c=-1,d=-1;ga_flag(FUN_tool_geombounds(f,a,b,c,d));ga_real(a);ga_real(b);ga_real(c);ga_real(d))
 GA_TRY(double a=-1,b=-1,c=-1,d=-1;ga_flag(FUN_tool_isobounds(f,a,b,c,d));ga_real(a);ga_real(b);ga_real(c);ga_real(d))
 GA_TRY(double a=-1,b=-1,c=-1,d=-1;bool o=false;ga_flag(FUN_tool_outbounds(f,a,b,c,d,o));ga_real(a);ga_real(b);ga_real(c);ga_real(d);ga_flag(o))
 GA_TRY(BRepAdaptor_Curve bc(e);gp_Pnt p=bc.Value(par);ga_flag(FUN_tool_PinC(p,bc,1.e-7));
  ga_flag(FUN_tool_PinC(p.Translated(gp_Vec(0,0,1.e-3)),bc,bc.FirstParameter(),bc.LastParameter(),1.e-2)))
 GA_TRY(gp_Pnt p(-1,-1,-1);ga_flag(FUN_tool_value(par,e,p));ga_pnt(p);FUN_tool_value(uv,f,p);ga_pnt(p))
 GA_TRY(gp_Pnt p=BRepAdaptor_Curve(e).Value(par);ga_int(FUN_tool_staPinE(p,e));ga_int(FUN_tool_staPinE(p.Translated(gp_Vec(0,0,0.5)),e,1.e-3)))
 GA_TRY(ga_int(FUN_tool_orientVinE(v1,e)))
 GA_TRY(TopAbs_Orientation o=TopAbs_INTERNAL;ga_flag(FUN_tool_orientEinF(e,f,o));ga_int(o);
  ga_flag(FUN_tool_orientEinFFORWARD(e,TopoDS::Face(f.Reversed()),o));ga_int(o);ga_flag(FUN_tool_EboundF(e,f)))
 GA_TRY(ga_xyz(FUN_tool_nggeomF(uv,f).XYZ()))
 GA_TRY(gp_Vec n(0,0,0);ga_flag(FUN_tool_nggeomF(par,e,f,n));ga_xyz(n.XYZ());ga_flag(FUN_tool_nggeomF(par,e2,f,n,1.e-9));ga_xyz(n.XYZ()))
 GA_TRY(ga_flag(FUN_tool_EtgF(par,e,uv,f,1.e-6));ga_flag(FUN_tool_EtgOOE(par,e,par,e2,1.e-6)))
 GA_TRY(double a=-10,b=10;ga_flag(FUN_nearestISO(f,uv.X(),true,a,b));ga_real(a);ga_real(b))
 GA_TRY(gp_Dir n(gp_Dir::D::Z);TopOpeBRepTool_TOOL::Nt(uv,f,n);ga_flag(FUN_tool_EitangenttoFe(n,e,par)))
 GA_TRY(ga_int(FUN_tool_typ(e));ga_flag(FUN_tool_line(e));ga_flag(FUN_tool_plane(f));ga_flag(FUN_tool_cylinder(f)))
 GA_TRY(ga_flag(FUN_tool_closedS(f));bool uc=false,vc=false;double up=-1,vp=-1;ga_flag(FUN_tool_closedS(f,uc,up,vc,vp));
  ga_flag(uc);ga_real(up);ga_flag(vc);ga_real(vp);bool inu=false;double xm=-1,xp=-1;ga_flag(FUN_tool_closedS(f,inu,xm,xp));
  ga_flag(inu);ga_real(xm);ga_real(xp))
 GA_TRY(TopExp_Explorer ex(f,TopAbs_WIRE);Bnd_Box2d b;FUN_tool_mkBnd2d(ex.Current(),f,b);double a,c,d,g;b.Get(a,c,d,g);
  ga_real(a);ga_real(c);ga_real(d);ga_real(g);ga_flag(FUN_tool_ClosingE(e,TopoDS::Wire(ex.Current()),f));
  ga_flag(FUN_tool_ClosedW(TopoDS::Wire(ex.Current()))))
 GA_TRY(ga_flag(FUN_tool_IsClosingE(e,f,f));ga_flag(FUN_tool_inS(e,f));ga_flag(FUN_tool_inS(e,x.f2)))
 GA_TRY(TopoDS_Shape es;ga_flag(FUN_tool_Eshared(v1,f,x.f2,es));ga_flag(es.IsSame(e)))
 GA_TRY(double p=-1;ga_flag(FUN_tool_parVonE(v1,e,p));ga_real(p);p=-1;ga_flag(FUN_tool_parE(e,par,e2,p));ga_real(p))
 GA_TRY(gp_Pnt2d a(-1,-1);ga_flag(FUN_tool_parF(e,par,f,a));ga_pnt2d(a);ga_flag(FUN_tool_paronEF(e,par,x.f2,a));ga_pnt2d(a))
 GA_TRY(BRepAdaptor_Curve bc(e);ga_dir(FUN_tool_dirC(par,bc));ga_xyz(FUN_tool_tggeomE(par,e).XYZ());
  ga_flag(FUN_tool_line(bc));ga_flag(FUN_tool_quad(e));ga_flag(FUN_tool_quad(bc));ga_flag(FUN_tool_quad(f)))
 GA_TRY(gp_Pnt p;double q=-1;FUN_tool_findPinE(e,p,q);ga_pnt(p);ga_real(q))
 GA_TRY(ga_real(FUN_tool_maxtol(x.s));double t=0;ga_flag(FUN_tool_maxtol(f,TopAbs_EDGE,t));ga_real(t))
 GA_TRY(ga_int(FUN_tool_nbshapes(x.s,TopAbs_EDGE));NCollection_List<TopoDS_Shape> l;FUN_tool_shapes(f,TopAbs_EDGE,l);ga_int(l.Extent()))
 GA_TRY(ga_int(FUN_tool_comparebndkole(f,x.s));ga_int(FUN_tool_comparebndkole(e,f)))
 GA_TRY(ga_flag(FUN_tool_SameOri(e,e));ga_flag(FUN_tool_SameOri(e,TopoDS::Edge(e.Reversed())));ga_flag(FUN_tool_haspc(e,f)))
 GA_TRY(bool so=false;ga_flag(FUN_tool_curvesSO(e,e2,so));ga_flag(so);so=false;ga_flag(FUN_tool_curvesSO(e,par,e,so));ga_flag(so))
 GA_TRY(NCollection_List<TopoDS_Shape> l;l.Append(x.f2);l.Append(f);TopoDS_Face fa;ga_flag(FUN_tool_findAncestor(l,e,fa));ga_flag(fa.IsSame(x.f2)))
 GA_TRY(ga_xyz(FUN_tool_getgeomxx(f,e,par).XYZ());gp_Dir d(gp_Dir::D::Z);ga_flag(FUN_tool_getxx(f,e,par,d));ga_dir(d))
 GA_TRY(gp_Vec2d d(-1,-1);ga_flag(FUN_tool_getdxx(f,e,par,d));ga_xy(d.XY()))
 ga_end();}

// `tt_make_transition solid1 solid2 ie fi parE factor ies parES`: the transition of the
// edge `ie` (MapShapes order) of the first solid crossing the face `fi` of the second at
// its parameter, restricted by the edge `ies` of the second solid (none for 0).
void bl_tt_make_transition_case(std::istream& input){
 GaReader r{input};TopoDS_Shape s1,s2;
 try{s1=bl_tt_solid(r);s2=bl_tt_solid(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int ie=r.integer(),fi=r.integer();double pe=r.real(),fac=r.real();int ies=r.integer();double pes=r.real();
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> me1,mf2,me2;
 TopExp::MapShapes(s1,TopAbs_EDGE,me1);TopExp::MapShapes(s2,TopAbs_FACE,mf2);TopExp::MapShapes(s2,TopAbs_EDGE,me2);
 const TopoDS_Edge e=TopoDS::Edge(me1(ie));const TopoDS_Face fs=TopoDS::Face(mf2(fi));
 TopOpeBRepTool_makeTransition mt;
 GA_TRY(gp_Pnt2d uv(0,0);double d;FUN_tool_projPonF(BRepAdaptor_Curve(e).Value(pe),fs,uv,d);double a,b;BRep_Tool::Range(e,a,b);
  ga_flag(mt.Initialize(e,a,b,pe,fs,uv,fac));ga_flag(mt.IsT2d()))
 if(ies>0)GA_TRY(ga_flag(mt.SetRest(TopoDS::Edge(me2(ies)),pes));ga_flag(mt.HasRest()))
 GA_TRY(TopAbs_State b=TopAbs_UNKNOWN,a=TopAbs_UNKNOWN;ga_flag(mt.MkT2donE(b,a));ga_int(b);ga_int(a))
 GA_TRY(TopAbs_State b=TopAbs_UNKNOWN,a=TopAbs_UNKNOWN;ga_flag(mt.MkT3onE(b,a));ga_int(b);ga_int(a))
 GA_TRY(TopAbs_State b=TopAbs_UNKNOWN,a=TopAbs_UNKNOWN;ga_flag(mt.MkT3dproj(b,a));ga_int(b);ga_int(a))
 GA_TRY(TopAbs_State b=TopAbs_UNKNOWN,a=TopAbs_UNKNOWN;ga_flag(mt.MkTonE(b,a));ga_int(b);ga_int(a);ga_real(mt.Getfactor()))
 ga_end();}

// `tt_mktondge solid1 solid2 ie fi`: the transitions on the degenerated edge `ie`
// (MapShapes order) of the first solid (on its first face) crossed by the face `fi` of
// the second at the projection of the edge vertex, then on each restriction edge.
void bl_tt_mktondge_case(std::istream& input){
 GaReader r{input};TopoDS_Shape s1,s2;
 try{s1=bl_tt_solid(r);s2=bl_tt_solid(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int ie=r.integer(),fi=r.integer();
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> mf2;TopExp::MapShapes(s2,TopAbs_FACE,mf2);
 NCollection_IndexedDataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> m;
 TopExp::MapShapesAndAncestors(s1,TopAbs_EDGE,TopAbs_FACE,m);
 const TopoDS_Edge dge=TopoDS::Edge(m.FindKey(ie));const TopoDS_Face f=TopoDS::Face(m(ie).First());
 const TopoDS_Face fi_=TopoDS::Face(mf2(fi));
 TopOpeBRepTool_mkTondgE mk;
 GA_TRY(gp_Pnt2d uv(0,0);double d;FUN_tool_projPonF(BRep_Tool::Pnt(TopExp::FirstVertex(dge)),fi_,uv,d);
  ga_flag(mk.Initialize(dge,f,uv,fi_));ga_flag(mk.IsT2d()))
 GA_TRY(int k=-1;double p1=-1,p2=-1;ga_flag(mk.MkTonE(k,p1,p2));ga_int(k);ga_real(p1);ga_real(p2))
 GA_TRY(NCollection_List<TopoDS_Shape> l;ga_int(mk.GetAllRest(l));
  for(NCollection_List<TopoDS_Shape>::Iterator it(l);it.More();it.Next()){
   GA_TRY(int k=-1;double p1=-1,p2=-1;ga_flag(mk.MkTonE(TopoDS::Edge(it.Value()),k,p1,p2));ga_int(k);ga_real(p1);ga_real(p2))})
 ga_end();}
