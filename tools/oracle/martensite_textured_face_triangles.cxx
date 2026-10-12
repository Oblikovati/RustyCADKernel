// Pinned CPU oracle invokes actual StdPrs_ShadedShape::FillTriangles and records float buffers.
#include "martensite_textured_face_fixture.hxx"
#include <StdPrs_ShadedShape.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Graphic3d_Buffer.hxx>
#include <Graphic3d_IndexBuffer.hxx>
#include <cstring>
#include <iomanip>
#include <iostream>
static void native_textured_attributes(const occ::handle<Graphic3d_Buffer>& buffer, int vertex) {
  for (const auto kind : {Graphic3d_TOA_POS, Graphic3d_TOA_NORM, Graphic3d_TOA_UV}) {
    int attribute = -1; size_t stride = 0;
    const auto* source = buffer->AttributeData(kind,attribute,stride);
    const int components = kind == Graphic3d_TOA_UV ? 2 : 3;
    float values[3] = {}; std::memcpy(values,source+vertex*stride,components*sizeof(float));
    for (int component = 0; component < components; ++component) std::cout << ' ' << values[component];
  }
}
static void native_textured_array(int scenario, const occ::handle<Graphic3d_ArrayOfTriangles>& array) {
  std::cout << "MARTENSITE_TRIANGLES " << scenario << ' ' << array.IsNull();
  if (array.IsNull()) { std::cout << '\n'; return; }
  const auto& buffer = array->Attributes(); const auto& indices = array->Indices();
  std::cout << ' ' << array->VertexNumber() << ' ' << array->EdgeNumber();
  for (int vertex = 0; vertex < array->VertexNumber(); ++vertex) native_textured_attributes(buffer,vertex);
  for (int index = 0; index < array->EdgeNumber(); ++index) std::cout << ' ' << indices->Index(index);
  std::cout << '\n';
}
int main() {
  std::cout << std::setprecision(17);
  for (int scenario = 0; scenario < 16; ++scenario) {
    const auto face = native_textured_face(scenario);
    native_textured_array(scenario,StdPrs_ShadedShape::FillTriangles(face,true,gp_Pnt2d(.25,-.5),gp_Pnt2d(2,3),gp_Pnt2d(.5,2)));
  }
  native_textured_array(16,StdPrs_ShadedShape::FillTriangles(TopoDS_Shape()));
}
