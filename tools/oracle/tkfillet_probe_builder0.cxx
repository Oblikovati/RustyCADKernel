// The free functions of ChFi3d_Builder_0 that run without the TKBool data structure.
#include <BRepAdaptor_Curve.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <Bnd_Box.hxx>
#include <ChFiDS_FilSpine.hxx>
#include <ChFiDS_Map.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <GeomFill_Boundary.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>

void fl_box(const Bnd_Box& b){GA_TRY(double a[6];b.Get(a[0],a[1],a[2],a[3],a[4],a[5]);for(double x:a)ga_real(x));}
void fl_adaptor_bounds(const Adaptor3d_Surface& s){
 ga_real(s.FirstUParameter());ga_real(s.LastUParameter());ga_real(s.FirstVParameter());ga_real(s.LastVParameter());}
// A 2d curve: its range and three points.
void fl_curve2d(const occ::handle<Geom2d_Curve>& c){
 ga_flag(c.IsNull());if(c.IsNull())return;double f=c->FirstParameter(),l=c->LastParameter();ga_real(f);ga_real(l);
 GA_TRY(ga_pnt2d(c->Value(f));ga_pnt2d(c->Value(0.5*(f+l)));ga_pnt2d(c->Value(l)));}
void fl_bezier2d(const occ::handle<Geom2d_Curve>& c){
 occ::handle<Geom2d_BezierCurve> b=occ::down_cast<Geom2d_BezierCurve>(c);ga_flag(b.IsNull());
 if(!b.IsNull())for(int i=1;i<=b->NbPoles();++i)ga_pnt2d(b->Pole(i));}
occ::handle<Geom_Surface> fl_geom_surface(GaReader& r){return occ::down_cast<GeomAdaptor_Surface>(fl_surface(r))->Surface();}
occ::handle<Geom_Curve> fl_geom_guide(GaReader& r){return occ::down_cast<GeomAdaptor_Curve>(fl_guide(r))->Curve();}
// The index of a face in the face map of the solid (0 for a null face).
int fl_face_index(const TopoDS_Shape& s,const TopoDS_Face& f){
 if(f.IsNull())return 0;NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> m;TopExp::MapShapes(s,TopAbs_FACE,m);return m.FindIndex(f);}

// `fl_b0_scalars u f l eps P1 P2 P3 P4`: ChFi3d_InPeriod and both ChFi3d_Boite.
void fl_b0_scalars_case(std::istream& in){
 GaReader r{in};double u=r.real(),f=r.real(),l=r.real(),eps=r.real();gp_Pnt2d p1=r.pnt2d(),p2=r.pnt2d(),p3=r.pnt2d(),p4=r.pnt2d();
 ga_real(ChFi3d_InPeriod(u,f,l,eps));double mu,Mu,mv,Mv,Du,Dv;
 ChFi3d_Boite(p1,p2,mu,Mu,mv,Mv);ga_real(mu);ga_real(Mu);ga_real(mv);ga_real(Mv);
 ChFi3d_Boite(p1,p2,p3,p4,Du,Dv,mu,Mu,mv,Mv);ga_real(Du);ga_real(Dv);ga_real(mu);ga_real(Mu);ga_real(mv);ga_real(Mv);
 ga_end();}

// `fl_bound_srf SURFACE trim u1 u2 v1 v2 umin umax vmin vmax check`: ChFi3d_BoundSrf on the
// surface (trimmed to [u1, u2] x [v1, v2] when asked): the bounds of the adaptor.
void fl_bound_srf_case(std::istream& in){
 GaReader r{in};occ::handle<Geom_Surface> s=fl_geom_surface(r);bool trim=r.flag();double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
 double umin=r.real(),umax=r.real(),vmin=r.real(),vmax=r.real();bool check=r.flag();
 GA_TRY(if(trim)s=new Geom_RectangularTrimmedSurface(s,u1,u2,v1,v2);
  GeomAdaptor_Surface a(s);ChFi3d_BoundSrf(a,umin,umax,vmin,vmax,check);fl_adaptor_bounds(a));
 ga_end();}

