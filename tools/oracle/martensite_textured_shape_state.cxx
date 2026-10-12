// Local-only state oracle. Rust tests replay committed rows without linking OCCT.
#include "martensite_textured_shape_state.hxx"
int main() {
  NativeTexturedState shape;
  std::cout << std::setprecision(17);
  print_textured_state("default", shape);
  native_textured_filename_cases(shape);
  native_textured_mapping_cases(shape);
  native_textured_handle_cases(shape);
  std::cout << "MARTENSITE_RTTI " << shape.DynamicType()->Name() << ' '
    << shape.IsKind(STANDARD_TYPE(AIS_Shape)) << ' ' << shape.IsKind(STANDARD_TYPE(AIS_InteractiveObject)) << '\n';
}
