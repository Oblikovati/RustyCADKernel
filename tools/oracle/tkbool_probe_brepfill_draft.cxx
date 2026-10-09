// BRepFill_Draft probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_draft.rs.
//
// The drafted shape and the stop shape are shapes of a shape program
// (tktopalgo_probe_shapes.cxx); shapes print through pr_shape (tkprim_probe_shape.cxx)
// and generated shapes through bl_list (tkbool_probe_brepfill_generator.cxx).
#include <BRepFill_Draft.hxx>
#include <Geom_Plane.hxx>
// `program shape dir angle style angmin angmax internal mode ...`: a BRepFill_Draft of the
// wire, face or shell with SetOptions and SetDraft, then by `mode` 0 `length`
// Perform(LengthMax), 1 `point normal keep` Perform(Geom_Plane, KeepInsideSurface),
// 2 `stop keep` Perform(StopShape, KeepOutSide); prints IsDone, the shape, the shell and,
// when done, Generated of every shape of the program (the tables are null natively
// otherwise, a crash).
void bl_bf_draft_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Shape s=t.shapes.at(r.integer());gp_Dir dir=r.dir();double angle=r.real();
 int style=r.integer();double angmin=r.real(),angmax=r.real();bool internal=r.flag();
 std::unique_ptr<BRepFill_Draft> draft;
 try{draft.reset(new BRepFill_Draft(s,dir,angle));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 draft->SetOptions((BRepFill_TransitionStyle)style,angmin,angmax);draft->SetDraft(internal);
 int mode=r.integer();
 try{
  if(mode==0)draft->Perform(r.real());
  else if(mode==1){gp_Pnt p=r.pnt();gp_Dir n=r.dir();bool keep=r.flag();draft->Perform(new Geom_Plane(p,n),keep);}
  else{const TopoDS_Shape& stop=t.shapes.at(r.integer());draft->Perform(stop,r.flag());}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(draft->IsDone());
 GA_TRY(pr_shape(draft->Shape()))
 GA_TRY(pr_shape(draft->Shell()))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 if(!draft->Shape().IsNull())TopExp::MapShapes(draft->Shape(),map);
 if(draft->IsDone())for(const TopoDS_Shape& g:t.shapes)GA_TRY(bl_list(map,draft->Generated(g)))
 ga_end();}