// `fl_bound_fac SOLID t check`: ChFi3d_BoundFac on each face of the solid around the middle
// part [t, 1 - t] of its UV bounds: the bounds of the adaptor.
void fl_bound_fac_case(std::istream& in){
 GaReader r{in};TopoDS_Shape s=fl_solid(r);double t=r.real();bool check=r.flag();
 for(TopExp_Explorer ex(s,TopAbs_FACE);ex.More();ex.Next()){TopoDS_Face f=TopoDS::Face(ex.Current());
  GA_TRY(double u1,u2,v1,v2;BRepTools::UVBounds(f,u1,u2,v1,v2);BRepAdaptor_Surface a(f);
   ChFi3d_BoundFac(a,u1+t*(u2-u1),u2-t*(u2-u1),v1+t*(v2-v1),v2-t*(v2-v1),check);fl_adaptor_bounds(a));}
 ga_end();}

// `fl_enlarge_box GUIDE wd wf SURFACE P D wd2 wf2`: the ChFi3d_EnlargeBox of the curve and
// of the line (P, D) on the surface.
void fl_enlarge_box_case(std::istream& in){
 GaReader r{in};occ::handle<Geom_Curve> c=fl_geom_guide(r);double wd=r.real(),wf=r.real();
 occ::handle<Adaptor3d_Surface> s=fl_surface(r);gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();double wd2=r.real(),wf2=r.real();
 Bnd_Box b1,b2,b3,b4;GA_TRY(ChFi3d_EnlargeBox(c,wd,wf,b1,b2));fl_box(b1);fl_box(b2);
 occ::handle<Geom2d_Curve> pc=new Geom2d_Line(p,d);GA_TRY(ChFi3d_EnlargeBox(s,pc,wd2,wf2,b3,b4));fl_box(b3);fl_box(b4);
 ga_end();}

// `fl_b0_faces SOLID t radius`: for each edge of the solid, ChFi3d_conexfaces (face
// indices), ChFi3d_evalconti, ChFi3d_EnlargeBox at the parameter t of the edge range and
// ChFi3d_KParticular of a constant fillet spine on the edge; for each vertex of three
// edges, ChFi3d_EdgeState.
void fl_b0_faces_case(std::istream& in){
 GaReader r{in};TopoDS_Shape s=fl_solid(r);double t=r.real(),radius=r.real();
 ChFiDS_Map ef,ve;ef.Fill(s,TopAbs_EDGE,TopAbs_FACE);ve.Fill(s,TopAbs_VERTEX,TopAbs_EDGE);FlEdgeFaces verts;TopExp::MapShapesAndAncestors(s,TopAbs_VERTEX,TopAbs_EDGE,verts);
 FlEdgeFaces edges=fl_edge_faces(s);ga_int(edges.Extent());
 for(int i=1;i<=edges.Extent();++i){TopoDS_Edge e=TopoDS::Edge(edges.FindKey(i));TopoDS_Face f1,f2;
  GA_TRY(ChFi3d_conexfaces(e,f1,f2,ef);ga_int(fl_face_index(s,f1));ga_int(fl_face_index(s,f2)));
  if(f1.IsNull()||f2.IsNull())continue;
  GA_TRY(ga_int(int(ChFi3d_evalconti(e,f1,f2))));
  GA_TRY(double a,b;BRep_Tool::Range(e,a,b);Bnd_Box bx;ChFi3d_EnlargeBox(e,ef(i),a+t*(b-a),bx);fl_box(bx));
  GA_TRY(occ::handle<ChFiDS_FilSpine> sp=new ChFiDS_FilSpine();sp->SetEdges(e);sp->Load();sp->SetRadius(radius);
   ga_flag(ChFi3d_KParticular(sp,1,BRepAdaptor_Surface(f1),BRepAdaptor_Surface(f2))));}
 for(int i=1;i<=verts.Extent();++i){const NCollection_List<TopoDS_Shape>& es=ve(i);ga_int(es.Extent());
  if(es.Extent()!=3)continue;TopoDS_Edge e3[3];int k=0;for(const TopoDS_Shape& x:es)e3[k++]=TopoDS::Edge(x);
  GA_TRY(ga_int(int(ChFi3d_EdgeState(e3,ef))));}
 ga_end();}

