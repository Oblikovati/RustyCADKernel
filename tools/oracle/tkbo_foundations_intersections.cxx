// Complete executable IntTools point/common-part/curve member coverage at 3d097a0.
#include <IntTools_PntOnFace.hxx>
#include <IntTools_PntOn2Faces.hxx>
#include <IntTools_CommonPrt.hxx>
#include <IntTools_Curve.hxx>
#include <BRep_Builder.hxx>
#include <Geom_Line.hxx>
#include <Geom_Circle.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <Geom2d_Line.hxx>
#include <limits>
void bo_point3(gp_Pnt p){bo_real(p.X());bo_real(p.Y());bo_real(p.Z());}
void bo_face_point_state(const IntTools_PntOnFace& value) {
 bo_int(value.Valid());bo_int(value.Face().IsNull());bo_point3(value.Pnt());double u,v;value.Parameters(u,v);bo_real(u);bo_real(v);
}
void bo_two_points_state(const IntTools_PntOn2Faces& value){bo_int(value.IsValid());bo_face_point_state(value.P1());bo_face_point_state(value.P2());}
void bo_points_case(const std::vector<double>& v) {
 IntTools_PntOnFace first;bo_face_point_state(first);TopoDS_Face face;BRep_Builder().MakeFace(face);
 first.Init(face,gp_Pnt(v[0],v[1],v[2]),v[3],v[4]);bo_face_point_state(first);bo_int(first.Face().IsSame(face));
 first.SetValid(v[5]!=0.);first.SetFace(TopoDS_Face());first.SetPnt(gp_Pnt(v[2],v[1],v[0]));first.SetParameters(v[4],v[3]);bo_face_point_state(first);
 IntTools_PntOnFace second;bo_two_points_state(IntTools_PntOn2Faces());IntTools_PntOn2Faces pair(first,second);bo_two_points_state(pair);
 pair.SetP1(second);pair.SetP2(first);pair.SetValid(true);bo_two_points_state(pair);
}
void bo_common_state(const IntTools_CommonPrt& part) {
 bo_int(part.Edge1().IsNull());bo_int(part.Edge2().IsNull());bo_int(part.Type());bo_range(part.Range1());double first,last;part.Range1(first,last);bo_real(first);bo_real(last);
 bo_real(part.VertexParameter1());bo_real(part.VertexParameter2());bo_int(part.AllNullFlag());gp_Pnt p,q;part.BoundingPoints(p,q);bo_point3(p);bo_point3(q);
 bo_int(part.Ranges2().Length());for(const auto& range:part.Ranges2())bo_range(range);
}
void bo_configure_common(IntTools_CommonPrt& part,const std::vector<double>& v) {
 bo_common_state(part);TopoDS_Edge edge;BRep_Builder().MakeEdge(edge);part.SetEdge1(edge);part.SetEdge2(edge);bo_int(part.Edge1().IsSame(edge));bo_int(part.Edge2().IsSame(edge));
 part.SetType(TopAbs_EDGE);part.SetRange1(IntTools_Range(v[0],v[1]));bo_common_state(part);part.SetRange1(v[2],v[3]);
 part.AppendRange2(IntTools_Range(v[0],v[2]));part.AppendRange2(v[1],v[3]);part.SetVertexParameter1(v[4]);part.SetVertexParameter2(v[5]);part.SetAllNullFlag(true);
 part.SetBoundingPoints(gp_Pnt(v[0],v[1],v[2]),gp_Pnt(v[3],v[4],v[5]));
}
void bo_common_case(const std::vector<double>& v) {
 IntTools_CommonPrt part;bo_configure_common(part,v);bo_common_state(part);IntTools_CommonPrt clone(part);bo_common_state(clone);
 clone.Assign(part);bo_common_state(clone);clone=part;bo_common_state(clone);
 IntTools_CommonPrt target;target.AppendRange2(-9.,-8.);part.Copy(target);bo_common_state(target);target.ChangeRanges2().Clear();bo_common_state(target);
}
occ::handle<Geom_Curve> bo_curve_geometry(int kind) {
 occ::handle<Geom_Curve> line=new Geom_Line(gp_Ax1());
 if(kind==1)return new Geom_TrimmedCurve(line,2.,4.);
 if(kind==2)return new Geom_Circle(gp_Ax2(),2.);
 if(kind==3){NCollection_Array1<gp_Pnt> poles(1,3);poles(1)=gp_Pnt(0,0,0);poles(2)=gp_Pnt(1,2,3);poles(3)=gp_Pnt(4,5,6);return new Geom_BezierCurve(poles);}
 return line;
}
void bo_curve_emit(const IntTools_Curve& curve,double parameter) {
 bo_int(curve.HasBounds());bo_real(curve.Tolerance());bo_real(curve.TangentialTolerance());bo_int(curve.Type());
 double first=31.,last=32.;gp_Pnt p(8,9,10),q(11,12,13);bo_int(curve.Bounds(first,last,p,q));bo_real(first);bo_real(last);bo_point3(p);bo_point3(q);
 bo_int(curve.D0(parameter,p));bo_point3(p);bo_int(curve.D0(std::numeric_limits<double>::quiet_NaN(),p));bo_point3(p);
}
void bo_intersection_curve_case(const std::vector<double>& v) {
 bo_int(IntTools_Curve().HasBounds());auto geom=bo_curve_geometry(int(v[0]));occ::handle<Geom2d_Curve> pcurve=new Geom2d_Line(gp_Ax2d());
 IntTools_Curve curve(geom,pcurve,pcurve,v[2],v[3]);bo_curve_emit(curve,v[1]);bo_int(curve.Curve()==geom);bo_int(curve.FirstCurve2d()==pcurve);bo_int(curve.SecondCurve2d()==pcurve);
 curve.SetCurves(nullptr,nullptr,nullptr);bo_int(curve.Curve().IsNull());curve.SetCurve(geom);curve.SetFirstCurve2d(pcurve);curve.SetSecondCurve2d(pcurve);
 curve.SetTolerance(v[3]);curve.SetTangentialTolerance(v[2]);bo_curve_emit(curve,v[1]);
}
