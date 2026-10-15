// SPDX-License-Identifier: GPL-2.0-only
// OCCT 8.0.0 Geom2dAPI_InterCurveCurve replay of native-cases.csv,
// native-curve-cases.csv and native-offset-cases.csv (tolerance 1e-9). A source
// field with a ':' is a curve spec like the targets; otherwise it holds the two
// points of an infinite line. Offsets (ADR 0080) wrap the untrimmed basis in a
// Geom2d_OffsetCurve and trim it like the basis; their arcs keep their travel sense.
// Native points are rounded solver output, not certified roots; see README.md.
#include <Geom2dAPI_InterCurveCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <NCollection_Array1.hxx>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

static std::vector<std::string> split(const std::string &text, char separator) {
  std::vector<std::string> fields; std::stringstream stream(text); std::string field;
  while (std::getline(stream, field, separator)) fields.push_back(field);
  return fields;
}
static double bits(const std::string &field) {
  const std::uint64_t value = std::stoull(field, nullptr, 16); double result;
  std::memcpy(&result, &value, sizeof(result)); return result;
}
static std::vector<double> numbers(const std::string &fields) {
  std::vector<double> values; for (const auto &field : split(fields, ',')) values.push_back(bits(field));
  return values;
}
static std::string hex(double value) {
  std::uint64_t raw; std::memcpy(&raw, &value, sizeof(raw));
  char text[17]; std::snprintf(text, sizeof(text), "%016llx", static_cast<unsigned long long>(raw));
  return text;
}
static Handle(Geom2d_Curve) circle(const std::vector<std::string> &fields) {
  const std::vector<std::string> raw = split(fields[1], ',');
  Handle(Geom2d_Circle) base = new Geom2d_Circle(gp_Ax2d(gp_Pnt2d(bits(raw[0]), bits(raw[1])), gp_Dir2d(1, 0)), bits(raw[2]));
  if (raw.size() == 3) return base;
  double first = std::atan2(bits(raw[4]), bits(raw[3])), last = std::atan2(bits(raw[6]), bits(raw[5]));
  if (raw[7] != "1") std::swap(first, last);
  if (last <= first) last += 2 * M_PI;
  return new Geom2d_TrimmedCurve(base, first, last);
}
static Handle(Geom2d_Curve) spline(const std::vector<std::string> &fields) {
  const int degree = std::stoi(fields[1]);
  const auto coordinates = numbers(fields[2]), flat = numbers(fields[3]);
  NCollection_Array1<gp_Pnt2d> poles(1, static_cast<int>(coordinates.size() / 2));
  for (int i = 0; i < poles.Length(); ++i) poles(i + 1) = gp_Pnt2d(coordinates[2 * i], coordinates[2 * i + 1]);
  std::vector<double> knots; std::vector<int> multiplicities;
  for (double knot : flat) {
    if (!knots.empty() && knots.back() == knot) ++multiplicities.back();
    else { knots.push_back(knot); multiplicities.push_back(1); }
  }
  NCollection_Array1<double> knotArray(1, static_cast<int>(knots.size()));
  NCollection_Array1<int> multiplicityArray(1, static_cast<int>(knots.size()));
  for (std::size_t i = 0; i < knots.size(); ++i) { knotArray(i + 1) = knots[i]; multiplicityArray(i + 1) = multiplicities[i]; }
  return new Geom2d_BSplineCurve(poles, knotArray, multiplicityArray, degree);
}
static Handle(Geom2d_Curve) target(const std::string &spec);
// A circle arc that travels in its sector sense (clockwise arcs on an indirect frame).
static Handle(Geom2d_Curve) directedCircle(const std::vector<std::string> &fields) {
  const std::vector<std::string> raw = split(fields[1], ',');
  const bool ccw = raw.size() == 3 || raw[7] == "1";
  const gp_Ax2d axis(gp_Pnt2d(bits(raw[0]), bits(raw[1])), gp_Dir2d(1, 0));
  Handle(Geom2d_Circle) base = new Geom2d_Circle(axis, bits(raw[2]), ccw);
  if (raw.size() == 3) return base;
  const double turn = ccw ? 1.0 : -1.0;
  double first = turn * std::atan2(bits(raw[4]), bits(raw[3])), last = turn * std::atan2(bits(raw[6]), bits(raw[5]));
  if (last <= first) last += 2 * M_PI;
  return new Geom2d_TrimmedCurve(base, first, last);
}
// offset:<distance>:<basis spec>: the offset of the untrimmed basis, trimmed like it.
static Handle(Geom2d_Curve) offsetCurve(const std::string &spec) {
  const std::size_t split_at = spec.find(':', 7);
  const double distance = bits(spec.substr(7, split_at - 7));
  const std::string basisSpec = spec.substr(split_at + 1);
  Handle(Geom2d_Curve) basis = basisSpec.rfind("circle:", 0) == 0 ? directedCircle(split(basisSpec, ':')) : target(basisSpec);
  Handle(Geom2d_TrimmedCurve) trimmed = Handle(Geom2d_TrimmedCurve)::DownCast(basis);
  if (trimmed.IsNull()) return new Geom2d_OffsetCurve(basis, distance);
  Handle(Geom2d_OffsetCurve) whole = new Geom2d_OffsetCurve(trimmed->BasisCurve(), distance);
  return new Geom2d_TrimmedCurve(whole, trimmed->FirstParameter(), trimmed->LastParameter());
}
// periodic:degree:poles:knots:multiplicities:bounds, a periodic B-spline (trimmed to its bounds).
static Handle(Geom2d_Curve) periodic(const std::vector<std::string> &fields) {
  const int degree = std::stoi(fields[1]);
  const auto coordinates = numbers(fields[2]), knots = numbers(fields[3]), bounds = numbers(fields[5]);
  const auto counts = split(fields[4], ',');
  NCollection_Array1<gp_Pnt2d> poles(1, static_cast<int>(coordinates.size() / 2));
  for (int i = 0; i < poles.Length(); ++i) poles(i + 1) = gp_Pnt2d(coordinates[2 * i], coordinates[2 * i + 1]);
  NCollection_Array1<double> knotArray(1, static_cast<int>(knots.size()));
  NCollection_Array1<int> multiplicityArray(1, static_cast<int>(knots.size()));
  for (std::size_t i = 0; i < knots.size(); ++i) { knotArray(i + 1) = knots[i]; multiplicityArray(i + 1) = std::stoi(counts[i]); }
  Handle(Geom2d_BSplineCurve) curve = new Geom2d_BSplineCurve(poles, knotArray, multiplicityArray, degree, true);
  if (bounds[0] == knots.front() && bounds[1] == knots.back()) return curve;
  return new Geom2d_TrimmedCurve(curve, bounds[0], bounds[1]);
}
static Handle(Geom2d_Curve) target(const std::string &spec) {
  if (spec.rfind("offset:", 0) == 0) return offsetCurve(spec);
  const auto fields = split(spec, ':');
  if (fields[0] == "periodic") return periodic(fields);
  if (fields[0] == "circle") return circle(fields);
  if (fields[0] == "spline") return spline(fields);
  const auto values = numbers(fields[1]);
  if (fields[0] == "ellipse")
    return new Geom2d_Ellipse(gp_Ax22d(gp_Pnt2d(values[0], values[1]), gp_Dir2d(values[2], values[3])), values[4], values[5]);
  if (fields[0] == "line") {
    const gp_Pnt2d start(values[0], values[1]), end(values[2], values[3]);
    return new Geom2d_Line(start, gp_Dir2d(gp_Vec2d(start, end)));
  }
  if (fields[0] == "segment") {
    const gp_Pnt2d start(values[0], values[1]), end(values[2], values[3]);
    Handle(Geom2d_Line) line = new Geom2d_Line(start, gp_Dir2d(gp_Vec2d(start, end)));
    return new Geom2d_TrimmedCurve(line, 0, start.Distance(end));
  }
  throw std::runtime_error("curve kind " + fields[0] + ": expected circle, ellipse, line, segment or spline");
}
int main(int argc, char **argv) {
  if (argc != 2) throw std::runtime_error("argument count " + std::to_string(argc) + ": expected corpus path");
  std::ifstream corpus(argv[1]); std::string row;
  if (!corpus) throw std::runtime_error("path " + std::string(argv[1]) + ": expected readable corpus");
  while (std::getline(corpus, row)) {
    const auto fields = split(row, ';');
    Handle(Geom2d_Curve) source;
    if (fields[2].find(':') != std::string::npos) {
      source = target(fields[2]);
    } else {
      const auto points = numbers(fields[2]);
      const gp_Pnt2d start(points[0], points[1]), end(points[2], points[3]);
      source = new Geom2d_Line(start, gp_Dir2d(gp_Vec2d(start, end)));
    }
    Geom2dAPI_InterCurveCurve native(source, target(fields[3]), 1e-9);
    std::cout << fields[0] << ';' << native.NbPoints() << ';';
    for (int i = 1; i <= native.NbPoints(); ++i) {
      const gp_Pnt2d point = native.Point(i);
      std::cout << (i > 1 ? "|" : "") << hex(point.X()) << ',' << hex(point.Y());
    }
    std::cout << ';' << native.NbSegments();
    // Coincidence zones (none in the line corpus) report their two end points.
    for (int i = 1; i <= native.NbSegments(); ++i) {
      Handle(Geom2d_Curve) first, second;
      native.Segment(i, first, second);
      const gp_Pnt2d start = first->Value(first->FirstParameter()), end = first->Value(first->LastParameter());
      std::cout << (i > 1 ? '|' : ';') << hex(start.X()) << ',' << hex(start.Y()) << ',' << hex(end.X()) << ',' << hex(end.Y());
    }
    std::cout << '\n';
  }
}
