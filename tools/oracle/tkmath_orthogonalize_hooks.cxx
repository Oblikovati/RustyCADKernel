// Protected gp hooks at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <gp_Trsf.hxx>
#include <gp_Trsf2d.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <sstream>
#include <stdexcept>

struct SpatialHook : gp_Trsf { using gp_Trsf::Orthogonalize; };
struct PlanarHook : gp_Trsf2d {
  explicit PlanarHook(const gp_Trsf& source) : gp_Trsf2d(source) {}
  using gp_Trsf2d::Orthogonalize;
};

static SpatialHook stored_transform(const char* matrix) {
  std::stringstream stream;
  stream << "\"Location\": [5, 6, 7], \"Matrix\": [" << matrix
         << "], \"shape\": 7, \"scale\": -2";
  SpatialHook result;
  int position = 1;
  if (!result.InitFromJson(stream, position)) throw std::runtime_error("invalid fixture JSON");
  return result;
}

template<class Transform>
static void print_fields(const Transform& transform, int dimension) {
  std::cout << ' ' << transform.Form() << ' ' << transform.ScaleFactor();
  for (int row = 1; row <= dimension; ++row)
    for (int col = 1; col <= dimension + 1; ++col)
      std::cout << ' ' << transform.Value(row, col);
  std::cout << '\n';
}

template<class Transform>
static void invoke(const char* label, Transform& transform, int dimension) {
  std::cout << label;
  try { transform.Orthogonalize(); std::cout << " ok"; }
  catch (const Standard_Failure& failure) { std::cout << ' ' << failure.ExceptionType(); }
  print_fields(transform, dimension);
}

static void replay(const char* matrix) {
  SpatialHook spatial = stored_transform(matrix);
  PlanarHook planar(spatial);
  invoke("spatial", spatial, 3);
  invoke("planar", planar, 2);
  invoke("spatial-repeat", spatial, 3);
  invoke("planar-repeat", planar, 2);
}

int main() {
  replay("2, 1, 1, 0, 3, 1, 0, 0, 4");
  replay("0, 1, 1, 0, 3, 1, 0, 0, 4");
  replay("2, 4, 1, 0, 0, 1, 0, 0, 4");
}
