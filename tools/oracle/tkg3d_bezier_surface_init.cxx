#include <GeomEval_RepSurfaceDesc.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_Plane.hxx>
#include <Standard_Failure.hxx>
#include <gp_Pln.hxx>
#include <iomanip>
#include <iostream>
#include <string>

class InitSurface : public Geom_BezierSurface
{
public:
  using Geom_BezierSurface::init;
  explicit InitSurface(const NCollection_Array2<gp_Pnt>& poles) : Geom_BezierSurface(poles) {}
  InitSurface(const NCollection_Array2<gp_Pnt>& poles,
              const NCollection_Array2<double>& weights)
      : Geom_BezierSurface(poles, weights)
  {
  }
};

NCollection_Array2<gp_Pnt> make_poles(int rows, int columns, double scale)
{
  NCollection_Array2<gp_Pnt> poles(1, rows, 1, columns);
  for (int row = 1; row <= rows; ++row)
    for (int col = 1; col <= columns; ++col)
      poles(row, col) = gp_Pnt(row * scale, col * scale, 0.);
  return poles;
}

NCollection_Array2<double> make_weights(int rows, int columns, double first, double second)
{
  NCollection_Array2<double> weights(1, rows, 1, columns);
  for (int row = 1; row <= rows; ++row)
    for (int col = 1; col <= columns; ++col)
      weights(row, col) = col == 1 ? first : second;
  return weights;
}

void set_representation(InitSurface& surface)
{
  auto descriptor = new GeomEval_RepSurfaceDesc::Full();
  descriptor->Representation = new Geom_Plane(gp_Pln());
  surface.SetEvalRepresentation(descriptor);
}

void print_state(const InitSurface& surface)
{
  const auto& weights = surface.WeightsArray();
  std::cout << surface.NbUPoles() << ' ' << surface.NbVPoles() << ' '
            << surface.IsURational() << ' ' << surface.IsVRational() << ' '
            << surface.HasEvalRepresentation() << ' ' << surface.Poles()(1, 1).X() << ' '
            << weights.NbRows() << ' ' << weights.NbColumns() << '\n';
}

void print_resolution(InitSurface& surface, const char* label)
{
  double u = 0.;
  double v = 0.;
  surface.Resolution(1., u, v);
  std::cout << label << ' ' << u << ' ' << v << '\n';
}

void probe_constructor_validation()
{
  const auto poles = make_poles(2, 2, 1.);
  const auto invalid = make_weights(2, 2, 0., 0.);
  try
  {
    Geom_BezierSurface rejected(poles, invalid);
  }
  catch (const Standard_Failure& error)
  {
    std::cout << "Ctor " << error.ExceptionType() << '\n';
  }
}

void probe_successful_reuse()
{
  const auto initial_poles = make_poles(2, 2, 1.);
  const auto poles = make_poles(3, 2, 4.);
  const auto valid = make_weights(3, 2, 1., 2.);
  const auto invalid = make_weights(3, 2, 0., 0.);
  InitSurface surface(initial_poles);
  print_resolution(surface, "Before");
  set_representation(surface);
  surface.init(poles, &valid);
  print_state(surface);
  print_resolution(surface, "After");
  surface.init(poles, &invalid);
  print_state(surface);
  std::cout << "Weight " << surface.Weight(1, 1) << '\n';
}

void probe_null_weights()
{
  const auto poles = make_poles(2, 2, 2.);
  InitSurface surface(poles);
  set_representation(surface);
  surface.init(poles, nullptr);
  print_state(surface);
  std::cout << "NullWeight " << surface.Weight(1, 1) << '\n';
}

void probe_same_count_weighted_crash()
{
  const auto poles = make_poles(2, 2, 1.);
  const auto rational_weights = make_weights(2, 2, 1., 2.);
  InitSurface surface(poles);
  std::cerr << "before-same-count-init\n";
  surface.init(poles, &rational_weights);
  std::cerr << "after-same-count-init\n";
}

void probe_dimension_failure()
{
  const auto initial_poles = make_poles(2, 2, 1.);
  const auto initial_weights = make_weights(2, 2, 1., 2.);
  const auto changed_poles = make_poles(3, 2, 5.);
  const auto transposed_weights = make_weights(2, 3, 1., 2.);
  InitSurface surface(initial_poles, initial_weights);
  set_representation(surface);
  print_resolution(surface, "FailureBefore");
  try
  {
    surface.init(changed_poles, &transposed_weights);
  }
  catch (const Standard_Failure& error)
  {
    std::cout << "Init " << error.ExceptionType() << '\n';
  }
  std::cout << "FailureState " << surface.NbUPoles() << ' ' << surface.NbVPoles() << ' '
            << surface.IsURational() << ' ' << surface.IsVRational() << ' '
            << surface.HasEvalRepresentation() << ' ' << surface.Poles()(1, 1).X() << '\n';
}

int main(int argc, char** argv)
{
  if (argc == 2 && std::string(argv[1]) == "same-count-crash")
  {
    probe_same_count_weighted_crash();
    return 0;
  }
  std::cout << std::setprecision(17);
  probe_constructor_validation();
  probe_successful_reuse();
  probe_null_weights();
  probe_dimension_failure();
}
