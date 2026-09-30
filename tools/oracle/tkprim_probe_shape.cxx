// The shape digest of the TKPrim probes; the Rust twin is
// crates/tkprim/examples/pr_probe/shape.rs. Every TKPrim result is compared through the
// TKTopAlgo digest `tp_shape` (tktopalgo_probe_breplib_common.cxx): its sub-shapes in
// TopExp::MapShapes order with their types, orientations, flags, children and geometry.
#include <BRep_Builder.hxx>
#include <TopoDS_Vertex.hxx>
// `x y z tol`: the digest of a vertex built with BRep_Builder; it proves that both probes
// print the shared digest identically before any TKPrim class is compared.
void pr_digest_vertex_case(std::istream& input){
 GaReader r{input};gp_Pnt point=r.pnt();double tolerance=r.real();
 TopoDS_Vertex vertex;
 GA_TRY(BRep_Builder().MakeVertex(vertex,point,tolerance);tp_shape(vertex))
 ga_end();}
// `pr_shape`: the TKTopAlgo digest of a shape, then the first derivatives of the located
// surface of each face (in TopExp::MapShapes order) at (0.3, 0.7), which pins the frame
// of a planar or elementary surface beyond the single point of the digest.
#include <Geom_Surface.hxx>
#include <gp_Vec.hxx>
void pr_face_frames(const TopoDS_Shape& s){
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> faces;TopExp::MapShapes(s,TopAbs_FACE,faces);
 ga_int(faces.Extent());
 for(int i=1;i<=faces.Extent();++i){
  occ::handle<Geom_Surface> surface=BRep_Tool::Surface(TopoDS::Face(faces(i)));
  if(surface.IsNull()){ga_int(-1);continue;}
  gp_Pnt p;gp_Vec du,dv;GA_TRY(surface->D1(0.3,0.7,p,du,dv);ga_pnt(p);ga_xyz(du.XYZ());ga_xyz(dv.XYZ()))}}
void pr_shape(const TopoDS_Shape& s){tp_shape(s);if(!s.IsNull())pr_face_frames(s);}
