// BRepBuilderAPI_MakeShapeOnMesh probe operation; the Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_builder_api_on_mesh.rs.
#include <BRepBuilderAPI_MakeShapeOnMesh.hxx>
// `nt (x y z)... ntri (a b c)... defl` (nt < 0: a null mesh): the triangulation, sized
// first, so a mesh may have no node or no triangle.
occ::handle<Poly_Triangulation> tp_onmesh_mesh(GaReader& r){
 int nt=r.integer();if(nt<0)return occ::handle<Poly_Triangulation>();
 std::vector<gp_Pnt> nodes;for(int i=0;i<nt;++i)nodes.push_back(r.pnt());
 int ntri=r.integer();occ::handle<Poly_Triangulation> mesh=new Poly_Triangulation(nt,ntri,false);
 for(int i=1;i<=nt;++i)mesh->SetNode(i,nodes[i-1]);
 for(int i=1;i<=ntri;++i){int a=r.integer(),b=r.integer();mesh->SetTriangle(i,Poly_Triangle(a,b,r.integer()));}
 mesh->Deflection(r.real());return mesh;}
// `mesh builds`: Build() `builds` times (each exception printed), then tp_bba_made.
void tp_bba_onmesh_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Poly_Triangulation> mesh=tp_onmesh_mesh(r);int builds=r.integer();
  BRepBuilderAPI_MakeShapeOnMesh m(mesh);for(int i=0;i<builds;i++)GA_TRY(m.Build())
  tp_bba_made(m))
 ga_end();}
