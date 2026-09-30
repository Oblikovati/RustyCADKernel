#include <BRepMesh_Deflection.hxx>
void mesh_deflection(std::istream& input) {
  std::array<int,6> v;for(auto& x:v)input>>x;auto shape=model_shape(v[0]);
  IMeshTools_Parameters p;p.Relative=v[1];p.ForceFaceDeflection=v[2];p.Deflection=std::array<double,3>{-1,0.001,2}[v[3]];p.DeflectionInterior=std::array<double,3>{-1,0.01,4}[v[4]];
  ga_real(BRepMesh_Deflection::ComputeAbsoluteDeflection(shape,p.Deflection,std::array<double,3>{-1,0.1,100}[v[5]]));
  BRepMesh_ModelBuilder builder;auto model=builder.Perform(shape,p);if(model.IsNull()){ga_end();return;}
  for(int i=0;i<model->EdgesNb();++i){auto edge=model->GetEdge(i);BRepMesh_Deflection::ComputeDeflection(edge,model->GetMaxSize(),p);ga_real(edge->GetDeflection());ga_real(edge->GetAngularDeflection());}
  for(int i=0;i<model->FacesNb();++i){auto face=model->GetFace(i);for(int j=0;j<face->WiresNb();++j){auto wire=face->GetWire(j);BRepMesh_Deflection::ComputeDeflection(wire,p);ga_real(wire->GetDeflection());}BRepMesh_Deflection::ComputeDeflection(face,p);ga_real(face->GetDeflection());}ga_end();
}
void mesh_consistent(std::istream& input) {
  int current,required,decrease,ratio;input>>current>>required>>decrease>>ratio;
  const std::array<double,9> values={-1,-0.0,0.001,0.9,1,1.1,2,INFINITY,NAN};
  ga_flag(BRepMesh_Deflection::IsConsistent(values[current],values[required],decrease,std::array<double,4>{-0.1,0,0.1,1}[ratio]));ga_end();
}
