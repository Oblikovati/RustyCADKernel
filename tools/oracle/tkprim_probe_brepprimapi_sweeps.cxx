// BRepPrimAPI sweep and half-space probe operations; the Rust twin is
// crates/tkprim/examples/pr_probe/brep_prim_api_sweeps.rs.
//
// The generating shape is a shape program (tktopalgo_probe_shapes.cxx) and the index of
// the swept shape (pr_generating of tkprim_probe_brepsweep.cxx). Generated shapes print
// through pr_generated, against the sub-shapes of the maker's Shape().
#include <BRepPrimAPI_MakeHalfSpace.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
// A list of generated shapes: its extent, then each shape.
void pr_pa_list(const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& result,const NCollection_List<TopoDS_Shape>& l){
 ga_int(l.Extent());for(const TopoDS_Shape& s:l)pr_generated(result,s);}
// The history of a sweep maker: IsDone, Shape, FirstShape, LastShape, then for every shape
// of the program Generated, Modified (the base answer), IsDeleted, FirstShape(S) and
// LastShape(S).
template<class M> void pr_pa_sweep_history(M& m,const TpShapes& t){
 ga_flag(m.IsDone());
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> result;TopoDS_Shape shape;
 GA_TRY(shape=m.Shape();pr_shape(shape))
 if(!shape.IsNull())TopExp::MapShapes(shape,result);
 GA_TRY(pr_generated(result,m.FirstShape()))
 GA_TRY(pr_generated(result,m.LastShape()))
 for(const TopoDS_Shape& s:t.shapes){
  GA_TRY(pr_pa_list(result,m.Generated(s)))
  GA_TRY(ga_int(m.Modified(s).Extent()))
  GA_TRY(ga_flag(m.IsDeleted(s)))
  GA_TRY(pr_generated(result,m.FirstShape(s)))
  GA_TRY(pr_generated(result,m.LastShape(s)))}}
// `program index kind ...`: kind 0 `V copy canonize` (MakePrism(S, V, Copy, Canonize)),
// 1 `D inf copy canonize` (MakePrism(S, D, Inf, Copy, Canonize)); prints Prism().Vec()
// and the history.
void pr_pa_prism_case(std::istream& input){
 GaReader r{input};TpShapes t;TopoDS_Shape s;
 try{s=pr_generating(r,t);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int kind=r.integer();std::unique_ptr<BRepPrimAPI_MakePrism> prism;
 try{
  if(kind==0){gp_Vec v(r.xyz());bool copy=r.flag();prism.reset(new BRepPrimAPI_MakePrism(s,v,copy,r.flag()));}
  else{gp_Dir d=r.dir();bool inf=r.flag(),copy=r.flag();prism.reset(new BRepPrimAPI_MakePrism(s,d,inf,copy,r.flag()));}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_xyz(prism->Prism().Vec().XYZ());pr_pa_sweep_history(*prism,t);
 ga_end();}
// `program index kind P D ...`: kind 0 `angle copy` (MakeRevol(S, gp_Ax1(P, D), angle,
// Copy)), 1 `copy` (MakeRevol(S, gp_Ax1(P, D), Copy)); prints the axis and angle of
// Revol(), HasDegenerated, Degenerated and the history.
void pr_pa_revol_case(std::istream& input){
 GaReader r{input};TpShapes t;TopoDS_Shape s;
 try{s=pr_generating(r,t);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int kind=r.integer();gp_Pnt p=r.pnt();gp_Dir d=r.dir();std::unique_ptr<BRepPrimAPI_MakeRevol> revol;
 try{
  if(kind==0){double angle=r.real();revol.reset(new BRepPrimAPI_MakeRevol(s,gp_Ax1(p,d),angle,r.flag()));}
  else revol.reset(new BRepPrimAPI_MakeRevol(s,gp_Ax1(p,d),r.flag()));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_pnt(revol->Revol().Axe().Location());ga_dir(revol->Revol().Axe().Direction());ga_real(revol->Revol().Angle());
 ga_flag(revol->HasDegenerated());
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> result;
 if(revol->IsDone())TopExp::MapShapes(revol->Shape(),result);
 pr_pa_list(result,revol->Degenerated());
 pr_pa_sweep_history(*revol,t);
 ga_end();}
// `program index kind P`: MakeHalfSpace(TopoDS::Face(S), P) for kind 0, else
// MakeHalfSpace(TopoDS::Shell(S), P); prints IsDone, Solid and Shape.
void pr_pa_halfspace_case(std::istream& input){
 GaReader r{input};TpShapes t;TopoDS_Shape s;
 try{s=pr_generating(r,t);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int kind=r.integer();gp_Pnt p=r.pnt();std::unique_ptr<BRepPrimAPI_MakeHalfSpace> h;
 try{
  if(kind==0)h.reset(new BRepPrimAPI_MakeHalfSpace(TopoDS::Face(s),p));
  else h.reset(new BRepPrimAPI_MakeHalfSpace(TopoDS::Shell(s),p));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(h->IsDone());
 GA_TRY(pr_shape(h->Solid()))
 GA_TRY(pr_shape(h->Shape()))
 ga_end();}
