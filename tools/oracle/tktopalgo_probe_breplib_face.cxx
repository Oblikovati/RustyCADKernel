// BRepLib_MakeFace, BRepLib_FindSurface, BRepLib::SameParameter and BRepLib::ExtendFace
// probe operations and the shape records 5 to 8; the Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_lib_face.rs.
//
// A surface2 record is `kind ...`: 0 a surface record of the face reader; 1 an offset
// surface `value (surface2)`; 2 a linear extrusion `D (Geom curve)`; 3 a revolution
// `P D (Geom curve)`.
//
// Shape records (continuing tktopalgo_probe_breplib_shapes.cxx):
// 5 a BRepLib_MakePolygon wire `n (x y z)... closed` (a null shape when not done);
// 6 the first wire of the BRepLib_MakeFace face of a surface2 record over `u1 u2 v1 v2`
//   (at Precision::Confusion()), reversed when `reversed`;
// 7 an edge `curve u1 u2 pcurve p1 p2 surface tol sameparameter samerange`: the 3d curve
//   on [u1, u2] (3d only), the p-curve on the surface on [p1, p2], vertices at the curve
//   ends, then the two flags;
// 8 `record build unflag`: BuildCurves3d on the shape when `build`, then SameParameter
//   and SameRange cleared on every edge when `unflag`;
// 9 the BRepLib_MakeFace face of a surface2 record over `u1 u2 v1 v2` at `toldegen`;
// 10 and 11 are read by tp_shape_record_fuse (tktopalgo_probe_breplib_fuse.cxx).
//
// `blm_face mode ...`: 0 `surface` the gp constructor; 1 `surface u1 u2 v1 v2` the gp
//   constructor on bounds; 2 `surface toldegen`; 3 `surface u1 u2 v1 v2 toldegen`;
//   4 `surface wire inside` the gp constructor on a wire; 5 `surface wire inside`;
//   6 `wire onlyplane`; 7 `face`; 8 `face wire`; 9 `surface bound toldegen n (wire)...`
//   the default constructor, Init then the Adds; 10 `surface u1 u2 v1 v2 toldegen` twice,
//   two bounded Inits on one maker. Surfaces are surface2 records, wires and faces
//   shape records; the gp
//   constructors need an elementary surface (-2 otherwise). Prints IsDone, Error and
//   the digest of Face().
// `blm_facedegen curve maxtol`: IsDegenerated and the actual tolerance.
// `blp_findsurface shape tol onlyplane onlyclosed`: Found, Existed, Tolerance,
//   ToleranceReached, the location identity flag, the surface type and points at
//   (0,0), (1,0) and (0,1).
// `blp_sameparam shape mode tol forced`: 0 SameParameter(S, tol, forced); 1 the
//   re-shaper overload (the digest of S, then of the re-shaper Apply(S)); 2
//   SameParameter(E, tol) per edge (MapShapes order); 3 per edge SameParameter(E, tol,
//   newtol, false) with newtol and the edge digest; 4 the same with IsUseOldEdge.
//   Ends with the digest of S.
// `blp_extendface shape ext umin umax vmin vmax`: ExtendFace per face (MapShapes order).
#include <BRepLib_FindSurface.hxx>
#include <BRepLib_MakeFace.hxx>
#include <BRepLib_MakePolygon.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
occ::handle<Geom_Surface> tp_surface2(GaReader& r){
 int kind=r.integer();
 if(kind==0)return tp_surface(r);
 if(kind==1){double v=r.real();return new Geom_OffsetSurface(tp_surface2(r),v);}
 if(kind==2){gp_Dir d=r.dir();return new Geom_SurfaceOfLinearExtrusion(tp_geom_curve(r),d);}
 gp_Pnt p=r.pnt();gp_Dir d=r.dir();return new Geom_SurfaceOfRevolution(tp_geom_curve(r),gp_Ax1(p,d));}
