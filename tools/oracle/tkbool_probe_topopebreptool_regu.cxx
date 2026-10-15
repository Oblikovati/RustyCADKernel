// TopOpeBRepTool CLASSI, REGUW (Regularize), CORRISO (CorrectONUVISO, PurgeClosingEdges,
// MakeFaces), FuseEdges and PurgeInternalEdges probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/top_ope_brep_tool_regu.rs. Solids are the records of
// tkbool_probe_topopebreptool_tool.cxx.
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepBndLib.hxx>
#include <algorithm>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <TopOpeBRepTool.hxx>
#include <TopOpeBRepTool_CLASSI.hxx>
#include <TopOpeBRepTool_CORRISO.hxx>
#include <TopOpeBRepTool_FuseEdges.hxx>
#include <TopOpeBRepTool_PurgeInternalEdges.hxx>
#include <gp_Pln.hxx>

// The plane z = 0 face bounded by nothing (natural restriction).
TopoDS_Face bl_tt_plane(){return BRepBuilderAPI_MakeFace(gp_Pln()).Face();}

// `n (x0 y0 x1 y1 ccw)...`: the square wires on the plane z = 0.
std::vector<TopoDS_Shape> bl_tt_squares(GaReader& r){
 int n=r.integer();std::vector<TopoDS_Shape> w;
 for(int i=0;i<n;i++){double x0=r.real(),y0=r.real(),x1=r.real(),y1=r.real();bool ccw=r.flag();
  BRepBuilderAPI_MakePolygon poly;gp_Pnt p[4]={gp_Pnt(x0,y0,0),gp_Pnt(x1,y0,0),gp_Pnt(x1,y1,0),gp_Pnt(x0,y1,0)};
  for(int k=0;k<4;k++)poly.Add(p[ccw?k:3-k]);poly.Close();w.push_back(poly.Wire());}
 return w;}

// The index (from 1) of a shape among the wires, 0 when absent.
int bl_tt_index(const std::vector<TopoDS_Shape>& w,const TopoDS_Shape& s){
 for(size_t i=0;i<w.size();i++)if(w[i].IsSame(s))return int(i+1);return 0;}

// `tt_classi squares`: ClassiBnd2d (strict and large) and Classip2d of every ordered pair
// of wires on the plane, then Classilist: per wire, whether it is a key and its smaller
// wires.
void bl_tt_classi_case(std::istream& input){
 GaReader r{input};std::vector<TopoDS_Shape> w=bl_tt_squares(r);
 TopOpeBRepTool_CLASSI c;c.Init2d(bl_tt_plane());
 for(size_t i=0;i<w.size();i++)for(size_t j=0;j<w.size();j++){if(i==j)continue;
  GA_TRY(int b=c.ClassiBnd2d(w[i],w[j],1.e-7,true);ga_int(b);ga_int(c.ClassiBnd2d(w[i],w[j],1.e-7,false));ga_int(c.Classip2d(w[i],w[j],b)))}
 GA_TRY(NCollection_List<TopoDS_Shape> l;for(auto& s:w)l.Append(s);
  NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> m;ga_flag(c.Classilist(l,m));
  for(auto& s:w){bool b=m.IsBound(s);ga_flag(b);if(!b)continue;ga_int(m(s).Extent());
   for(NCollection_List<TopoDS_Shape>::Iterator it(m(s));it.More();it.Next())ga_int(bl_tt_index(w,it.Value()));})
 ga_end();}

// The faces of a list sorted by their boxes (the list order follows a map of shapes,
// whose iteration depends on memory addresses natively): per face its box, wire and
// edge counts and orientation.
void bl_tt_sorted_faces(const NCollection_List<TopoDS_Shape>& l){
 std::vector<std::vector<double>> rows;
 for(NCollection_List<TopoDS_Shape>::Iterator it(l);it.More();it.Next()){Bnd_Box b;BRepBndLib::Add(it.Value(),b);
  double x0,y0,z0,x1,y1,z1;b.Get(x0,y0,z0,x1,y1,z1);int nw=0,ne=0;
  for(TopExp_Explorer ex(it.Value(),TopAbs_WIRE);ex.More();ex.Next())nw++;
  for(TopExp_Explorer ex(it.Value(),TopAbs_EDGE);ex.More();ex.Next())ne++;
  rows.push_back({x0,y0,z0,x1,y1,z1,double(nw),double(ne),double(it.Value().Orientation())});}
 std::sort(rows.begin(),rows.end());
 for(auto& row:rows){for(int k=0;k<6;k++)ga_real(row[k]);for(int k=6;k<9;k++)ga_int(int(row[k]));}}

