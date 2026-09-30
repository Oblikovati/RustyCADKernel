// The shape records and the grid mesher of the BRepLib package probes
// (tktopalgo_probe_breplib_package.cxx); the Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_lib_shapes.rs.
//
// A shape record is `kind ...`: 0 a face of the face reader; 1 the BRepLib_MakeShell
// shell of a surface record, `bounded` (then UMin UMax VMin VMax) and the segment flag;
// 2 a compound `n (shape record)...`; 3 the BRepLib_MakeEdge edge of a Geom curve record
// between two parameters, its vertices enlarged to two tolerances (a null shape when the
// maker is not done); 4 a BRep_Builder solid `n (shape record)... reversed` of the
// shells of the records (each reversed when `reversed`); 5 to 8 are read by
// tp_shape_record_more (tktopalgo_probe_breplib_face.cxx).
//
// `tp_mesh(shape, n, uv)` gives every face (explorer order, each once) an n x n grid
// triangulation over the box of the ends of its curves on surface (the node points on
// the surface, UV nodes when `uv`), and each edge on a side of that box (by the middle
// of its curve on surface, seams excepted) a polygon on it, ordered along the edge.
#include <BRepLib_MakeShell.hxx>
#include <Poly_PolygonOnTriangulation.hxx>
#include <Poly_Triangulation.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Solid.hxx>
#include <TopTools_ShapeMapHasher.hxx>
TopoDS_Shape tp_shape_record_more(GaReader& r,int kind);
TopoDS_Shape tp_shape_record(GaReader& r){
 int kind=r.integer();BRep_Builder B;
 if(kind==0)return tp_face(r).face;
 if(kind==1){occ::handle<Geom_Surface> s=tp_surface(r);bool bounded=r.flag();
  if(bounded){double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();BRepLib_MakeShell m(s,u1,u2,v1,v2,r.flag());return m.Shell();}
  BRepLib_MakeShell m(s,r.flag());return m.Shell();}
 if(kind==2){TopoDS_Compound c;B.MakeCompound(c);int n=r.integer();for(int i=0;i<n;++i)B.Add(c,tp_shape_record(r));return c;}
 if(kind==4){TopoDS_Solid so;B.MakeSolid(so);int n=r.integer();std::vector<TopoDS_Shape> shells;
  for(int i=0;i<n;++i)shells.push_back(tp_shape_record(r));bool rev=r.flag();
  for(const TopoDS_Shape& sh:shells)B.Add(so,rev?sh.Reversed():sh);return so;}
 if(kind>=5)return tp_shape_record_more(r,kind);
 occ::handle<Geom_Curve> c=tp_geom_curve(r);double p1=r.real(),p2=r.real(),t1=r.real(),t2=r.real();
 BRepLib_MakeEdge m(c,p1,p2);if(!m.IsDone())return TopoDS_Shape();
 if(!m.Vertex1().IsNull())B.UpdateVertex(m.Vertex1(),t1);
 if(!m.Vertex2().IsNull())B.UpdateVertex(m.Vertex2(),t2);
 return m.Edge();}
// The box of the ends of the curves on surface of the face.
void tp_face_box(const TopoDS_Face& f,double& u1,double& u2,double& v1,double& v2){
 u1=v1=RealLast();u2=v2=-RealLast();
 for(TopExp_Explorer x(f,TopAbs_EDGE);x.More();x.Next()){double a,b;
  occ::handle<Geom2d_Curve> c=BRep_Tool::CurveOnSurface(TopoDS::Edge(x.Current()),f,a,b);if(c.IsNull())continue;
  for(double t:{a,b}){if(Precision::IsInfinite(t))continue;gp_Pnt2d p=c->Value(t);
   u1=std::min(u1,p.X());u2=std::max(u2,p.X());v1=std::min(v1,p.Y());v2=std::max(v2,p.Y());}}}
