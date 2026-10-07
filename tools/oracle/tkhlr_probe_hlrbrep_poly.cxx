// HLRBRep_PolyAlgo, HLRBRep_PolyHLRToShape, HLRBRep_BiPnt2D and HLRBRep_BiPoint, the Rust
// twin of crates/tkhlr/examples/hl_probe/hlrbrep_poly.rs. The shapes are meshed by a
// deterministic grid (fan on a disk) both probes build alike, not by BRepMesh.
#include <BRepTools.hxx>
#include <Geom2d_Circle.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <HLRBRep_BiPnt2D.hxx>
#include <HLRBRep_BiPoint.hxx>
#include <HLRBRep_PolyAlgo.hxx>
#include <HLRBRep_PolyHLRToShape.hxx>
#include <Poly_PolygonOnTriangulation.hxx>
#include <Poly_Triangulation.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <vector>
// The polygon of a boundary of the UV rectangle from the pcurve start `a` to its end `z`:
// the grid nodes along it and their edge parameters at the fraction of the way.
occ::handle<Poly_PolygonOnTriangulation> hb_grid_polygon(const gp_Pnt2d& a,const gp_Pnt2d& z,double f0,double l0,
 const double us[4],int nu,int nv){
 const double eps=1e-9;bool vert=std::abs(a.X()-z.X())<eps;int n=vert?nv:nu;
 bool atMin=vert?std::abs(a.X()-us[0])<eps:std::abs(a.Y()-us[2])<eps;
 NCollection_Array1<int> nodes(1,n+1);NCollection_Array1<double> pars(1,n+1);
 bool up=vert?a.Y()<z.Y():a.X()<z.X();
 for(int k=0;k<=n;++k){int s=up?k:n-k;int i=vert?(atMin?0:nu):s,j=vert?s:(atMin?0:nv);
  nodes(k+1)=1+j*(nu+1)+i;double c=vert?us[2]+(us[3]-us[2])*j/nv:us[0]+(us[1]-us[0])*i/nu;
  double frac=vert?(c-a.Y())/(z.Y()-a.Y()):(c-a.X())/(z.X()-a.X());pars(k+1)=f0+(l0-f0)*frac;}
 occ::handle<Poly_PolygonOnTriangulation> p=new Poly_PolygonOnTriangulation(nodes,pars);return p;}
void hb_mesh_grid(const TopoDS_Face& f,int nu,int nv){
 TopLoc_Location loc;occ::handle<Geom_Surface> s=BRep_Tool::Surface(f,loc);double us[4];
 BRepTools::UVBounds(f,us[0],us[1],us[2],us[3]);
 occ::handle<Poly_Triangulation> t=new Poly_Triangulation((nu+1)*(nv+1),2*nu*nv,true);
 for(int j=0;j<=nv;++j)for(int i=0;i<=nu;++i){double u=us[0]+(us[1]-us[0])*i/nu,v=us[2]+(us[3]-us[2])*j/nv;
  int k=1+j*(nu+1)+i;t->SetNode(k,s->Value(u,v));t->SetUVNode(k,gp_Pnt2d(u,v));}
 int k=1;for(int j=0;j<nv;++j)for(int i=0;i<nu;++i){int a=1+j*(nu+1)+i,b=a+1,c=b+nu+1,d=a+nu+1;
  t->SetTriangle(k++,Poly_Triangle(a,b,c));t->SetTriangle(k++,Poly_Triangle(a,c,d));}
 BRep_Builder bb;bb.UpdateFace(f,t);
 std::vector<TopoDS_Edge> done;
 for(TopExp_Explorer ex(f,TopAbs_EDGE);ex.More();ex.Next()){TopoDS_Edge e=TopoDS::Edge(ex.Current());
  bool seen=false;for(const TopoDS_Edge& d:done)seen|=d.IsSame(e);if(seen)continue;done.push_back(e);
  if(BRep_Tool::IsClosed(e,f)){occ::handle<Poly_PolygonOnTriangulation> p[2];
   for(int o=0;o<2;++o){TopoDS_Edge eo=e;eo.Orientation(o==0?TopAbs_FORWARD:TopAbs_REVERSED);double f0,l0;
    occ::handle<Geom2d_Curve> pc=BRep_Tool::CurveOnSurface(eo,f,f0,l0);p[o]=hb_grid_polygon(pc->Value(f0),pc->Value(l0),f0,l0,us,nu,nv);}
   bb.UpdateEdge(e,p[0],p[1],t,loc);}
  else{double f0,l0;occ::handle<Geom2d_Curve> pc=BRep_Tool::CurveOnSurface(e,f,f0,l0);
   bb.UpdateEdge(e,hb_grid_polygon(pc->Value(f0),pc->Value(l0),f0,l0,us,nu,nv),t,loc);}}}