// `tt_regularize n (x y)... m (i)...`: the face on the plane z = 0 of the closed wire of
// the vertices of the points visited in that order (shared vertices make multiple
// vertices), regularized: RegularizeWires (the new wire count) then Regularize (the
// faces).
void bl_tt_regularize_case(std::istream& input){
 GaReader r{input};int n=r.integer();std::vector<TopoDS_Vertex> v;
 for(int i=0;i<n;i++){double x=r.real();v.push_back(BRepBuilderAPI_MakeVertex(gp_Pnt(x,r.real(),0)).Vertex());}
 int m=r.integer();std::vector<int> ids;for(int i=0;i<m;i++)ids.push_back(r.integer());
 TopoDS_Face face;
 try{BRep_Builder bb;TopoDS_Wire w;bb.MakeWire(w);
  for(int i=0;i<m;i++)bb.Add(w,BRepBuilderAPI_MakeEdge(v[ids[i]],v[ids[(i+1)%m]]).Edge());
  face=BRepBuilderAPI_MakeFace(gp_Pln(),w,true).Face();}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 GA_TRY(NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> ow,es;
  ga_flag(TopOpeBRepTool::RegularizeWires(face,ow,es));ga_int(ow.Extent());
  for(TopExp_Explorer ex(face,TopAbs_WIRE);ex.More();ex.Next())ga_int(ow.IsBound(ex.Current())?ow(ex.Current()).Extent():-1);
  ga_int(es.Extent()))
 GA_TRY(NCollection_List<TopoDS_Shape> lof;NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> es;
  ga_flag(TopOpeBRepTool::Regularize(face,lof,es));ga_int(lof.Extent());bl_tt_sorted_faces(lof))
 ga_end();}