// `fl_b0_spines Pd Vd Pf Vf R`: ChFi3d_CircularSpine and ChFi3d_Spine.
void fl_b0_spines_case(std::istream& in){
 GaReader r{in};gp_Pnt pd=r.pnt();gp_Vec vd(r.xyz());gp_Pnt pf=r.pnt();gp_Vec vf(r.xyz());double rad=r.real();
 GA_TRY(double w1=-1,w2=-1;occ::handle<Geom_Circle> c=ChFi3d_CircularSpine(w1,w2,pd,vd,pf,vf,rad);ga_flag(c.IsNull());
  if(!c.IsNull()){ga_pnt(c->Location());ga_dir(c->Axis().Direction());ga_dir(c->XAxis().Direction());ga_real(c->Radius());ga_real(w1);ga_real(w2);});
 GA_TRY(occ::handle<Geom_BezierCurve> b=ChFi3d_Spine(pd,vd,pf,vf,rad);for(int i=1;i<=b->NbPoles();++i)ga_pnt(b->Pole(i));
  ga_xyz(vd.XYZ());ga_xyz(vf.XYZ()));
 ga_end();}

// `fl_inter_plane_edge SURFACE GUIDE f l sens tolc`: ChFi3d_InterPlaneEdge of the guide on
// [f, l] with the surface.
void fl_inter_plane_edge_case(std::istream& in){
 GaReader r{in};occ::handle<Adaptor3d_Surface> s=fl_surface(r);occ::handle<Geom_Curve> c=fl_geom_guide(r);
 double f=r.real(),l=r.real();bool sens=r.flag();double tolc=r.real();
 GA_TRY(double w=-7.0;ga_flag(ChFi3d_InterPlaneEdge(s,new GeomAdaptor_Curve(c,f,l),w,sens,tolc));ga_real(w));
 ga_end();}

// `fl_b0_pcurves UV1 UV2 pardeb parfin reverse GUIDE SURFACE tol3d`: ChFi3d_ComputePCurv
// (straight and brought to the same parameter as the guide on [pardeb, parfin]) and
// ChFi3d_CheckSameParameter; the adaptor and the Geom overloads.
void fl_b0_pcurves_case(std::istream& in){
 GaReader r{in};gp_Pnt2d uv1=r.pnt2d(),uv2=r.pnt2d();double pd=r.real(),pf=r.real();bool rev=r.flag();
 occ::handle<Geom_Curve> c=fl_geom_guide(r);occ::handle<Geom_Surface> s=fl_geom_surface(r);double tol=r.real();
 occ::handle<Geom2d_Curve> pc;GA_TRY(ChFi3d_ComputePCurv(uv1,uv2,pc,pd,pf,rev);fl_curve2d(pc));
 occ::handle<Adaptor3d_Curve> hc=new GeomAdaptor_Curve(c,pd,pf);occ::handle<Adaptor3d_Surface> hs=new GeomAdaptor_Surface(s);
 GA_TRY(double reached=-1;ga_flag(ChFi3d_CheckSameParameter(hc,pc,hs,tol,reached));ga_real(reached));
 GA_TRY(occ::handle<Geom2d_Curve> pc2;double reached=-1;ChFi3d_ComputePCurv(c,uv1,uv2,pc2,s,pd,pf,tol,reached,rev);ga_real(reached);fl_curve2d(pc2));
 GA_TRY(occ::handle<Geom2d_Curve> pc3=pc;double reached=-1;ga_flag(ChFi3d_SameParameter(c,pc3,s,pd,pf,tol,reached));ga_real(reached);fl_curve2d(pc3));
 ga_end();}

// `fl_reparam_pcurv P1 P2 P3 P4 k1 k2 trim t1 t2 Uf Ul`: ChFi3d_ReparamPcurv of the cubic
// B-spline of the poles on [k1, k2] (trimmed to [t1, t2] when asked).
void fl_reparam_pcurv_case(std::istream& in){
 GaReader r{in};NCollection_Array1<gp_Pnt2d> p(1,4);for(int i=1;i<=4;++i)p(i)=r.pnt2d();
 NCollection_Array1<double> k(1,2);k(1)=r.real();k(2)=r.real();NCollection_Array1<int> m(1,2);m.Init(4);
 bool trim=r.flag();double t1=r.real(),t2=r.real(),uf=r.real(),ul=r.real();
 GA_TRY(occ::handle<Geom2d_Curve> c=new Geom2d_BSplineCurve(p,k,m,3);if(trim)c=new Geom2d_TrimmedCurve(c,t1,t2);
  ChFi3d_ReparamPcurv(uf,ul,c);fl_curve2d(c));
 ga_end();}

