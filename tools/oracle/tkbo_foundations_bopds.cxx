// Calls each scalar BOPDS member and its native Dump at 3d097a0.
#include <BOPDS_IndexRange.hxx>
#include <BOPDS_Pave.hxx>
#include <BOPDS_Pair.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BOPDS_CommonBlock.hxx>
#include <BOPDS_CoupleOfPaveBlocks.hxx>
#include <BOPDS_ShapeInfo.hxx>
#include <BOPDS_FaceInfo.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_Curve.hxx>
#include <Standard_NoSuchObject.hxx>
#include <BOPDS_Point.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <Precision.hxx>
#include <gp_Lin.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <gp_Circ.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt2d.hxx>
#include <BRep_Builder.hxx>
#include <Bnd_Box.hxx>
#include <cstdio>
#include <unistd.h>
void bo_index_state(const BOPDS_IndexRange& value,int query) {
 bo_int(value.First());bo_int(value.Last());int first,last;value.Indices(first,last);bo_int(first);bo_int(last);
 bo_int(value.Contains(query));value.Dump();std::cout<<' ';
}
void bo_index_case(const std::vector<double>& v) {
 bo_index_state(BOPDS_IndexRange(),int(v[4]));BOPDS_IndexRange value{int(v[0]),int(v[1])};bo_index_state(value,int(v[4]));
 value.SetFirst(int(v[2]));value.SetLast(int(v[3]));bo_index_state(value,int(v[4]));
 value.SetIndices(int(v[1]),int(v[0]));bo_index_state(value,int(v[4]));
}
void bo_pave_state(const BOPDS_Pave& value) {
 bo_int(value.Index());bo_real(value.Parameter());int index;double parameter;value.Contents(index,parameter);bo_int(index);bo_real(parameter);
 std::cout<<std::hash<BOPDS_Pave>{}(value)<<' ';value.Dump();std::cout<<' ';
}
void bo_pave_case(const std::vector<double>& v) {
 bo_pave_state(BOPDS_Pave());BOPDS_Pave a(int(v[0]),v[1]),b(int(v[2]),v[3]);
 bo_pave_state(a);bo_pave_state(b);bo_int(a.IsEqual(b));bo_int(a==b);bo_int(a.IsLess(b));bo_int(b<a);
 a.SetIndex(int(v[2]));a.SetParameter(v[3]);bo_pave_state(a);bo_int(a.IsEqual(b));
}
void bo_pair_state(const BOPDS_Pair& value) {
 int a,b;value.Indices(a,b);bo_int(a);bo_int(b);std::cout<<std::hash<BOPDS_Pair>{}(value)<<' ';
}
void bo_pair_case(const std::vector<double>& v) {
 bo_pair_state(BOPDS_Pair());BOPDS_Pair a{int(v[0]),int(v[1])},b{int(v[2]),int(v[3])};
 bo_pair_state(a);bo_pair_state(b);bo_int(a.IsEqual(b));bo_int(a==b);bo_int(a<b);bo_int(b<a);
 a.SetIndices(int(v[3]),int(v[2]));bo_pair_state(a);bo_int(a==b);
}
void bo_point_state(const BOPDS_Point& value) {
 for(double coordinate:{value.Pnt().X(),value.Pnt().Y(),value.Pnt().Z(),value.Pnt2D1().X(),value.Pnt2D1().Y(),value.Pnt2D2().X(),value.Pnt2D2().Y()})bo_real(coordinate);
 bo_int(value.Index());
}
void bo_point_case(const std::vector<double>& v) {
 BOPDS_Point value;bo_point_state(value);value.SetPnt(gp_Pnt(v[0],v[1],v[2]));value.SetPnt2D1(gp_Pnt2d(v[3],v[4]));
 value.SetPnt2D2(gp_Pnt2d(v[5],v[6]));value.SetIndex(int(v[7]));bo_point_state(value);
}
void bo_pave_block_case(const std::vector<double>& v) {
 BOPDS_PaveBlock block;int edge=99;double first,last;int i1,i2;
 bo_int(block.Edge());bo_int(block.OriginalEdge());bo_int(block.HasEdge());bo_int(block.HasEdge(edge));bo_int(edge);
 block.SetEdge(int(v[0]));block.SetOriginalEdge(int(v[1]));block.SetPave1(BOPDS_Pave(int(v[2]),v[3]));block.SetPave2(BOPDS_Pave(int(v[4]),v[5]));
 bo_int(block.IsSplitEdge());block.Range(first,last);bo_real(first);bo_real(last);block.Indices(i1,i2);bo_int(i1);bo_int(i2);
 block.AppendExtPave(BOPDS_Pave(int(v[6]),v[7]));block.AppendExtPave(BOPDS_Pave(int(v[8]),v[9]));block.AppendExtPave(BOPDS_Pave(int(v[10]),v[11]));
 bo_int(block.ExtPaves().Extent());int found=-7;bo_int(block.ContainsParameter(v[12],v[13],found));bo_int(found);bo_int(block.IsToUpdate());
 NCollection_List<occ::handle<BOPDS_PaveBlock>> splits;block.Update(splits,v[14]!=0);bo_int(splits.Extent());
 for(NCollection_List<occ::handle<BOPDS_PaveBlock>>::Iterator it(splits);it.More();it.Next()) {
  const auto& split=it.Value();split->Range(first,last);split->Indices(i1,i2);bo_real(first);bo_real(last);bo_int(i1);bo_int(i2);bo_int(split->OriginalEdge());
 }
 Bnd_Box box;box.Update(v[15],v[16],v[17],v[18],v[19],v[20]);block.SetShrunkData(v[21],v[22],box,v[23]!=0);
 Bnd_Box result;bool splittable=false;block.ShrunkData(first,last,result,splittable);bo_int(block.HasShrunkData());bo_real(first);bo_real(last);bo_int(splittable);
 double xmin,ymin,zmin,xmax,ymax,zmax;result.Get(xmin,ymin,zmin,xmax,ymax,zmax);for(double x:{xmin,ymin,zmin,xmax,ymax,zmax})bo_real(x);
}
void bo_common_block_case(const std::vector<double>& v) {
 BOPDS_CommonBlock group;bo_real(group.Tolerance());try{group.PaveBlock1();bo_int(0);}catch(const Standard_NoSuchObject&){bo_int(1);}try{group.Edge();bo_int(0);}catch(const Standard_NoSuchObject&){bo_int(1);}bo_int(group.PaveBlocks().Extent());bo_int(group.Faces().Extent());
 occ::handle<BOPDS_PaveBlock> a=new BOPDS_PaveBlock(),b=new BOPDS_PaveBlock(),c=new BOPDS_PaveBlock();
 a->SetOriginalEdge(int(v[0]));a->SetEdge(int(v[1]));b->SetOriginalEdge(int(v[2]));b->SetEdge(int(v[3]));c->SetOriginalEdge(int(v[4]));c->SetEdge(int(v[5]));
 group.AddPaveBlock(a);group.AddPaveBlock(b);group.AddPaveBlock(c);
 bo_int(group.PaveBlocks().Extent());for(const auto& pb:group.PaveBlocks())bo_int(pb->OriginalEdge());
 bo_int(group.IsPaveBlockOnEdge(int(v[8])));bo_int(group.Contains(c));bo_int(group.IsPaveBlockOnFace(int(v[9])));
 group.SetEdge(int(v[10]));for(const auto& pb:group.PaveBlocks())bo_int(pb->Edge());bo_int(group.Edge());
 group.SetRealPaveBlock(c);bo_int(group.PaveBlock1()==c);for(const auto& pb:group.PaveBlocks())bo_int(pb->OriginalEdge());
 NCollection_List<int> faces;faces.Append(int(v[6]));faces.Append(int(v[7]));group.SetFaces(faces);
 NCollection_List<int> appended;appended.Append(int(v[11]));appended.Append(int(v[12]));group.AppendFaces(appended);
 bo_int(appended.Extent());bo_int(group.Faces().Extent());for(int face:group.Faces())bo_int(face);
 bo_int(group.Contains(int(v[11])));bo_int(group.IsPaveBlockOnFace(int(v[9])));
 auto& selected=group.PaveBlockOnEdge(int(v[8]));bo_int(selected==c);selected->SetEdge(int(v[13]));bo_int(c->Edge());
 group.SetTolerance(v[14]);bo_real(group.Tolerance());std::fflush(stdout);FILE* capture=std::tmpfile();int saved=dup(STDOUT_FILENO);dup2(fileno(capture),STDOUT_FILENO);group.Dump();std::fflush(stdout);dup2(saved,STDOUT_FILENO);close(saved);std::rewind(capture);int character;while((character=std::fgetc(capture))!=EOF){if(character=='\n')std::cout<<"\\n";else std::cout<<char(character);}std::fclose(capture);
}
void bo_couple_of_pave_blocks_case(const std::vector<double>& v) {
 BOPDS_CoupleOfPaveBlocks empty;bo_int(empty.Index());bo_int(empty.IndexInterf());bo_real(empty.Tolerance());bo_int(empty.PaveBlock1().IsNull());bo_int(empty.PaveBlock2().IsNull());
 occ::handle<BOPDS_PaveBlock> a=new BOPDS_PaveBlock(),b=new BOPDS_PaveBlock();a->SetOriginalEdge(int(v[0]));b->SetOriginalEdge(int(v[1]));
 BOPDS_CoupleOfPaveBlocks value(a,b);bo_int(value.PaveBlock1()==a);bo_int(value.PaveBlock2()==b);
 value.SetIndex(int(v[2]));value.SetIndexInterf(int(v[3]));value.SetTolerance(v[4]);
 bo_int(value.Index());bo_int(value.IndexInterf());bo_real(value.Tolerance());
 occ::handle<BOPDS_PaveBlock> first,second;value.PaveBlocks(first,second);bo_int(first==a);bo_int(second==b);
 value.SetPaveBlock1(b);value.SetPaveBlock2(occ::handle<BOPDS_PaveBlock>());bo_int(value.PaveBlock1()==b);bo_int(value.PaveBlock2().IsNull());
 value.SetPaveBlocks(a,b);bo_int(value.PaveBlock1()==a);bo_int(value.PaveBlock2()==b);
}
void bo_shape_info_case(const std::vector<double>& v) {
 BOPDS_ShapeInfo value;int flag=99;bo_int(value.ShapeType());bo_int(value.Shape().IsNull());bo_int(value.Box().IsVoid());bo_int(value.SubShapes().Extent());bo_int(value.Reference());bo_int(value.HasReference());bo_int(value.HasBRep());bo_int(value.IsInterfering());bo_int(value.HasFlag());bo_int(value.HasFlag(flag));bo_int(flag);
 value.SetShape(TopoDS_Shape());value.SetShapeType(TopAbs_ShapeEnum(int(v[0])));Bnd_Box box;box.Update(v[3],v[4],v[5],v[6],v[7],v[8]);value.SetBox(box);value.SetReference(int(v[1]));value.SetFlag(int(v[2]));for(int i=9;i<12;i++)value.ChangeSubShapes().Append(int(v[i]));value.ChangeBox().SetGap(v[12]);
 flag=99;bo_int(value.ShapeType());bo_int(value.Shape().IsNull());bo_int(value.Box().IsVoid());bo_int(value.SubShapes().Extent());for(int index:value.SubShapes())bo_int(index);bo_int(value.HasSubShape(int(v[9])));bo_int(value.HasSubShape(-999));bo_int(value.Reference());bo_int(value.HasReference());bo_int(value.HasBRep());bo_int(value.IsInterfering());bo_int(value.HasFlag());bo_int(value.HasFlag(flag));bo_int(flag);
 double xmin,ymin,zmin,xmax,ymax,zmax;value.Box().Get(xmin,ymin,zmin,xmax,ymax,zmax);for(double x:{xmin,ymin,zmin,xmax,ymax,zmax})bo_real(x);value.Dump();
}
void bo_face_info_case(const std::vector<double>&) {
 BOPDS_FaceInfo info;bo_int(info.Index());bo_int(info.PaveBlocksIn().Extent());bo_int(info.VerticesIn().Extent());bo_int(info.PaveBlocksOn().Extent());bo_int(info.VerticesOn().Extent());bo_int(info.PaveBlocksSc().Extent());bo_int(info.VerticesSc().Extent());
 occ::handle<BOPDS_PaveBlock> a=new BOPDS_PaveBlock(),b=new BOPDS_PaveBlock(),c=new BOPDS_PaveBlock();
 bo_int(info.ChangePaveBlocksIn().Add(a));bo_int(info.ChangePaveBlocksIn().Add(b));bo_int(info.ChangePaveBlocksIn().Add(a));
 bo_int(info.ChangeVerticesIn().Add(4));bo_int(info.ChangeVerticesIn().Add(9));bo_int(info.ChangeVerticesIn().Add(4));
 bo_int(info.ChangePaveBlocksOn().Add(b));bo_int(info.ChangeVerticesOn().Add(9));bo_int(info.ChangePaveBlocksSc().Add(c));bo_int(info.ChangePaveBlocksSc().Add(a));bo_int(info.ChangeVerticesSc().Add(6));bo_int(info.ChangeVerticesSc().Add(7));
 info.SetIndex(12);bo_int(info.Index());bo_int(info.PaveBlocksIn().Extent());bo_int(info.VerticesIn().Extent());bo_int(info.PaveBlocksOn().Extent());bo_int(info.VerticesOn().Extent());bo_int(info.PaveBlocksSc().Extent());bo_int(info.VerticesSc().Extent());
 bo_int(info.PaveBlocksIn().Contains(a));bo_int(info.PaveBlocksIn().Contains(c));bo_int(info.VerticesIn().Contains(9));
 info.Clear();bo_int(info.Index());bo_int(info.PaveBlocksIn().Extent());bo_int(info.VerticesIn().Extent());bo_int(info.PaveBlocksOn().Extent());bo_int(info.VerticesOn().Extent());bo_int(info.PaveBlocksSc().Extent());bo_int(info.VerticesSc().Extent());bo_int(info.PaveBlocksSc().Contains(c));bo_int(info.PaveBlocksSc().Contains(a));bo_int(info.VerticesSc().Contains(7));
}
void bo_data_structure_case(const std::vector<double>&) {
 BOPDS_DS ds;TopoDS_Shape first=BRepBuilderAPI_MakeVertex(gp_Pnt(1.,0.,0.)),second=BRepBuilderAPI_MakeVertex(gp_Pnt(2.,0.,0.));
 NCollection_List<TopoDS_Shape> arguments;arguments.Append(first);arguments.Append(second);ds.SetArguments(arguments);
 bo_int(ds.NbShapes());bo_int(ds.NbSourceShapes());bo_int(ds.NbRanges());bo_int(ds.Arguments().Extent());
 bo_int(ds.Append(first));bo_int(ds.Append(second));TopoDS_Shape reversed=first;reversed.Reverse();bo_int(ds.Append(reversed));
 bo_int(ds.NbShapes());bo_int(ds.NbSourceShapes());bo_int(ds.NbRanges());bo_int(ds.Rank(0));bo_int(ds.Rank(1));bo_int(ds.IsNewShape(0));bo_int(ds.IsNewShape(-1));
 bo_int(ds.Index(first));bo_int(ds.Index(reversed));bo_int(ds.Index(second));bo_int(ds.Shape(0).IsSame(first));bo_int(ds.ShapeInfo(0).Shape().IsSame(first));
 BOPDS_ShapeInfo info;info.SetShape(second);info.SetFlag(17);bo_int(ds.Append(info));bo_int(ds.ShapeInfo(3).Flag());ds.ChangeShapeInfo(3).SetFlag(21);bo_int(ds.ShapeInfo(3).Flag());
 ds.Clear();bo_int(ds.NbShapes());bo_int(ds.NbSourceShapes());bo_int(ds.NbRanges());bo_int(ds.Arguments().Extent());bo_int(ds.Index(first));
}
void bo_data_structure_common_block_case(const std::vector<double>&) {
 BOPDS_DS ds;occ::handle<BOPDS_PaveBlock> first=new BOPDS_PaveBlock(),real=new BOPDS_PaveBlock(),unrelated=new BOPDS_PaveBlock();
 first->SetOriginalEdge(8);real->SetOriginalEdge(3);unrelated->SetOriginalEdge(3);
 NCollection_List<occ::handle<BOPDS_PaveBlock>> blocks;blocks.Append(first);blocks.Append(real);
 occ::handle<BOPDS_CommonBlock> common=new BOPDS_CommonBlock();common->SetPaveBlocks(blocks);
 bo_int(ds.IsCommonBlock(first));bo_int(ds.CommonBlock(first).IsNull());bo_int(ds.RealPaveBlock(first)==first);bo_int(ds.IsCommonBlockOnEdge(first));
 ds.SetCommonBlock(first,common);ds.SetCommonBlock(real,common);
 bo_int(ds.IsCommonBlock(first));bo_int(ds.IsCommonBlock(real));bo_int(ds.IsCommonBlock(unrelated));bo_int(ds.CommonBlock(first)==common);
 bo_int(ds.RealPaveBlock(first)==real);bo_int(ds.IsCommonBlockOnEdge(first));
 NCollection_List<occ::handle<BOPDS_PaveBlock>> replacementBlocks;replacementBlocks.Append(first);
 occ::handle<BOPDS_CommonBlock> replacement=new BOPDS_CommonBlock();replacement->SetPaveBlocks(replacementBlocks);ds.SetCommonBlock(first,replacement);
 bo_int(ds.CommonBlock(first)==replacement);bo_int(ds.RealPaveBlock(first)==first);bo_int(ds.IsCommonBlockOnEdge(first));
 ds.Clear();bo_int(ds.IsCommonBlock(first));
}
void bo_bopds_curve_case(const std::vector<double>&) {
 BOPDS_Curve value;bo_real(value.Tolerance());bo_real(value.TangentialTolerance());bo_int(value.Box().IsVoid());bo_int(value.PaveBlocks().Extent());bo_int(value.TechnoVertices().Extent());bo_int(value.HasEdge());
 try {value.ChangePaveBlock1();bo_int(0);} catch(const Standard_NoSuchObject&) {bo_int(1);}
 value.InitPaveBlock1();value.InitPaveBlock1();bo_int(value.PaveBlocks().Extent());value.ChangePaveBlock1()->SetEdge(12);bo_int(value.HasEdge());
 IntTools_Curve curve;curve.SetTangentialTolerance(4.5e-5);value.SetCurve(curve);bo_real(value.TangentialTolerance());
 Bnd_Box box;box.Update(-1.,-2.,-3.,4.,5.,6.);box.SetGap(0.125);value.SetBox(box);box.SetGap(9.);bo_real(value.Box().GetGap());value.ChangeBox().SetGap(0.25);bo_real(value.Box().GetGap());double x0,y0,z0,x1,y1,z1;value.Box().Get(x0,y0,z0,x1,y1,z1);for(double x:{x0,y0,z0,x1,y1,z1})bo_real(x);
 occ::handle<BOPDS_PaveBlock> first=new BOPDS_PaveBlock(),edge=new BOPDS_PaveBlock();edge->SetEdge(7);NCollection_List<occ::handle<BOPDS_PaveBlock>> blocks;blocks.Append(first);blocks.Append(edge);blocks.Append(edge);value.SetPaveBlocks(blocks);bo_int(value.PaveBlocks().Extent());bo_int(value.PaveBlocks().First()==first);NCollection_List<occ::handle<BOPDS_PaveBlock>>::Iterator aIt(value.PaveBlocks());aIt.Next();bo_int(aIt.Value()==edge);aIt.Next();bo_int(aIt.Value()==edge);bo_int(value.HasEdge());value.ChangePaveBlocks().RemoveFirst();bo_int(value.PaveBlocks().Extent());
 value.ChangeTechnoVertices().Append(8);value.ChangeTechnoVertices().Append(3);value.ChangeTechnoVertices().Append(8);bo_int(value.TechnoVertices().Extent());NCollection_List<int>::Iterator aItVertex(value.TechnoVertices());for(;aItVertex.More();aItVertex.Next())bo_int(aItVertex.Value());value.SetTolerance(-0.);bo_real(value.Tolerance());
}
void bo_bopds_interf_case(const std::vector<double>&) {
 BOPDS_InterfVV vv;bo_int(vv.Index1());bo_int(vv.Index2());bo_int(vv.IndexNew());bo_int(vv.HasIndexNew());int newIndex=77;bo_int(vv.HasIndexNew(newIndex));bo_int(newIndex);bo_int(vv.GetIndexNew().has_value());
 vv.SetIndices(3,8);int first,second;vv.Indices(first,second);bo_int(first);bo_int(second);vv.SetIndex1(-4);vv.SetIndex2(0);bo_int(vv.Index1());bo_int(vv.Index2());bo_int(vv.OppositeIndex(-4));bo_int(vv.OppositeIndex(0));bo_int(vv.OppositeIndex(19));bo_int(vv.Contains(-4));bo_int(vv.Contains(8));
 vv.SetIndexNew(-2);newIndex=77;bo_int(vv.IndexNew());bo_int(vv.HasIndexNew(newIndex));bo_int(newIndex);bo_int(vv.HasIndexNew());bo_int(vv.GetIndexNew().has_value());bo_int(*vv.GetIndexNew());vv.SetIndexNew(-1);bo_int(vv.HasIndexNew());bo_int(vv.GetIndexNew().has_value());
 BOPDS_InterfVE ve;bo_int(ve.Parameter());ve.SetParameter(-0.);bo_real(ve.Parameter());ve.SetIndices(6,9);bo_int(ve.OppositeIndex(9));
 BOPDS_InterfVF vf;double u=7.,v=8.;vf.UV(u,v);bo_real(u);bo_real(v);vf.SetUV(-1.25,2.5);vf.UV(u,v);bo_real(u);bo_real(v);vf.SetIndices(11,13);bo_int(vf.Contains(13));
}
void bo_common_part_state(const IntTools_CommonPrt& part) {
 bo_int(int(part.Type()));double first,last;part.Range1(first,last);bo_real(first);bo_real(last);bo_real(part.VertexParameter1());bo_real(part.VertexParameter2());bo_int(part.AllNullFlag());gp_Pnt p1,p2;part.BoundingPoints(p1,p2);for(double c:{p1.X(),p1.Y(),p1.Z(),p2.X(),p2.Y(),p2.Z()})bo_real(c);bo_int(part.Ranges2().Length());for(int i=1;i<=part.Ranges2().Length();++i){double a,b;part.Ranges2()(i).Range(a,b);bo_real(a);bo_real(b);}
}
void bo_bopds_interf_common_case(const std::vector<double>&) {
 IntTools_CommonPrt first;first.SetType(TopAbs_EDGE);first.SetRange1(1.,2.);first.SetVertexParameter1(1.25);first.SetVertexParameter2(-2.5);first.AppendRange2(3.,4.);first.SetAllNullFlag(true);first.SetBoundingPoints(gp_Pnt(1.,2.,3.),gp_Pnt(4.,5.,6.));
 BOPDS_InterfEE ee;bo_int(ee.Index1());bo_int(ee.Index2());bo_common_part_state(ee.CommonPart());ee.SetIndices(7,9);ee.SetCommonPart(first);bo_int(ee.Index1());bo_int(ee.Index2());bo_common_part_state(ee.CommonPart());
 IntTools_CommonPrt second;second.SetType(TopAbs_VERTEX);second.SetRange1(-1.,-2.);second.SetVertexParameter1(1.25);second.SetVertexParameter2(-2.5);second.AppendRange2(7.,8.);second.SetAllNullFlag(false);second.SetBoundingPoints(gp_Pnt(-1.,-2.,-3.),gp_Pnt(-4.,-5.,-6.));ee.SetCommonPart(second);bo_common_part_state(ee.CommonPart());
 BOPDS_InterfEF ef;ef.SetIndices(11,19);ef.SetCommonPart(first);bo_int(ef.Index1());bo_int(ef.Index2());bo_common_part_state(ef.CommonPart());
}
void bo_bopds_interf_ff_case(const std::vector<double>&) {
 BOPDS_InterfFF value;int index1,index2;value.Indices(index1,index2);bo_int(index1);bo_int(index2);bo_int(value.TangentFaces());bo_int(value.Curves().Length());bo_int(value.Curves().Lower());bo_int(value.Curves().Upper());bo_int(value.Points().Length());bo_int(value.Points().Lower());bo_int(value.Points().Upper());
 value.Init(3,2);bo_int(value.Curves().Length());bo_int(value.Points().Length());value.SetIndices(7,11);value.SetTangentFaces(true);
 BOPDS_Curve firstCurve;firstCurve.SetTolerance(0.125);value.ChangeCurves().SetValue(0,firstCurve);BOPDS_Curve lastCurve;lastCurve.SetTolerance(0.875);value.ChangeCurves().SetValue(4,lastCurve);
 BOPDS_Point firstPoint;firstPoint.SetPnt(gp_Pnt(1.,2.,3.));firstPoint.SetPnt2D1(gp_Pnt2d(4.,5.));firstPoint.SetPnt2D2(gp_Pnt2d(6.,7.));firstPoint.SetIndex(10);value.ChangePoints().SetValue(0,firstPoint);
 BOPDS_Point lastPoint;lastPoint.SetPnt(gp_Pnt(-1.,-2.,-3.));lastPoint.SetPnt2D1(gp_Pnt2d(-4.,-5.));lastPoint.SetPnt2D2(gp_Pnt2d(-6.,-7.));lastPoint.SetIndex(12);value.ChangePoints().SetValue(2,lastPoint);value.Init(1,1);
 value.Indices(index1,index2);bo_int(index1);bo_int(index2);bo_int(value.TangentFaces());bo_int(value.Curves().Length());bo_int(value.Curves().Lower());bo_int(value.Curves().Upper());for(int index:{0,1,4})bo_real(value.Curves().Value(index).Tolerance());bo_int(value.Points().Length());bo_int(value.Points().Lower());bo_int(value.Points().Upper());const BOPDS_Point& first=value.Points().Value(0);const BOPDS_Point& last=value.Points().Value(2);for(double x:{first.Pnt().X(),first.Pnt().Y(),first.Pnt().Z(),first.Pnt2D1().X(),first.Pnt2D1().Y(),first.Pnt2D2().X(),first.Pnt2D2().Y()})bo_real(x);bo_int(first.Index());for(double x:{last.Pnt().X(),last.Pnt().Y(),last.Pnt().Z(),last.Pnt2D1().X(),last.Pnt2D1().Y(),last.Pnt2D2().X(),last.Pnt2D2().Y()})bo_real(x);bo_int(last.Index());
}
void bo_data_structure_init_case(const std::vector<double>& v) {
 BRep_Builder builder;TopoDS_Vertex first,second,third;builder.MakeVertex(first,gp_Pnt(1.,2.,3.),2e-5);builder.MakeVertex(second,gp_Pnt(4.,5.,6.),5e-7);builder.MakeVertex(third,gp_Pnt(7.,8.,9.),1e-4);
 TopoDS_Compound root;builder.MakeCompound(root);builder.Add(root,first);builder.Add(root,second);builder.Add(root,first);
 NCollection_List<TopoDS_Shape> arguments;arguments.Append(root);arguments.Append(first);arguments.Append(third);BOPDS_DS ds;ds.SetArguments(arguments);ds.Init(v[0]);
 bo_int(ds.NbShapes());bo_int(ds.NbSourceShapes());bo_int(ds.NbRanges());for(int i=0;i<ds.NbRanges();++i){bo_int(ds.Range(i).First());bo_int(ds.Range(i).Last());}
 bo_int(ds.Rank(0));bo_int(ds.Rank(2));bo_int(ds.Rank(3));bo_int(ds.Index(root));bo_int(ds.Index(first));bo_int(ds.Index(second));bo_int(ds.Index(third));
 for(int i=0;i<ds.NbShapes();++i){const BOPDS_ShapeInfo& info=ds.ShapeInfo(i);bo_int(info.ShapeType());bo_int(info.SubShapes().Extent());for(int child:info.SubShapes())bo_int(child);bo_int(info.Box().IsVoid());if(info.ShapeType()==TopAbs_VERTEX){bo_real(info.Box().GetGap());double x0,y0,z0,x1,y1,z1;info.Box().Get(x0,y0,z0,x1,y1,z1);for(double x:{x0,y0,z0,x1,y1,z1})bo_real(x);}}
}
void bo_data_structure_edges_case(const std::vector<double>& v) {
 BRep_Builder builder;TopoDS_Vertex first,shared,last;builder.MakeVertex(first,gp_Pnt(-1.,0.,0.),1e-7);builder.MakeVertex(shared,gp_Pnt(0.,0.,0.),2e-7);builder.MakeVertex(last,gp_Pnt(1.,1.,0.),3e-7);
 TopoDS_Edge firstEdge=BRepBuilderAPI_MakeEdge(first,shared),secondEdge=BRepBuilderAPI_MakeEdge(shared,last);TopoDS_Wire wire;builder.MakeWire(wire);builder.Add(wire,firstEdge);builder.Add(wire,secondEdge);
 NCollection_List<TopoDS_Shape> arguments;arguments.Append(wire);BOPDS_DS ds;ds.SetArguments(arguments);ds.Init(v[0]);
 bo_int(ds.NbShapes());bo_int(ds.NbSourceShapes());bo_int(ds.NbRanges());bo_int(ds.Index(first));bo_int(ds.Index(shared));bo_int(ds.Index(last));bo_int(ds.Index(firstEdge));bo_int(ds.Index(secondEdge));
 for(int i=0;i<ds.NbShapes();++i){const BOPDS_ShapeInfo& info=ds.ShapeInfo(i);bo_int(info.ShapeType());bo_int(info.SubShapes().Extent());for(int child:info.SubShapes())bo_int(child);bo_int(info.Box().IsVoid());if(!info.Box().IsVoid()){bo_real(info.Box().GetGap());double x0,y0,z0,x1,y1,z1;info.Box().Get(x0,y0,z0,x1,y1,z1);for(double x:{x0,y0,z0,x1,y1,z1})bo_real(x);}}
 ds.InitPaveBlocksForVertex(ds.Index(shared));bo_int(ds.PaveBlocksPool().Length());
 for(int edgeIndex:{ds.Index(firstEdge),ds.Index(secondEdge)}){bo_int(ds.HasPaveBlocks(edgeIndex));bo_int(ds.ShapeInfo(edgeIndex).Reference());const auto& blocks=ds.PaveBlocks(edgeIndex);bo_int(blocks.Extent());for(const auto& block:blocks){bo_int(block->OriginalEdge());int i1,i2;double p1,p2;block->Indices(i1,i2);block->Range(p1,p2);bo_int(i1);bo_int(i2);bo_real(p1);bo_real(p2);bo_int(block->ExtPaves().Extent());for(const auto& pave:block->ExtPaves()){bo_int(pave.Index());bo_real(pave.Parameter());}}}
}
void bo_data_structure_box_case(const std::vector<double>& v) {
 TopoDS_Solid solid=BRepPrimAPI_MakeBox(1.,2.,3.).Solid();NCollection_List<TopoDS_Shape> arguments;arguments.Append(solid);BOPDS_DS ds;ds.SetArguments(arguments);ds.Init(v[0]);
 bo_int(ds.NbShapes());bo_int(ds.NbSourceShapes());bo_int(ds.NbRanges());bo_int(ds.Index(solid));
 for(int i=0;i<ds.NbShapes();++i){const BOPDS_ShapeInfo& info=ds.ShapeInfo(i);bo_int(info.ShapeType());bo_int(info.SubShapes().Extent());for(int child:info.SubShapes())bo_int(child);bo_int(info.Box().IsVoid());if(!info.Box().IsVoid()){bo_real(info.Box().GetGap());double x0,y0,z0,x1,y1,z1;info.Box().Get(x0,y0,z0,x1,y1,z1);for(double x:{x0,y0,z0,x1,y1,z1})bo_real(x);}}
}
void bo_data_structure_infinite_case(const std::vector<double>& v) {
 const double infinity=Precision::Infinite();double first=-infinity,last=infinity;if(int(v[1])==1)last=2.;if(int(v[1])==2)first=-2.;if(int(v[1])==3||int(v[1])==4){first=-2.;last=3.;}
 TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(gp_Lin(),first,last);if(int(v[1])==4)edge.Orientation(TopAbs_INTERNAL);NCollection_List<TopoDS_Shape> arguments;arguments.Append(edge);BOPDS_DS ds;ds.SetArguments(arguments);ds.Init(v[0]);
 bo_int(ds.NbShapes());bo_int(ds.NbSourceShapes());bo_int(ds.NbRanges());bo_int(ds.Index(edge));
 for(int i=0;i<ds.NbShapes();++i){const BOPDS_ShapeInfo& info=ds.ShapeInfo(i);bo_int(info.ShapeType());bo_int(info.HasFlag());bo_int(info.Flag());bo_int(info.SubShapes().Extent());for(int child:info.SubShapes())bo_int(child);bo_int(info.Box().IsVoid());if(!info.Box().IsVoid()){bo_real(info.Box().GetGap());double x0,y0,z0,x1,y1,z1;info.Box().Get(x0,y0,z0,x1,y1,z1);for(double x:{x0,y0,z0,x1,y1,z1})bo_real(x);}}
 const int edgeIndex=ds.Index(edge);auto& blocks=ds.ChangePaveBlocks(edgeIndex);bo_int(ds.PaveBlocksPool().Length());bo_int(ds.ShapeInfo(edgeIndex).Reference());bo_int(blocks.Extent());for(const auto& block:blocks){int i1,i2;double p1,p2;block->Indices(i1,i2);block->Range(p1,p2);bo_int(block->OriginalEdge());bo_int(i1);bo_int(i2);bo_real(p1);bo_real(p2);}
}
void bo_data_structure_seam_case(const std::vector<double>& v) {
 TopoDS_Edge edge;if(int(v[1])%2==0){edge=BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(),1.));}else{Handle(Geom_CylindricalSurface) surface=new Geom_CylindricalSurface(gp_Ax3(),1.);Handle(Geom2d_Line) pcurve=new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0.,0.),gp_Dir2d(1.,0.)));TopoDS_Vertex vertex=BRepBuilderAPI_MakeVertex(gp_Pnt(1.,0.,0.));edge=BRepBuilderAPI_MakeEdge(pcurve,surface,vertex,vertex,0.,2.*M_PI);}
 if(int(v[1])>=2)edge.Reverse();NCollection_List<TopoDS_Shape> arguments;arguments.Append(edge);BOPDS_DS ds;ds.SetArguments(arguments);ds.Init(v[0]);const int edgeIndex=ds.Index(edge);bo_int(ds.NbShapes());bo_int(ds.NbSourceShapes());bo_int(edgeIndex);const BOPDS_ShapeInfo& info=ds.ShapeInfo(edgeIndex);bo_int(info.SubShapes().Extent());for(int child:info.SubShapes()){bo_int(child);bo_int(int(ds.Shape(child).Orientation()));}
 double first,last;TopLoc_Location location;bo_int(BRep_Tool::Curve(edge,location,first,last).IsNull());TopLoc_Location surfaceLocation;Handle(Geom2d_Curve) pcurve;Handle(Geom_Surface) curveSurface;BRep_Tool::CurveOnSurface(edge,pcurve,curveSurface,surfaceLocation,first,last);bo_int(pcurve.IsNull());
 auto& blocks=ds.ChangePaveBlocks(edgeIndex);bo_int(ds.PaveBlocksPool().Length());bo_int(ds.ShapeInfo(edgeIndex).Reference());bo_int(blocks.Extent());for(const auto& block:blocks){int i1,i2;double p1,p2;block->Indices(i1,i2);block->Range(p1,p2);bo_int(block->OriginalEdge());bo_int(i1);bo_int(i2);bo_real(p1);bo_real(p2);bo_int(block->ExtPaves().Extent());}
}
