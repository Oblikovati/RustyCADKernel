#include <BRepMesh_EdgeParameterProvider.hxx>
template<class Collection> void report_edge_provider(BRepMesh_EdgeParameterProvider<Collection>& provider,const BRepAdaptor_Curve& adaptor,const std::array<int,7>& v,int lower) {
  std::vector<int> order=v[6]==0?std::vector<int>{0,1,2,3,4}:v[6]==1?std::vector<int>{4,3,2,1,0}:std::vector<int>{0,4,1,3,2,4,0};
  auto pcurve=provider.GetPCurve();double first=adaptor.FirstParameter(),last=adaptor.LastParameter();
  for(int index:order){double parameter=first+(last-first)*index/4.;double result=provider.Parameter(index+lower,adaptor.Value(parameter));ga_real(result);ga_pnt2d(pcurve->Value(result));}
}
void mesh_edge_parameter(std::istream& input) {
  std::array<int,7> v;for(auto& x:v)input>>x;BRepMesh_ModelBuilder builder;IMeshTools_Parameters p;auto model=builder.Perform(model_shape(v[0]),p);auto edge=model->GetEdge(v[1]%model->EdgesNb());IMeshData::IFaceHandle face=edge->GetPCurve(0)->GetFace();auto oriented=TopoDS::Edge(edge->GetEdge().Oriented(TopAbs_Orientation(v[2])));BRepAdaptor_Curve adaptor(oriented,face->GetFace());
  BRepMesh_EdgeParameterProvider<occ::handle<NCollection_HArray1<double>>> arrayProvider;BRepMesh_EdgeParameterProvider<IMeshData::ICurveArrayAdaptorHandle> curveProvider;
  for(bool same:{v[5]!=0,v[5]==0,v[5]!=0}){edge->SetSameParam(same);double first=adaptor.FirstParameter(),last=adaptor.LastParameter();double start=v[3]==0?first:v[3]==3?20.:10.;double end=v[3]==0?last:v[3]==1?20.:10.;occ::handle<NCollection_HArray1<double>> values=new NCollection_HArray1<double>(1,5);edge->GetCurve()->Clear(false);
    for(int i=0;i<5;++i){double parameter=start+(end-start)*i/4.;values->SetValue(i+1,parameter);edge->GetCurve()->AddPoint(gp_Pnt(),parameter);}
    if(v[4]==0){arrayProvider.Init(edge,TopAbs_Orientation(v[2]),face,values);report_edge_provider(arrayProvider,adaptor,v,1);}else{IMeshData::ICurveArrayAdaptorHandle values2=new IMeshData::ICurveArrayAdaptor(edge->GetCurve());curveProvider.Init(edge,TopAbs_Orientation(v[2]),face,values2);report_edge_provider(curveProvider,adaptor,v,0);}}
  ga_flag(arrayProvider.IsInstance(STANDARD_TYPE(Standard_Transient)));ga_end();
}