TopoDS_Shape tp_shape_record_fuse(GaReader& r,int kind);
TopoDS_Shape tp_shape_record_more(GaReader& r,int kind){
 BRep_Builder B;
 if(kind>=10)return tp_shape_record_fuse(r,kind);
 if(kind==5){int n=r.integer();BRepLib_MakePolygon m;for(int i=0;i<n;++i)m.Add(r.pnt());if(r.flag())m.Close();
  if(!m.IsDone())return TopoDS_Shape();return m.Wire();}
 if(kind==9){occ::handle<Geom_Surface> s=tp_surface2(r);double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
  BRepLib_MakeFace m(s,u1,u2,v1,v2,r.real());if(!m.IsDone())return TopoDS_Shape();return m.Face();}
 if(kind==6){occ::handle<Geom_Surface> s=tp_surface2(r);double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();bool rev=r.flag();
  BRepLib_MakeFace m(s,u1,u2,v1,v2,Precision::Confusion());TopoDS_Iterator it(m.Face());if(!it.More())return TopoDS_Shape();
  return rev?it.Value().Reversed():it.Value();}
 if(kind==7){occ::handle<Geom_Curve> c=tp_geom_curve(r);double u1=r.real(),u2=r.real();occ::handle<Geom2d_Curve> pc=tp_geom2d_curve(r);
  double p1=r.real(),p2=r.real();occ::handle<Geom_Surface> s=tp_surface(r);double tol=r.real();bool sp=r.flag(),sr=r.flag();
  TopoDS_Edge e;B.MakeEdge(e,c,tol);B.UpdateEdge(e,pc,s,TopLoc_Location(),tol);B.Range(e,u1,u2,true);B.Range(e,s,TopLoc_Location(),p1,p2);
  TopoDS_Vertex v1,v2;B.MakeVertex(v1,c->Value(u1),tol);B.MakeVertex(v2,c->Value(u2),tol);
  B.Add(e,v1.Oriented(TopAbs_FORWARD));B.Add(e,v2.Oriented(TopAbs_REVERSED));B.SameParameter(e,sp);B.SameRange(e,sr);return e;}
 TopoDS_Shape sh=tp_shape_record(r);bool build=r.flag(),unflag=r.flag();
 if(build)BRepLib::BuildCurves3d(sh);
 if(unflag)for(TopExp_Explorer x(sh,TopAbs_EDGE);x.More();x.Next()){B.SameParameter(TopoDS::Edge(x.Current()),false);B.SameRange(TopoDS::Edge(x.Current()),false);}
 return sh;}
// The gp constructors of MakeFace on an elementary surface; false for another one. The
// face requests are templates over the maker, so that the BRepBuilderAPI probe runs the
// same calls on BRepBuilderAPI_MakeFace.
template<class M> bool tp_gp_face(const occ::handle<Geom_Surface>& s,int mode,GaReader& r,M& m){
 double b[4]={0,0,0,0};TopoDS_Wire w;bool inside=true;
 if(mode==1)for(double& x:b)x=r.real();
 if(mode==4){w=TopoDS::Wire(tp_shape_record(r));inside=r.flag();}
 auto make=[&](const auto& gp){if(mode==0)m=M(gp);else if(mode==1)m=M(gp,b[0],b[1],b[2],b[3]);else m=M(gp,w,inside);};
 if(auto p=occ::down_cast<Geom_Plane>(s))make(p->Pln());
 else if(auto c=occ::down_cast<Geom_CylindricalSurface>(s))make(c->Cylinder());
 else if(auto k=occ::down_cast<Geom_ConicalSurface>(s))make(k->Cone());
 else if(auto sp=occ::down_cast<Geom_SphericalSurface>(s))make(sp->Sphere());
 else if(auto t=occ::down_cast<Geom_ToroidalSurface>(s))make(t->Torus());
 else return false;
 return true;}
