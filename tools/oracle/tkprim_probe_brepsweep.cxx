// BRepSweep probe operations; the Rust twin is crates/tkprim/examples/pr_probe/brep_sweep.rs.
//
// The generating shape is a shape program (tktopalgo_probe_shapes.cxx) followed by the
// index of the shape to sweep. Sweep records:
// - prism `0 V copy canonize` (BRepSweep_Prism(S, V, Copy, Canonize)) or
//   `1 D inf copy canonize` (BRepSweep_Prism(S, D, Inf, Copy, Canonize));
// - revol `0 P D angle copy` (BRepSweep_Revol(S, gp_Ax1(P, D), angle, C)) or
//   `1 P D copy` (the complete revolution).
// A generated shape prints as its index among the sub-shapes of the sweep result
// (TopExp::MapShapes, IsSame) with its orientation, or as its pr_shape digest when it is
// not one of them.
#include <BRepSweep_Builder.hxx>
#include <BRepSweep_Iterator.hxx>
#include <BRepSweep_Prism.hxx>
#include <BRepSweep_Revol.hxx>
#include <BRepSweep_Rotation.hxx>
#include <BRepSweep_Tool.hxx>
#include <BRepSweep_Translation.hxx>
#include <gp_Ax1.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <TopExp.hxx>
#include <type_traits>
// A generated shape: -1 for null, else its index in `result` (0 when absent, then its
// digest) and its orientation.
void pr_generated(const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& result,const TopoDS_Shape& s){
 if(s.IsNull()){ga_int(-1);return;}
 int i=result.FindIndex(s);ga_int(i);ga_int(s.Orientation());
 if(i==0)pr_shape(s);}
// The generating shape of a request: a shape program and the index of the swept shape.
TopoDS_Shape pr_generating(GaReader& r,TpShapes& t){t=tp_shapes(r);return t.shapes.at(r.integer());}
// The history queries of a sweep on every shape of the program (the swept shape's
// sub-shapes and the foreign ones): IsUsed, [GenIsUsed,] Shape, FirstShape, LastShape.
template<class S> void pr_sweep_queries(S& sweep,const TpShapes& t){
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> result;TopoDS_Shape shape;
 GA_TRY(shape=sweep.Shape();pr_shape(shape))
 if(!shape.IsNull())TopExp::MapShapes(shape,result);
 GA_TRY(pr_generated(result,sweep.FirstShape()))
 GA_TRY(pr_generated(result,sweep.LastShape()))
 for(const TopoDS_Shape& s:t.shapes){
  GA_TRY(ga_flag(sweep.IsUsed(s)))
  if constexpr(std::is_same_v<S,BRepSweep_Prism>)GA_TRY(ga_flag(sweep.GenIsUsed(s)))
  GA_TRY(pr_generated(result,sweep.Shape(s)))
  GA_TRY(pr_generated(result,sweep.FirstShape(s)))
  GA_TRY(pr_generated(result,sweep.LastShape(s)))}}
// `program index kind ...`: a prism, its vector and the history queries.
void pr_bs_prism_case(std::istream& input){
 GaReader r{input};TpShapes t;TopoDS_Shape s;
 try{s=pr_generating(r,t);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int kind=r.integer();std::unique_ptr<BRepSweep_Prism> prism;
 try{
  if(kind==0){gp_Vec v(r.xyz());bool copy=r.flag();prism.reset(new BRepSweep_Prism(s,v,copy,r.flag()));}
  else{gp_Dir d=r.dir();bool inf=r.flag(),copy=r.flag();prism.reset(new BRepSweep_Prism(s,d,inf,copy,r.flag()));}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_xyz(prism->Vec().XYZ());pr_sweep_queries(*prism,t);
 ga_end();}
// `program index kind ...`: a revolution, its axis and angle and the history queries.
void pr_bs_revol_case(std::istream& input){
 GaReader r{input};TpShapes t;TopoDS_Shape s;
 try{s=pr_generating(r,t);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int kind=r.integer();gp_Pnt p=r.pnt();gp_Dir d=r.dir();std::unique_ptr<BRepSweep_Revol> revol;
 try{
  if(kind==0){double angle=r.real();revol.reset(new BRepSweep_Revol(s,gp_Ax1(p,d),angle,r.flag()));}
  else revol.reset(new BRepSweep_Revol(s,gp_Ax1(p,d),r.flag()));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_pnt(revol->Axe().Location());ga_dir(revol->Axe().Direction());ga_real(revol->Angle());
 pr_sweep_queries(*revol,t);
 ga_end();}
