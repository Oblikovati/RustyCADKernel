// SPDX-License-Identifier: GPL-2.0-only
// Independent OCCT 8.0.0 ordinary-input receipt for ADR 0006, kernel issue #10.
#include <Geom2dAPI_InterCurveCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Line.hxx>
#include <Standard_Version.hxx>
#include <algorithm>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

std::vector<std::string> split(const std::string& text, char delimiter) {
  std::vector<std::string> fields;
  std::stringstream stream(text);
  std::string field;
  while (std::getline(stream, field, delimiter)) fields.push_back(field);
  return fields;
}
std::vector<double> decode(const std::string& text) {
  std::vector<double> values;
  for (const auto& field : split(text, ',')) {
    const uint64_t bits = std::stoull(field, nullptr, 16);
    double value;
    std::memcpy(&value, &bits, sizeof(value));
    values.push_back(value);
  }
  return values;
}
occ::handle<Geom2d_BezierCurve> curve(const std::vector<std::string>& fields) {
  const auto points = split(fields.at(1), ';');
  NCollection_Array1<gp_Pnt2d> poles(1, static_cast<int>(points.size()));
  for (int index = 1; index <= poles.Upper(); ++index) {
    const auto point = decode(points.at(index - 1));
    poles.SetValue(index, gp_Pnt2d(point.at(0), point.at(1)));
  }
  if (fields.at(2) == "-") return new Geom2d_BezierCurve(poles);
  const auto values = decode(fields.at(2));
  NCollection_Array1<double> weights(1, static_cast<int>(values.size()));
  for (int index = 1; index <= weights.Upper(); ++index)
    weights.SetValue(index, values.at(index - 1));
  return new Geom2d_BezierCurve(poles, weights);
}
void record(const std::vector<std::string>& fields, bool sides) {
  const auto endpoints = decode(fields.at(3));
  const gp_Pnt2d origin(endpoints.at(0), endpoints.at(1));
  const gp_Dir2d direction(endpoints.at(2)-origin.X(), endpoints.at(3)-origin.Y());
  const occ::handle<Geom2d_Line> line = new Geom2d_Line(origin, direction);
  // Native tolerance only; exact certification is provided by the rational oracle.
  const Geom2dAPI_InterCurveCurve result(curve(fields), line, 1e-10);
  if (!result.Intersector().IsDone()) throw std::runtime_error(fields[0] + ": unfinished; expected done");
  std::vector<std::pair<double, int>> parameters;
  for (int index = 1; index <= result.NbPoints(); ++index) {
    const auto point = result.Point(index);
    const double along = direction.X()*(point.X()-origin.X()) + direction.Y()*(point.Y()-origin.Y());
    parameters.emplace_back(result.Intersector().Point(index).ParamOnFirst(), (along > 0) - (along < 0));
  }
  std::sort(parameters.begin(), parameters.end());
  std::cout << fields[0] << '\t' << result.NbSegments() << '\t';
  for (size_t index = 0; index < parameters.size(); ++index)
    std::cout << (index ? "," : "") << (sides ? parameters[index].second : parameters[index].first);
  std::cout << '\n';
}
int main(int argc, char** argv) {
  static_assert(OCC_VERSION_MAJOR == 8 && OCC_VERSION_MINOR == 0 && OCC_VERSION_MAINTENANCE == 0);
  if (argc != 2 && !(argc == 3 && std::string(argv[2]) == "--sides"))
    throw std::runtime_error("argument count " + std::to_string(argc) + "; expected corpus path [--sides]");
  std::ifstream input(argv[1]);
  if (!input) throw std::runtime_error(std::string(argv[1]) + "; expected readable corpus");
  std::cout << std::setprecision(17);
  std::string row;
  while (std::getline(input, row)) {
    const auto fields = split(row, '\t');
    const auto& name = fields[0];
    if (name == "linear" || name == "tangent" || name == "endpoints" ||
        name == "weighted_crossing" || name == "rational_quadratic" || name == "oblique") record(fields, argc == 3);
  }
}