// `tt_corriso solid ie du dv`: on a copy of the first face of the solid with a periodic
// surface, the p-curve of its edge `ie` (explorer order) translated by (du, dv); then the
// CORRISO queries on the copy, the purge of its faulty closing edges, the faces made
// without them and CorrectONUVISO.
void bl_tt_corriso_case(std::istream& input){
 GaReader r{input};TopoDS_Shape s;
 try{s=bl_tt_solid(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int ie=r.integer();double tu=r.real(),tv=r.real();gp_Vec2d t(tu,tv);
 TopoDS_Face fin;for(TopExp_Explorer ex(s,TopAbs_FACE);ex.More();ex.Next()){TopoDS_Face f=TopoDS::Face(ex.Current());
  if(BRep_Tool::Surface(f)->IsUPeriodic()){fin=f;break;}}
 if(fin.IsNull()){ga_int(-1);ga_end();return;}
 TopoDS_Face fsp=TopoDS::Face(BRepBuilderAPI_Copy(fin).Shape());
 GA_TRY(int k=0;for(TopExp_Explorer ex(fsp,TopAbs_EDGE);ex.More();ex.Next())if(++k==ie){TopoDS_Edge e=TopoDS::Edge(ex.Current());
   ga_flag(TopOpeBRepTool_TOOL::TrslUVModifE(t,fsp,e));break;})
 TopOpeBRepTool_CORRISO c(fin);
 GA_TRY(double p=-1;ga_flag(c.Refclosed(1,p));ga_real(p);ga_flag(c.Refclosed(2,p));ga_real(p);ga_real(c.Tol(1,1.e-7));ga_real(c.Tol(2,1.e-7)))
 GA_TRY(ga_flag(c.Init(fsp));ga_int(c.Eds().Extent());ga_flag(c.UVClosed()))
 GA_TRY(for(NCollection_List<TopoDS_Shape>::Iterator it(c.Eds());it.More();it.Next()){const TopoDS_Edge& e=TopoDS::Edge(it.Value());
   int iv=-1;GA_TRY(ga_flag(c.EdgeWithFaultyUV(e,iv));ga_int(iv))
   double sp=-1;GA_TRY(ga_int(c.EdgeOUTofBoundsUV(e,true,1.e-7,sp));ga_real(sp))})
 GA_TRY(NCollection_DataMap<TopoDS_Shape,int> m;ga_flag(c.EdgesWithFaultyUV(c.Eds(),3,m));ga_int(m.Extent());
  ga_flag(c.EdgesOUTofBoundsUV(c.Eds(),true,1.e-7,m));ga_int(m.Extent()))
 GA_TRY(NCollection_List<TopoDS_Shape> cl,fy;for(NCollection_List<TopoDS_Shape>::Iterator it(c.Eds());it.More();it.Next())
   if(BRep_Tool::IsClosed(TopoDS::Edge(it.Value()),fin))cl.Append(it.Value());
  ga_int(cl.Extent());ga_flag(c.PurgeFyClosingE(cl,fy));ga_int(fy.Extent()))
 GA_TRY(NCollection_DataMap<TopoDS_Shape,int,TopTools_ShapeMapHasher> old;NCollection_IndexedMap<TopoDS_Shape> nok;
  NCollection_List<TopoDS_Shape> lof;lof.Append(fsp);ga_flag(TopOpeBRepTool::PurgeClosingEdges(fin,lof,old,nok));ga_int(nok.Extent());
  NCollection_List<TopoDS_Shape> loff;ga_flag(TopOpeBRepTool::MakeFaces(fin,lof,nok,loff));ga_int(loff.Extent());
  for(NCollection_List<TopoDS_Shape>::Iterator it(loff);it.More();it.Next())pr_shape(it.Value()))
 GA_TRY(TopoDS_Face g=fsp;ga_flag(TopOpeBRepTool::CorrectONUVISO(fin,g));ga_flag(g.IsSame(fsp));
  TopOpeBRepTool_CORRISO c2(fin);ga_flag(c2.Init(g));ga_flag(c2.UVClosed()))
 ga_end();}

// `tt_fuse_edges solid1 solid2`: FuseEdges on the fusion of the solids: the chains, the
// removed vertices, the fused edges, the changed faces and the shape.
void bl_tt_fuse_edges_case(std::istream& input){
 GaReader r{input};TopoDS_Shape s;
 try{TopoDS_Shape s1=bl_tt_solid(r),s2=bl_tt_solid(r);s=BRepAlgoAPI_Fuse(s1,s2).Shape();}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 GA_TRY(TopOpeBRepTool_FuseEdges fe(s,false);NCollection_DataMap<int,NCollection_List<TopoDS_Shape>> le;fe.Edges(le);ga_int(le.Extent());
  ga_int(fe.NbVertices());NCollection_DataMap<int,TopoDS_Shape> re;fe.ResultEdges(re);ga_int(re.Extent());
  NCollection_DataMap<TopoDS_Shape,TopoDS_Shape,TopTools_ShapeMapHasher> mf;fe.Faces(mf);ga_int(mf.Extent());pr_shape(fe.Shape()))
 ga_end();}

// The first vertex of the shape at the point.
TopoDS_Vertex bl_tt_vertex_at(const TopoDS_Shape& s,const gp_Pnt& p){
 for(TopExp_Explorer ex(s,TopAbs_VERTEX);ex.More();ex.Next())
  if(BRep_Tool::Pnt(TopoDS::Vertex(ex.Current())).Distance(p)<1.e-9)return TopoDS::Vertex(ex.Current());
 return TopoDS_Vertex();}

// `tt_purge_internal a`: the square [0, a]^2 face of the plane z = 0 with its diagonal
// from (0, 0) to (a, a) added INTERNAL to its wire, purged of its internal edges.
void bl_tt_purge_internal_case(std::istream& input){
 GaReader r{input};double a=r.real();
 GA_TRY(BRepBuilderAPI_MakePolygon poly;poly.Add(gp_Pnt(0,0,0));poly.Add(gp_Pnt(a,0,0));poly.Add(gp_Pnt(a,a,0));poly.Add(gp_Pnt(0,a,0));poly.Close();
  TopoDS_Face f=BRepBuilderAPI_MakeFace(poly.Wire()).Face();
  TopoDS_Edge d=BRepBuilderAPI_MakeEdge(bl_tt_vertex_at(f,gp_Pnt(0,0,0)),bl_tt_vertex_at(f,gp_Pnt(a,a,0))).Edge();
  BRep_Builder bb;TopoDS_Shape w=TopExp_Explorer(f,TopAbs_WIRE).Current();w.Free(true);bb.Add(w,d.Oriented(TopAbs_INTERNAL));
  TopOpeBRepTool_PurgeInternalEdges p(f,true);ga_int(p.NbEdges());NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> m;
  p.Faces(m);ga_int(m.Extent());ga_flag(p.IsDone());pr_shape(p.Shape()))
 ga_end();}
