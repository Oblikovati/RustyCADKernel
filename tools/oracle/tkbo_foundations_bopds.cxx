// Calls each scalar BOPDS member and its native Dump at 3d097a0.
#include <BOPDS_IndexRange.hxx>
#include <BOPDS_Pave.hxx>
#include <BOPDS_Pair.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BOPDS_Point.hxx>
#include <Bnd_Box.hxx>
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