void tp_face_maker(const BRepLib_MakeFace& m){ga_flag(m.IsDone());ga_int(m.Error());GA_TRY(tp_shape(m.Face()))}
// The maker of a `blm_face` request (`print` shows the first maker of mode 10); false for
// a gp mode on a surface that is not elementary.
template<class M,class P> bool tp_face_request(GaReader& r,M& m,P print){
 int mode=r.integer();
 if(mode==0||mode==1||mode==4){occ::handle<Geom_Surface> s=tp_surface2(r);return tp_gp_face(s,mode,r,m);}
 if(mode==2){occ::handle<Geom_Surface> s=tp_surface2(r);m=M(s,r.real());}
 else if(mode==3||mode==10){occ::handle<Geom_Surface> s=tp_surface2(r);double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();m=M(s,u1,u2,v1,v2,r.real());
  if(mode==10){print(m);occ::handle<Geom_Surface> s2=tp_surface2(r);double a1=r.real(),a2=r.real(),b1=r.real(),b2=r.real();m.Init(s2,a1,a2,b1,b2,r.real());}}
 else if(mode==5){occ::handle<Geom_Surface> s=tp_surface2(r);TopoDS_Wire w=TopoDS::Wire(tp_shape_record(r));m=M(s,w,r.flag());}
 else if(mode==6){TopoDS_Wire w=TopoDS::Wire(tp_shape_record(r));m=M(w,r.flag());}
 else if(mode==7){m=M(TopoDS::Face(tp_shape_record(r)));}
 else if(mode==8){TopoDS_Face f=TopoDS::Face(tp_shape_record(r));m=M(f,TopoDS::Wire(tp_shape_record(r)));}
 else{occ::handle<Geom_Surface> s=tp_surface2(r);bool bound=r.flag();m.Init(s,bound,r.real());int n=r.integer();
  for(int i=0;i<n;++i)m.Add(TopoDS::Wire(tp_shape_record(r)));}
 return true;}
void tp_blm_face_case(std::istream& input){
 GaReader r{input};
 try{BRepLib_MakeFace m;
  if(!tp_face_request(r,m,tp_face_maker)){ga_int(-2);ga_end();return;}
  tp_face_maker(m);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blm_facedegen_case(std::istream& input){
 GaReader r{input};
 try{occ::handle<Geom_Curve> c=tp_geom_curve(r);double tol=-7.;ga_flag(BRepLib_MakeFace::IsDegenerated(c,r.real(),tol));ga_real(tol);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_findsurface_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);double tol=r.real();bool plane=r.flag(),closed=r.flag();
  BRepLib_FindSurface fs(s,tol,plane,closed);
  ga_flag(fs.Found());ga_flag(fs.Existed());ga_real(fs.Tolerance());ga_real(fs.ToleranceReached());ga_flag(fs.Location().IsIdentity());
  occ::handle<Geom_Surface> surf=fs.Surface();tp_type(surf);
  if(!surf.IsNull())for(auto uv:{gp_Pnt2d(0,0),gp_Pnt2d(1,0),gp_Pnt2d(0,1)})GA_TRY(ga_pnt(surf->Value(uv.X(),uv.Y())))
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_sameparam_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);int mode=r.integer();double tol=r.real();bool forced=r.flag();
  if(mode==0)BRepLib::SameParameter(s,tol,forced);
  else if(mode==1){BRepTools_ReShape rs;BRepLib::SameParameter(s,rs,tol,forced);tp_shape(s);tp_shape(rs.Apply(s));}
  else{TpShapeMap m=tp_sub_shapes(s,TopAbs_EDGE);
   for(int i=1;i<=m.Extent();++i){const TopoDS_Edge& e=TopoDS::Edge(m(i));
    if(mode==2){GA_TRY(BRepLib::SameParameter(e,tol))continue;}
    double nt=-1.;GA_TRY(TopoDS_Edge e2=BRepLib::SameParameter(e,tol,nt,mode==4);ga_real(nt);if(mode==3)tp_shape(e2))}}
  tp_shape(s);
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
void tp_blp_extendface_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);double ext=r.real();bool d[4];for(bool& x:d)x=r.flag();TpShapeMap m=tp_sub_shapes(s,TopAbs_FACE);
  for(int i=1;i<=m.Extent();++i){TopoDS_Face fe;GA_TRY(BRepLib::ExtendFace(TopoDS::Face(m(i)),ext,d[0],d[1],d[2],d[3],fe);tp_shape(fe))}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
