// SPDX-License-Identifier: GPL-2.0-only
// OCCT 8.0.0 evaluates the retained Go recipes; no production dependency is added.
#include <Geom2d_BSplineCurve.hxx>
#include <NCollection_Array1.hxx>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>
struct Recipe {
  int degree = 0;
  std::vector<gp_Pnt2d> poles;
  std::vector<double> flat;
  std::vector<double> parameters;
};
std::vector<std::string> fields(const std::string &line) {
  std::vector<std::string> result;
  std::stringstream stream(line);
  std::string field;
  while (std::getline(stream, field, ',')) result.push_back(field);
  return result;
}
void consume(Recipe &recipe, const std::vector<std::string> &row) {
  if (row.at(1) == "shape") recipe.degree = std::stoi(row.at(3));
  if (row.at(1) == "pole") recipe.poles.emplace_back(std::stod(row.at(3)), std::stod(row.at(4)));
  if (row.at(1) == "knot") recipe.flat.push_back(std::stod(row.at(3)));
  if (row.at(1) == "sample") recipe.parameters.push_back(std::stod(row.at(3)));
}
std::vector<Recipe> read_recipes(const std::string &path) {
  std::ifstream source(path);
  if (!source) throw std::runtime_error("oracle path " + path + ": expected readable CSV");
  std::vector<Recipe> recipes;
  std::string line;
  while (std::getline(source, line)) {
    const auto row = fields(line);
    const auto index = std::stoul(row.at(0));
    if (recipes.size() <= index) recipes.resize(index + 1);
    consume(recipes.at(index), row);
  }
  return recipes;
}
Handle(Geom2d_BSplineCurve) curve(const Recipe &recipe) {
  std::vector<double> knots;
  std::vector<int> mults;
  for (double knot : recipe.flat) {
    if (!knots.empty() && knots.back() == knot) ++mults.back();
    else { knots.push_back(knot); mults.push_back(1); }
  }
  NCollection_Array1<gp_Pnt2d> poles(1, recipe.poles.size());
  NCollection_Array1<double> distinct(1, knots.size());
  NCollection_Array1<int> multiplicities(1, mults.size());
  for (int i = 1; i <= poles.Upper(); ++i) poles(i) = recipe.poles.at(i - 1);
  for (int i = 1; i <= distinct.Upper(); ++i) {
    distinct(i) = knots.at(i - 1); multiplicities(i) = mults.at(i - 1);
  }
  return new Geom2d_BSplineCurve(poles, distinct, multiplicities, recipe.degree, false);
}
void emit(std::size_t index, const Recipe &recipe) {
  const auto analytic = curve(recipe);
  for (double parameter : recipe.parameters) {
    gp_Pnt2d point;
    gp_Vec2d first, second;
    analytic->D2(parameter, point, first, second);
    std::cout << index << ',' << parameter << ',' << point.X() << ',' << point.Y()
              << ',' << first.X() << ',' << first.Y() << ',' << second.X() << ',' << second.Y() << '\n';
  }
}
int main(int argc, char **argv) {
  if (argc != 2) throw std::runtime_error("oracle argument count " + std::to_string(argc) + ": expected one Go CSV path");
  const auto recipes = read_recipes(argv[1]);
  std::cout << std::setprecision(17);
  for (std::size_t index = 0; index < recipes.size(); ++index) emit(index, recipes[index]);
}