// A disk bounded by one closed edge: a fan from the circle centre.
void hb_mesh_fan(const TopoDS_Face& f,const TopoDS_Edge& e,int n){
 TopLoc_Location loc;occ::handle<Geom_Surface> s=BRep_Tool::Surface(f,loc);double f0,l0;
 occ::handle<Geom2d_Curve> pc=BRep_Tool::CurveOnSurface(e,f,f0,l0);
 gp_Pnt2d c=occ::down_cast<Geom2d_Circle>(pc)->Location();
 occ::handle<Poly_Triangulation> t=new Poly_Triangulation(n+1,n,true);
 t->SetNode(1,s->Value(c.X(),c.Y()));t->SetUVNode(1,c);NCollection_Array1<double> pars(1,n+1);
 for(int k=0;k<n;++k){double tk=f0+(l0-f0)*k/n;gp_Pnt2d uv=pc->Value(tk);pars(k+1)=tk;
  t->SetNode(k+2,s->Value(uv.X(),uv.Y()));t->SetUVNode(k+2,uv);}
 pars(n+1)=l0;gp_Pnt2d a=t->UVNode(2),b=t->UVNode(3);
 bool flip=(a.X()-c.X())*(b.Y()-c.Y())-(a.Y()-c.Y())*(b.X()-c.X())<0;
 for(int k=0;k<n;++k){int i=k+2,j=k+1<n?k+3:2;t->SetTriangle(k+1,flip?Poly_Triangle(1,j,i):Poly_Triangle(1,i,j));}
 NCollection_Array1<int> nodes(1,n+1);for(int k=0;k<n;++k)nodes(k+1)=k+2;nodes(n+1)=2;
 BRep_Builder bb;bb.UpdateFace(f,t);bb.UpdateEdge(e,new Poly_PolygonOnTriangulation(nodes,pars),t,loc);}
// Meshes every face: a fan on a plane bounded by one edge, else an `n` x `n` grid on a
// plane and an `n` x `m` grid on other surfaces.
void hb_mesh(const TopoDS_Shape& sh,int n,int m){
 for(TopExp_Explorer fx(sh,TopAbs_FACE);fx.More();fx.Next()){TopoDS_Face f=TopoDS::Face(fx.Current());
  std::vector<TopoDS_Edge> es;for(TopExp_Explorer ex(f,TopAbs_EDGE);ex.More();ex.Next())es.push_back(TopoDS::Edge(ex.Current()));
  bool plane=GeomAdaptor_Surface(BRep_Tool::Surface(f)).GetType()==GeomAbs_Plane;
  if(plane&&es.size()==1)hb_mesh_fan(f,es[0],n);else hb_mesh_grid(f,n,plane?n:m);}}
void hb_poly_segment(const HLRAlgo_BiPoint::PointsT& p,const TopoDS_Shape& s,const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& m,
 bool r1,bool rn,bool ol,bool il){
 for(const gp_XYZ& x:{p.Pnt1,p.Pnt2,p.PntP1,p.PntP2}){ga_real(x.X());ga_real(x.Y());ga_real(x.Z());}
 ga_int(m.FindIndex(s));for(bool b:{r1,rn,ol,il})ga_flag(b);}
// `projector shape n m tolcoef`: the mesh, PolyAlgo::Update, every hidden segment with
// its visible parts, every shown segment, the PolyHLRToShape compounds in hide and show
// modes and the outlined shape.
void hb_poly_case(std::istream& input){
 GaReader r{input};HLRAlgo_Projector projector=hb_projector(r);
 try{TopoDS_Shape s=hb_shape(r);int n=r.integer(),m=r.integer();double tc=r.real();hb_mesh(s,n,m);
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;TopExp::MapShapes(s,TopAbs_EDGE,map);TopExp::MapShapes(s,TopAbs_FACE,map);
  occ::handle<HLRBRep_PolyAlgo> a=new HLRBRep_PolyAlgo(s);a->Projector(projector);a->TolCoef(tc);a->Update();
  HLRAlgo_EdgeStatus st;TopoDS_Shape sh;bool r1,rn,ol,il;int nh=0;
  for(a->InitHide();a->MoreHide()&&nh<3000;a->NextHide(),++nh){
   HLRAlgo_BiPoint::PointsT& p=a->Hide(st,sh,r1,rn,ol,il);hb_poly_segment(p,sh,map,r1,rn,ol,il);
   int nv=st.NbVisiblePart();ga_int(nv);for(int k=1;k<=nv;++k){double x,y;float tx,ty;st.VisiblePart(k,x,tx,y,ty);ga_real(x);ga_real(y);}}
  ga_int(nh);int ns=0;
  for(a->InitShow();a->MoreShow()&&ns<3000;a->NextShow(),++ns){HLRAlgo_BiPoint::PointsT& p=a->Show(sh,r1,rn,ol,il);hb_poly_segment(p,sh,map,r1,rn,ol,il);}
  ga_int(ns);HLRBRep_PolyHLRToShape t;t.Update(a);
  for(int mode=0;mode<2;++mode){if(mode==1)t.Show();
   GA_TRY(hb_compound(t.VCompound()));GA_TRY(hb_compound(t.HCompound()));GA_TRY(hb_compound(t.OutLineVCompound()));
   GA_TRY(hb_compound(t.Rg1LineVCompound()));GA_TRY(hb_compound(t.RgNLineHCompound()));GA_TRY(hb_compound(t.VCompound(s)));}
  GA_TRY(hb_compound(a->OutLinedShape(s)));
 }catch(const Standard_Failure& ga_error){std::cout<<ga_error.ExceptionType()<<' ';}
 ga_end();}
