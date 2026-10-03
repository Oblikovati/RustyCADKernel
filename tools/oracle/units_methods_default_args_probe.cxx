#include <UnitsMethods.hxx>

#include <iomanip>
#include <iostream>

int main()
{
  UnitsMethods::SetCasCadeLengthUnit(25.4);
  std::cout << std::setprecision(17)
            << UnitsMethods::GetCasCadeLengthUnit() << '\n'
            << static_cast<int>(UnitsMethods::GetLengthUnitByFactorValue(25.4)) << '\n'
            << UnitsMethods::DumpLengthUnit(25.4) << '\n';
}
