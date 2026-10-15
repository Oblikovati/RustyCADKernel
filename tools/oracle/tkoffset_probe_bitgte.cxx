// BiTgte_Blend probe operations; the Rust twin is crates/tkoffset/examples/of_probe/bitgte.rs.
//
// The shapes are primitives (ot_primitive of tkoffset_probe_brepoffset_tool.cxx) or the
// two cylindrical faces of OCC29861; the result prints through om_digest
// (tkoffset_probe_brepoffset_make_offset.cxx) and the blend surfaces as sorted records,
// since the order of the center lines follows hashed maps (shape addresses natively).
#include <BiTgte_Blend.hxx>
#include <BRepLib_MakeFace.hxx>
#include <Geom_CylindricalSurface.hxx>
// The shape of a request: `0 prim` (a primitive) or `1` (the compound of the two
// cylindrical faces of OCC29861).
static TopoDS_Shape obt_shape(GaReader& r){
 if(r.integer()==0)return ot_primitive(r);
 occ::handle<Geom_CylindricalSurface> s1=new Geom_CylindricalSurface(gp_Ax3(gp_Pnt(0,360,0),gp_Dir(0,0,1),gp_Dir(0,1,0)),160);
 occ::handle<Geom_CylindricalSurface> s2=new Geom_CylindricalSurface(gp_Ax3(gp_Pnt(0,0,0),gp_Dir(0,1,0)),80);
 TopoDS_Face f1=BRepLib_MakeFace(s1,0,2.*M_PI,-113.2000012,113.2000012,1e-7).Face();
 TopoDS_Face f2=BRepLib_MakeFace(s2,0,2.*M_PI,168.4,600,1e-7).Face();
 BRep_Builder b;TopoDS_Compound c;b.MakeCompound(c);b.Add(c,f1);b.Add(c,f2);return c;}
// The sorted records of the blend surfaces: contact type (-1 when ContactType raises, as
// for the cut center lines without ascendants), surface type, rounded area and edge count
// of each face.
static void obt_surfaces(const BiTgte_Blend& blend){
 std::vector<std::vector<long long>> records;
 for(int i=1;i<=blend.NbSurfaces();i++){
  const TopoDS_Face& f=blend.Face(i);GProp_GProps p;BRepGProp::SurfaceProperties(f,p);
  BRepAdaptor_Surface a(f,false);
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> em;TopExp::MapShapes(f,TopAbs_EDGE,em);
  long long contact=-1;try{contact=blend.ContactType(i);}catch(const Standard_Failure&){}
  records.push_back({contact,(long long)a.GetType(),om_round(p.Mass()),(long long)em.Extent()});}
 std::sort(records.begin(),records.end());
 for(const auto& rec:records)for(long long v:rec)std::cout<<v<<' ';}
// `obt_blend shape radius tol build npairs (f1 f2)... nedges (e)... nstops (f)...`: a
// BiTgte_Blend of the shape with the support face pairs, edges and stop faces (explorer
// indices), performed with or without the shape; prints the exception or IsDone, the
// number of blend surfaces and their sorted records, the number of branches with their
// sorted sizes, and the digest of the result.
void of_obt_blend_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape s=obt_shape(r);double radius=r.real(),tol=r.real();bool build=r.flag();
  BiTgte_Blend blend;blend.Init(s,radius,tol,false);
  int n=r.integer();
  for(int i=0;i<n;i++){TopoDS_Face f1=TopoDS::Face(ot_sub(s,TopAbs_FACE,r.integer()));
   TopoDS_Face f2=TopoDS::Face(ot_sub(s,TopAbs_FACE,r.integer()));blend.SetFaces(f1,f2);}
  n=r.integer();
  for(int i=0;i<n;i++)blend.SetEdge(TopoDS::Edge(ot_sub(s,TopAbs_EDGE,r.integer())));
  n=r.integer();
  for(int i=0;i<n;i++)blend.SetStoppingFace(TopoDS::Face(ot_sub(s,TopAbs_FACE,r.integer())));
  blend.Perform(build);
  ga_flag(blend.IsDone());ga_int(blend.NbSurfaces());obt_surfaces(blend);
  int nb=blend.NbBranches();ga_int(nb);
  std::vector<int> sizes;
  for(int i=1;i<=nb;i++){int from,to;blend.IndicesOfBranche(i,from,to);sizes.push_back(to-from+1);}
  std::sort(sizes.begin(),sizes.end());for(int v:sizes)ga_int(v);
  om_digest(blend.Shape());}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