// The polygon of the curve of an edge on a side of the grid box (side 0 u=u1, 1 u=u2,
// 2 v=v1, 3 v=v2), null when the curve is on none.
occ::handle<Poly_PolygonOnTriangulation> tp_side_polygon(const TopoDS_Edge& e,const TopoDS_Face& f,int n,const double box[4]){
 double a,b;occ::handle<Geom2d_Curve> c=BRep_Tool::CurveOnSurface(e,f,a,b);
 if(c.IsNull()||Precision::IsInfinite(a)||Precision::IsInfinite(b))return nullptr;
 gp_Pnt2d m=c->Value(0.5*(a+b)),pa=c->Value(a),pb=c->Value(b);double tol=1e-9;int side=-1;
 if(std::abs(m.X()-box[0])<tol)side=0;else if(std::abs(m.X()-box[1])<tol)side=1;
 else if(std::abs(m.Y()-box[2])<tol)side=2;else if(std::abs(m.Y()-box[3])<tol)side=3;
 if(side<0)return nullptr;bool along_v=side<2;bool increasing=along_v?pa.Y()<pb.Y():pa.X()<pb.X();
 occ::handle<Poly_PolygonOnTriangulation> p=new Poly_PolygonOnTriangulation(n+1,false);
 for(int k=0;k<=n;++k){int s=increasing?k:n-k;int i=along_v?(side==0?0:n):s,j=along_v?s:(side==2?0:n);p->SetNode(k+1,i*(n+1)+j+1);}
 return p;}
// The polygon of an edge on the grid: a seam gets the polygons of its two curves (the
// FORWARD edge first, on the FORWARD face) when both are on sides.
void tp_edge_polygon(const TopoDS_Edge& e,const TopoDS_Face& f,const occ::handle<Poly_Triangulation>& t,int n,const double box[4]){
 BRep_Builder B;
 if(!BRep_Tool::IsClosed(e,f)){occ::handle<Poly_PolygonOnTriangulation> p=tp_side_polygon(e,f,n,box);if(!p.IsNull())B.UpdateEdge(e,p,t,TopLoc_Location());return;}
 TopoDS_Edge fe=TopoDS::Edge(e.Oriented(TopAbs_FORWARD));TopoDS_Face ff=TopoDS::Face(f.Oriented(TopAbs_FORWARD));
 occ::handle<Poly_PolygonOnTriangulation> p1=tp_side_polygon(fe,ff,n,box),p2=tp_side_polygon(TopoDS::Edge(fe.Reversed()),ff,n,box);
 if(!p1.IsNull()&&!p2.IsNull())B.UpdateEdge(e,p1,p2,t,TopLoc_Location());}
void tp_mesh_face(const TopoDS_Face& f,int n,bool uv){
 occ::handle<Geom_Surface> s=BRep_Tool::Surface(f);if(s.IsNull())return;
 double box[4];tp_face_box(f,box[0],box[1],box[2],box[3]);if(box[0]>box[1])return;
 occ::handle<Poly_Triangulation> t=new Poly_Triangulation((n+1)*(n+1),2*n*n,uv);
 for(int i=0;i<=n;++i)for(int j=0;j<=n;++j){double u=box[0]+(box[1]-box[0])*i/n,v=box[2]+(box[3]-box[2])*j/n;int k=i*(n+1)+j+1;
  t->SetNode(k,s->Value(u,v));if(uv)t->SetUVNode(k,gp_Pnt2d(u,v));}
 int tri=0;for(int i=0;i<n;++i)for(int j=0;j<n;++j){int a=i*(n+1)+j+1,b=a+n+1,c=b+1,d=a+1;
  t->SetTriangle(++tri,Poly_Triangle(a,b,c));t->SetTriangle(++tri,Poly_Triangle(a,c,d));}
 BRep_Builder().UpdateFace(f,t);
 for(TopExp_Explorer x(f,TopAbs_EDGE);x.More();x.Next())tp_edge_polygon(TopoDS::Edge(x.Current()),f,t,n,box);}
void tp_mesh(const TopoDS_Shape& sh,int n,bool uv){
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> faces;TopExp::MapShapes(sh,TopAbs_FACE,faces);
 for(int i=1;i<=faces.Extent();++i)tp_mesh_face(TopoDS::Face(faces(i)),n,uv);}
// The normals of every face triangulation (MapShapes order) as doubles of their floats,
// `-1` for a face without triangulation, `0` for a triangulation without normals.
void tp_normals(const TopoDS_Shape& sh){
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> faces;TopExp::MapShapes(sh,TopAbs_FACE,faces);
 for(int i=1;i<=faces.Extent();++i){TopLoc_Location l;occ::handle<Poly_Triangulation> t=BRep_Tool::Triangulation(TopoDS::Face(faces(i)),l);
  if(t.IsNull()){ga_int(-1);continue;}if(!t->HasNormals()){ga_int(0);continue;}ga_int(t->NbNodes());
  for(int k=1;k<=t->NbNodes();++k){NCollection_Vec3<float> v;t->Normal(k,v);ga_real(v.x());ga_real(v.y());ga_real(v.z());}}}
