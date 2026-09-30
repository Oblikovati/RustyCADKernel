// BRepSweep member probe operations (the Translation and Rotation classes called
// directly, and the Tool, Iterator and Builder); the Rust twin is
// crates/tkprim/examples/pr_probe/brep_sweep_members.rs.
//
// `bs_members kind program index nsh(index type closed beginf endinf) params copy`:
// kind 0 a BRepSweep_Translation of vector `params` = V (location the translation by V)
// with a canonize flag after `copy`; kind 1 a BRepSweep_Rotation of `params` = P D angle
// (location the rotation by the angle). Then Closed, the vector or the axis and angle,
// the Shape() digest, HasShape and IsInvariant of every program shape at every
// directing shape, the empty shapes of the swept shape's sub-shapes, the add tests of
// the side faces and SplitShell of the result.
// The directing shapes of a sweep: its tool's shapes 1..NbShapes.
template<class S> std::vector<Sweep_NumShape> pr_dir_shapes(const Sweep_NumShape& n){
 Sweep_NumShapeTool tool(n);std::vector<Sweep_NumShape> dirs;
 for(int i=1;i<=tool.NbShapes();++i)dirs.push_back(tool.Shape(i));
 return dirs;}
// The empty shapes of a generating sub-shape at every directing shape.
template<class S> void pr_empty_shapes(S& sweep,const TopoDS_Shape& s,const std::vector<Sweep_NumShape>& dirs){
 for(const Sweep_NumShape& d:dirs){
  TopAbs_ShapeEnum st=s.ShapeType();bool edge=d.Type()==TopAbs_EDGE;
  if(st==TopAbs_VERTEX&&!edge)GA_TRY(pr_shape(sweep.MakeEmptyVertex(s,d)))
  if(st==TopAbs_VERTEX&&edge)GA_TRY(pr_shape(sweep.MakeEmptyDirectingEdge(s,d)))
  if(st==TopAbs_EDGE&&!edge)GA_TRY(pr_shape(sweep.MakeEmptyGeneratingEdge(s,d)))
  if((st==TopAbs_EDGE&&edge)||(st==TopAbs_FACE&&!edge))GA_TRY(pr_shape(sweep.MakeEmptyFace(s,d)))
  if(st==TopAbs_FACE&&edge)GA_TRY(ga_int(sweep.DirectSolid(s,d)))}}
// The add tests of the side face of a generating edge (with its vertices and the
// directing vertices) and of the solid of a generating face.
template<class S> void pr_add_tests(S& sweep,const TopoDS_Shape& s,const std::vector<Sweep_NumShape>& dirs){
 const Sweep_NumShape& e=dirs.front();
 if(!sweep.HasShape(s,e))return;
 TopoDS_Shape side=sweep.Shape(s,e);
 for(TopoDS_Iterator it(s);it.More();it.Next())if(sweep.HasShape(it.Value(),e)){
  TopoDS_Shape sub=sweep.Shape(it.Value(),e);
  ga_flag(sweep.GGDShapeIsToAdd(side,sub,s,it.Value(),e));ga_flag(sweep.SeparatedWires(side,sub,s,it.Value(),e));}
 for(size_t k=1;k<dirs.size();++k)if(sweep.HasShape(s,dirs[k]))
  ga_flag(sweep.GDDShapeIsToAdd(side,sweep.Shape(s,dirs[k]),s,e,dirs[k]));}
template<class S> void pr_members(S& sweep,const TpShapes& t,const TopoDS_Shape& gen,const Sweep_NumShape& n){
 ga_flag(sweep.Closed());std::vector<Sweep_NumShape> dirs=pr_dir_shapes<S>(n);
 TopoDS_Shape shape;GA_TRY(shape=sweep.Shape();pr_shape(shape))
 for(const TopoDS_Shape& s:t.shapes){GA_TRY(ga_flag(sweep.IsInvariant(s)))
  for(const Sweep_NumShape& d:dirs)GA_TRY(ga_flag(sweep.HasShape(s,d)))}
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> subs;TopExp::MapShapes(gen,subs);
 for(int i=1;i<=subs.Extent();++i){pr_empty_shapes(sweep,subs(i),dirs);GA_TRY(pr_add_tests(sweep,subs(i),dirs))}
 if(!shape.IsNull())GA_TRY(pr_shape(sweep.SplitShell(shape)))}
Sweep_NumShape pr_dir_record(GaReader& r){
 int index=r.integer(),type=r.integer();bool closed=r.flag(),beg=r.flag();
 return Sweep_NumShape(index,TopAbs_ShapeEnum(type),closed,beg,r.flag());}
void pr_bs_members_case(std::istream& input){
 GaReader r{input};int kind=r.integer();TpShapes t;TopoDS_Shape s=pr_generating(r,t);Sweep_NumShape n=pr_dir_record(r);
 try{
  if(kind==0){gp_Vec v(r.xyz());bool copy=r.flag(),canonize=r.flag();gp_Trsf T;T.SetTranslation(v);
   BRepSweep_Translation sweep(s,n,TopLoc_Location(T),v,copy,canonize);ga_xyz(sweep.Vec().XYZ());pr_members(sweep,t,s,n);}
  else{gp_Pnt p=r.pnt();gp_Dir d=r.dir();double angle=r.real();bool copy=r.flag();gp_Trsf T;T.SetRotation(gp_Ax1(p,d),angle);
   BRepSweep_Rotation sweep(s,n,TopLoc_Location(T),gp_Ax1(p,d),angle,copy);ga_pnt(sweep.Axe().Location());ga_real(sweep.Angle());
   pr_members(sweep,t,s,n);}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
// `program index`: BRepSweep_Tool (NbShapes, Index of every program shape, Shape of 0 to
// NbShapes+1 as its program index, Type, Orientation, SetOrientation), BRepSweep_Iterator
// over the shape and BRepSweep_Builder containers with oriented adds.
void pr_bs_tools_case(std::istream& input){
 GaReader r{input};TpShapes t;TopoDS_Shape s=pr_generating(r,t);BRepSweep_Tool tool(s);
 ga_int(tool.NbShapes());for(const TopoDS_Shape& x:t.shapes)ga_int(tool.Index(x));
 for(int i=0;i<=tool.NbShapes()+1;++i)GA_TRY(TopoDS_Shape x=tool.Shape(i);ga_int(tp_shape_index(t,x));ga_int(tool.Type(x));ga_int(tool.Orientation(x)))
 TopoDS_Shape o=s;tool.SetOrientation(o,TopAbs_REVERSED);ga_int(o.Orientation());
 BRepSweep_Iterator it;ga_flag(it.More());
 for(it.Init(s);it.More();it.Next()){ga_int(tp_same_index(t,it.Value()));ga_int(it.Orientation());}
 BRepSweep_Builder B{BRep_Builder()};TopoDS_Shape c[5];
 GA_TRY(B.MakeCompound(c[0]);B.MakeCompSolid(c[1]);B.MakeSolid(c[2]);B.MakeShell(c[3]);B.MakeWire(c[4]);
  for(TopoDS_Shape& x:c)ga_int(x.ShapeType());
  B.Add(c[0],s,TopAbs_INTERNAL);B.Add(c[0],s);pr_shape(c[0]))
 GA_TRY(B.MakeWire(c[0]))
 ga_end();}
