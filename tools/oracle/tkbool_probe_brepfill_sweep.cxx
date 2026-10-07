// BRepFill_Sweep probe operations (BRepFill_TrimShellCorner runs at the corners of the
// right and round transitions); the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_sweep.rs.
//
// The path and the section are shapes of a shape program (tktopalgo_probe_shapes.cxx);
// the result prints through pr_shape (tkprim_probe_shape.cxx) and the shapes of the tables
// through bl_indexed (tkbool_probe_brepfill_generator.cxx).
#include <BRepFill_ShapeLaw.hxx>
#include <BRepFill_Sweep.hxx>
// A table of the sweep against the sub-shapes of the result: -1 for a null table, else
// its bounds and each cell (bl_indexed).
void bl_sweep_table(const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& result,
                    const occ::handle<NCollection_HArray2<TopoDS_Shape>>& table){
 if(table.IsNull()){ga_int(-1);return;}
 ga_int(table->LowerRow());ga_int(table->UpperRow());ga_int(table->LowerCol());ga_int(table->UpperCol());
 for(int i=table->LowerRow();i<=table->UpperRow();++i)
  for(int j=table->LowerCol();j<=table->UpperCol();++j)bl_indexed(result,table->Value(i,j));}
// `program path section law... transition kpart bounds tol3d angmin angmax c1`: a
// BRepFill_Sweep of a BRepFill_ShapeLaw of the vertex or wire `section` along the location
// law (bl_location_law_of) with SetTolerance(tol3d), SetAngularControl(angmin, angmax),
// SetForceApproxC1(c1) and (bounds 1: the section first, 2: also last, 3: last only)
// SetBounds; Build with the transition (0 modified, 1 right, 2 round) and the default
// approximation; prints IsDone, ErrorOnSurface, the result, SubShape, InterFaces and
// Sections, the child count of each tape and the extents of the reversed edges, the tapes
// and the rails.
void bl_bf_sweep_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Wire path=TopoDS::Wire(t.shapes.at(r.integer()));TopoDS_Shape section=t.shapes.at(r.integer());
 occ::handle<BRepFill_LocationLaw> loc;occ::handle<BRepFill_ShapeLaw> sec;
 try{loc=bl_location_law_of(r,t,path);
  if(section.ShapeType()==TopAbs_VERTEX)sec=new BRepFill_ShapeLaw(TopoDS::Vertex(section));
  else sec=new BRepFill_ShapeLaw(TopoDS::Wire(section));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int transition=r.integer();bool kpart=r.flag();int bounds=r.integer();
 double tol3d=r.real(),angmin=r.real(),angmax=r.real();bool c1=r.flag();
 if(loc.IsNull()){ga_end();return;}
 BRepFill_Sweep sweep(sec,loc,kpart);
 sweep.SetTolerance(tol3d);sweep.SetAngularControl(angmin,angmax);sweep.SetForceApproxC1(c1);
 if(bounds>0&&section.ShapeType()==TopAbs_WIRE){
  TopoDS_Wire w=TopoDS::Wire(section),none;
  GA_TRY(sweep.SetBounds(bounds==3?none:w,bounds>=2?w:none))}
 NCollection_Map<TopoDS_Shape,TopTools_ShapeMapHasher> reversed;
 NCollection_DataMap<TopoDS_Shape,occ::handle<NCollection_HArray2<TopoDS_Shape>>,TopTools_ShapeMapHasher> tapes,rails;
 try{sweep.Build(reversed,tapes,rails,(BRepFill_TransitionStyle)transition);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(sweep.IsDone());ga_real(sweep.ErrorOnSurface());
 TopoDS_Shape result=sweep.Shape();
 GA_TRY(pr_shape(result))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 if(!result.IsNull())TopExp::MapShapes(result,map);
 GA_TRY(bl_sweep_table(map,sweep.SubShape()))
 GA_TRY(bl_sweep_table(map,sweep.InterFaces()))
 GA_TRY(bl_sweep_table(map,sweep.Sections()))
 if(!sec->IsVertex())for(int i=1;i<=sec->NbLaw();++i)GA_TRY(ga_int(sweep.Tape(i).NbChildren()))
 ga_int(reversed.Extent());ga_int(tapes.Extent());ga_int(rails.Extent());
 ga_end();}
