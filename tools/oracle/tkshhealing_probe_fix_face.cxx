// ShapeFix_Face probe operation; the Rust twin is crates/tkshhealing/examples/sh_probe/fix_face.rs.
//
// `sh_fix_face (input) ctx prec maxtol op`: a ShapeFix_Face on the input face (with a new
// ShapeBuild_ReShape context when ctx, the precision and the maximal tolerance set), then
// op: 0 Perform; 1 FixOrientation; 2 FixAddNaturalBound; 3 FixMissingSeam;
// 4 FixSmallAreaWire(false); 5 FixSmallAreaWire(true); 6 FixIntersectingWires;
// 7 FixWiresTwoCoincEdges; 8 FixPeriodicDegenerated; 9 FixLoopWire (and the count and
// digests of its wires); 10 FixOrientation(map) then FixSplitFace(map); 11 to 14 Perform
// with FixWireMode, FixLoopWiresMode, FixSplitFaceMode or FixIntersectingWiresMode 0. Printed: the
// result flag, Status(DONE1..DONE8, FAIL1..FAIL3), the digests of Face() and Result().
// The input is
//  0 nw (m (point)*m)*nw (reversed)*nw: the face of the plane z = 0 bounded by the first
//    polygon (BRepBuilderAPI_MakeFace(plane, wire)), the other polygons added as wires
//    with BRep_Builder, each reversed when asked;
//  1 type angle params: the lateral face of a primitive of axis Z spanning `angle`:
//    type 0 cylinder R H, 1 cone R1 R2 H, 2 sphere R, 3 torus R1 R2;
//  2 n (edge)*n: the planar face (only plane) of a wire of edges (tkshhealing_probe_unify).
#include <BRepPrimAPI_MakeTorus.hxx>
#include <ShapeFix_Face.hxx>
#include <gp_Pln.hxx>
TopoDS_Wire sff_polygon(GaReader& r){
 int m=r.integer();BRepBuilderAPI_MakePolygon poly;for(int i=0;i<m;++i)poly.Add(r.pnt());poly.Close();return poly.Wire();}
TopoDS_Face sff_planar(GaReader& r){
 int nw=r.integer();std::vector<TopoDS_Wire> wires;for(int k=0;k<nw;++k)wires.push_back(sff_polygon(r));
 std::vector<bool> rev;for(int k=0;k<nw;++k)rev.push_back(r.flag());
 TopoDS_Wire outer=wires[0];if(rev[0])outer.Reverse();
 TopoDS_Face face=BRepBuilderAPI_MakeFace(gp_Pln(),outer,true).Face();BRep_Builder B;
 for(int k=1;k<nw;++k){TopoDS_Wire w=wires[k];if(rev[k])w.Reverse();B.Add(face,w);}
 return face;}
TopoDS_Face sff_lateral(GaReader& r){
 int type=r.integer();double angle=r.real();
 if(type==0){double R=r.real();return BRepPrimAPI_MakeCylinder(R,r.real(),angle).Face();}
 if(type==1){double R1=r.real(),R2=r.real();return BRepPrimAPI_MakeCone(R1,R2,r.real(),angle).Face();}
 if(type==2)return BRepPrimAPI_MakeSphere(r.real(),angle).Face();
 double R1=r.real();return BRepPrimAPI_MakeTorus(R1,r.real(),angle).Face();}
TopoDS_Face sff_input(GaReader& r){
 int kind=r.integer();
 if(kind==0)return sff_planar(r);
 if(kind==1)return sff_lateral(r);
 return BRepBuilderAPI_MakeFace(TopoDS::Wire(su_wire(r)),true).Face();}
void sff_status(const ShapeFix_Face& f){
 const ShapeExtend_Status s[11]={ShapeExtend_DONE1,ShapeExtend_DONE2,ShapeExtend_DONE3,ShapeExtend_DONE4,
  ShapeExtend_DONE5,ShapeExtend_DONE6,ShapeExtend_DONE7,ShapeExtend_DONE8,ShapeExtend_FAIL1,ShapeExtend_FAIL2,ShapeExtend_FAIL3};
 for(ShapeExtend_Status x:s)ga_flag(f.Status(x));}
void sff_run(ShapeFix_Face& f,int op){
 if(op==0){ga_flag(f.Perform());return;}
 if(op>=11&&op<=14){
  if(op==11)f.FixWireMode()=0;if(op==12)f.FixLoopWiresMode()=0;if(op==13)f.FixSplitFaceMode()=0;
  if(op==14)f.FixIntersectingWiresMode()=0;ga_flag(f.Perform());return;}
 if(op==1){ga_flag(f.FixOrientation());return;}
 if(op==2){ga_flag(f.FixAddNaturalBound());return;}
 if(op==3){ga_flag(f.FixMissingSeam());return;}
 if(op==4||op==5){ga_flag(f.FixSmallAreaWire(op==5));return;}
 if(op==6){ga_flag(f.FixIntersectingWires());return;}
 if(op==7){ga_flag(f.FixWiresTwoCoincEdges());return;}
 if(op==8){ga_flag(f.FixPeriodicDegenerated());return;}
 if(op==9){NCollection_Sequence<TopoDS_Shape> wires;ga_flag(f.FixLoopWire(wires));ga_int(wires.Length());
  for(int i=1;i<=wires.Length();++i)tp_shape(wires(i));return;}
 NCollection_DataMap<TopoDS_Shape,NCollection_List<TopoDS_Shape>,TopTools_ShapeMapHasher> map;
 ga_flag(f.FixOrientation(map));ga_int(map.Extent());ga_flag(f.FixSplitFace(map));}
void sh_fix_face_case(std::istream& input){
 GaReader r{input};TopoDS_Face face;GA_TRY(face=sff_input(r))
 bool ctx=r.flag();double prec=r.real(),maxtol=r.real();int op=r.integer();
 if(face.IsNull()){ga_end();return;}
 ShapeFix_Face f(face);
 if(ctx)f.SetContext(new ShapeBuild_ReShape);
 f.SetPrecision(prec);f.SetMaxTolerance(maxtol);
 GA_TRY(sff_run(f,op);sff_status(f);tp_shape(f.Face());tp_shape(f.Result()))
 ga_end();}
