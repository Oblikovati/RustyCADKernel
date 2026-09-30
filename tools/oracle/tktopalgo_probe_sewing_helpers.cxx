// Probe operations of the two sewing helpers, BRepBuilderAPI_VertexInspector and
// BRepBuilderAPI_BndBoxTreeSelector; the Rust twin is
// crates/tktopalgo/examples/tp_probe/sewing_helpers.rs.
//
// `bba_vertexinspector tol n (x y z)... (cx cy cz) nt (target)...`: the points are added
//   to an NCollection_CellFilter of cell size `tol` (target i for the i-th point, from 1)
//   and to the inspector; the filter inspects the box of the current point shifted by
//   -tol and +tol; prints the found indices, Coord(0..2) and Shift(+-tol) of the current
//   point, then clears the list and calls Inspect on each target (in range: the native
//   point vector is not checked) and prints the found indices again.
// `bba_boxselector n (box)... (box) nr (box)... na (object)...`, a box being `xmin ymin zmin
//   xmax ymax zmax`: the boxes fill an NCollection_UBTree through UBTreeFiller (object i
//   for the i-th box, from 1); the selector's current box is the next one; prints the
//   Select count and the found objects, Reject of each of the next boxes, then clears
//   the list and prints the results of Accept of each object and the list.
#include <BRepBuilderAPI_BndBoxTreeSelector.hxx>
#include <BRepBuilderAPI_VertexInspector.hxx>
#include <NCollection_UBTreeFiller.hxx>
void tp_int_list(const NCollection_List<int>& l){
 ga_int(l.Extent());for(NCollection_List<int>::Iterator it(l);it.More();it.Next())ga_int(it.Value());}
void tp_bba_vertexinspector_case(std::istream& input){
 GaReader r{input};double tol=r.real();int n=r.integer();
 NCollection_CellFilter<BRepBuilderAPI_VertexInspector> filter(tol);BRepBuilderAPI_VertexInspector inspector(tol);
 for(int i=1;i<=n;++i){gp_XYZ p=r.xyz();filter.Add(i,p);inspector.Add(p);}
 gp_XYZ current=r.xyz();inspector.SetCurrent(current);
 filter.Inspect(BRepBuilderAPI_VertexInspector::Shift(current,-tol),BRepBuilderAPI_VertexInspector::Shift(current,tol),inspector);
 tp_int_list(inspector.ResInd());
 for(int i=0;i<3;++i)ga_real(BRepBuilderAPI_VertexInspector::Coord(i,current));
 ga_xyz(BRepBuilderAPI_VertexInspector::Shift(current,tol));ga_xyz(BRepBuilderAPI_VertexInspector::Shift(current,-tol));
 inspector.ClearResList();int nt=r.integer();
 for(int i=0;i<nt;++i)ga_int(int(inspector.Inspect(r.integer())));
 tp_int_list(inspector.ResInd());
 ga_end();}
Bnd_Box tp_bba_box(GaReader& r){
 Bnd_Box b;double x0=r.real(),y0=r.real(),z0=r.real(),x1=r.real(),y1=r.real(),z1=r.real();
 b.Update(x0,y0,z0,x1,y1,z1);return b;}
void tp_bba_boxselector_case(std::istream& input){
 GaReader r{input};int n=r.integer();
 NCollection_UBTree<int,Bnd_Box> tree;
 {NCollection_UBTreeFiller<int,Bnd_Box> filler(tree);for(int i=1;i<=n;++i)filler.Add(i,tp_bba_box(r));filler.Fill();}
 BRepBuilderAPI_BndBoxTreeSelector selector;selector.SetCurrent(tp_bba_box(r));
 ga_int(tree.Select(selector));tp_int_list(selector.ResInd());
 int nr=r.integer();for(int i=0;i<nr;++i)ga_flag(selector.Reject(tp_bba_box(r)));
 selector.ClearResList();int na=r.integer();
 for(int i=0;i<na;++i)ga_flag(selector.Accept(r.integer()));
 tp_int_list(selector.ResInd());
 ga_end();}