// `fl_project_pcurv GUIDE f l SURFACE tol`: ChFi3d_ProjectPCurv of the guide on [f, l].
void fl_project_pcurv_case(std::istream& in){
 GaReader r{in};occ::handle<Geom_Curve> c=fl_geom_guide(r);double f=r.real(),l=r.real();occ::handle<Adaptor3d_Surface> s=fl_surface(r);double tol=r.real();
 GA_TRY(occ::handle<Geom2d_Curve> pc;double reached=-1;ChFi3d_ProjectPCurv(new GeomAdaptor_Curve(c,f,l),s,pc,tol,reached);ga_real(reached);fl_curve2d(pc));
 ga_end();}

// A boundary: its bounds, points and derivatives at both ends and the middle, normals.
void fl_boundary(const occ::handle<GeomFill_Boundary>& b){
 double f,l;b->Bounds(f,l);ga_real(f);ga_real(l);ga_flag(b->HasNormals());ga_flag(b->IsDegenerated());
 for(double u:{f,0.5*(f+l),l}){gp_Pnt p;gp_Vec v;b->D1(u,p,v);ga_pnt(p);ga_xyz(v.XYZ());if(b->HasNormals())ga_xyz(b->Norm(u).XYZ());}}

// `fl_b0_bounds SURFACE p1 d1 p2 d2 redresse V1 V2 sens1 sens2 t3d ta free`:
// ChFi3d_BuildPCurve (2d directions, 2d and 3d tangents on the surface) and the
// ChFi3d_mkbound overloads.
void fl_b0_bounds_case(std::istream& in){
 GaReader r{in};occ::handle<Adaptor3d_Surface> s=fl_surface(r);gp_Pnt2d p1=r.pnt2d();gp_Dir2d d1=r.dir2d();
 gp_Pnt2d p2=r.pnt2d();gp_Dir2d d2=r.dir2d();bool red=r.flag();gp_Vec v1(r.xyz()),v2(r.xyz());int s1=r.integer(),s2=r.integer();
 double t3d=r.real(),ta=r.real();bool free=r.flag();
 GA_TRY(gp_Dir2d e1=d1,e2=d2;fl_bezier2d(ChFi3d_BuildPCurve(p1,e1,p2,e2,red));ga_real(e1.X());ga_real(e1.Y());ga_real(e2.X());ga_real(e2.Y()));
 GA_TRY(fl_bezier2d(ChFi3d_BuildPCurve(s,p1,gp_Vec2d(d1),p2,gp_Vec2d(d2),red)));
 GA_TRY(fl_bezier2d(ChFi3d_BuildPCurve(s,p1,v1,p2,v2,red)));
 GA_TRY(fl_boundary(ChFi3d_mkbound(s,p1,p2,t3d,ta,free)));
 GA_TRY(occ::handle<Geom2d_Curve> c;fl_boundary(ChFi3d_mkbound(s,c,p1,p2,t3d,ta,free));fl_bezier2d(c));
 GA_TRY(fl_boundary(ChFi3d_mkbound(occ::down_cast<GeomAdaptor_Surface>(s)->Surface(),p1,p2,t3d,ta,free)));
 GA_TRY(occ::handle<Geom2d_Curve> c;fl_boundary(ChFi3d_mkbound(s,c,s1,p1,gp_Vec2d(d1),s2,p2,gp_Vec2d(d2),t3d,ta));fl_bezier2d(c));
 GA_TRY(occ::handle<Geom2d_Curve> c;gp_Vec w1=v1,w2=v2;fl_boundary(ChFi3d_mkbound(s,c,s1,p1,w1,s2,p2,w2,t3d,ta));fl_bezier2d(c);
  ga_xyz(w1.XYZ());ga_xyz(w2.XYZ()));
 ga_end();}
