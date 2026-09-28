// Units_UnitsSystem editing, UnitsAPI and UnitsMethods.
#include <UnitsAPI.hxx>
#include <UnitsMethods.hxx>

static void units_system_state(std::ostream& res)
{
  const occ::handle<Units_UnitsSystem>& aSystem = units_system();
  res << aSystem->IsEmpty() << ' ' << aSystem->ActiveUnitsSequence()->Length();
  for (int anIndex = 1; anIndex <= aSystem->ActiveUnitsSequence()->Length(); ++anIndex)
    res << ',' << aSystem->ActiveUnitsSequence()->Value(anIndex);
  aSystem->Dump();
}

static void units_system_op(std::istream& in, std::ostream& out)
{
  std::string anOp;
  in >> anOp;
  const std::string aQuantity = anOp == "new" || anOp == "named" || anOp == "activates" || anOp == "state"
                                  ? std::string()
                                  : units_arg(in);
  const double      aValue = anOp == "convert" ? units_real(in) : 0.0;
  const std::string aUnit = anOp == "specify" || anOp == "remove" || anOp == "activate" || anOp == "convert"
                              ? units_arg(in)
                              : std::string();
  units_run(out, [&](std::ostream& res) {
    occ::handle<Units_UnitsSystem>& aSystem = units_system();
    if (anOp == "new") aSystem = new Units_UnitsSystem();
    else if (anOp == "named") aSystem = new Units_UnitsSystem("Units", false);
    else if (anOp == "specify") aSystem->Specify(aQuantity.c_str(), aUnit.c_str());
    else if (anOp == "remove") aSystem->Remove(aQuantity.c_str(), aUnit.c_str());
    else if (anOp == "activate") aSystem->Activate(aQuantity.c_str(), aUnit.c_str());
    else if (anOp == "activates") aSystem->Activates();
    else if (anOp == "active") res << units_hex(aSystem->ActiveUnit(aQuantity.c_str()).ToCString());
    else if (anOp == "convert")
    {
      res << units_bits(aSystem->ConvertValueToUserSystem(aQuantity.c_str(), aValue, aUnit.c_str())) << ' ';
      res << units_bits(aSystem->ConvertSIValueToUserSystem(aQuantity.c_str(), aValue)) << ' ';
      res << units_bits(aSystem->ConvertUserSystemValueToSI(aQuantity.c_str(), aValue));
    }
    else units_system_state(res);
  });
}

static UnitsAPI_SystemUnits units_api_system(int theCode)
{
  return theCode == 0 ? UnitsAPI_DEFAULT : (theCode == 1 ? UnitsAPI_SI : UnitsAPI_MDTV);
}

static void units_api_values(const std::string& anOp, std::istream& in, std::ostream& res)
{
  const double      aValue = units_real(in);
  const std::string a = units_arg(in);
  if (anOp == "c2ls") res << units_bits(UnitsAPI::CurrentToLS(aValue, a.c_str()));
  else if (anOp == "c2si") res << units_bits(UnitsAPI::CurrentToSI(aValue, a.c_str()));
  else if (anOp == "cfls") res << units_bits(UnitsAPI::CurrentFromLS(aValue, a.c_str()));
  else if (anOp == "cfsi") res << units_bits(UnitsAPI::CurrentFromSI(aValue, a.c_str()));
  else if (anOp == "a2ls") res << units_bits(UnitsAPI::AnyToLS(aValue, a.c_str()));
  else if (anOp == "a2si") res << units_bits(UnitsAPI::AnyToSI(aValue, a.c_str()));
  else if (anOp == "afls") res << units_bits(UnitsAPI::AnyFromLS(aValue, a.c_str()));
  else if (anOp == "afsi") res << units_bits(UnitsAPI::AnyFromSI(aValue, a.c_str()));
  else if (anOp == "ls2si") res << units_bits(UnitsAPI::LSToSI(aValue, a.c_str()));
  else if (anOp == "si2ls") res << units_bits(UnitsAPI::SIToLS(aValue, a.c_str()));
  else if (anOp == "a2lsd" || anOp == "a2sid")
  {
    occ::handle<Units_Dimensions> aDims = Units_Dimensions::ALess();
    res << units_bits(anOp == "a2lsd" ? UnitsAPI::AnyToLS(aValue, a.c_str(), aDims)
                                      : UnitsAPI::AnyToSI(aValue, a.c_str(), aDims)) << ' ';
    units_dims(res, aDims);
  }
  else
  {
    const std::string b = units_arg(in);
    if (anOp == "c2any") res << units_bits(UnitsAPI::CurrentToAny(aValue, a.c_str(), b.c_str()));
    else if (anOp == "cfany") res << units_bits(UnitsAPI::CurrentFromAny(aValue, a.c_str(), b.c_str()));
    else res << units_bits(UnitsAPI::AnyToAny(aValue, a.c_str(), b.c_str()));
  }
}

static void units_api_dims(std::ostream& res)
{
  for (const occ::handle<Units_Dimensions>& aDims :
       {UnitsAPI::DimensionLess(), UnitsAPI::DimensionMass(), UnitsAPI::DimensionLength(),
        UnitsAPI::DimensionTime(), UnitsAPI::DimensionElectricCurrent(),
        UnitsAPI::DimensionThermodynamicTemperature(), UnitsAPI::DimensionAmountOfSubstance(),
        UnitsAPI::DimensionLuminousIntensity(), UnitsAPI::DimensionPlaneAngle(),
        UnitsAPI::DimensionSolidAngle()})
  {
    units_dims(res, aDims);
    res << ' ';
  }
}

static void units_api_op(std::istream& in, std::ostream& out)
{
  std::string anOp;
  in >> anOp;
  const bool isState = anOp == "sls" || anOp == "ls" || anOp == "reload" || anOp == "dimbasic"
                       || anOp == "save" || anOp == "scu" || anOp == "cu" || anOp == "check"
                       || anOp == "dim";
  const int         aCode = anOp == "sls" ? static_cast<int>(units_real(in)) : 0;
  const std::string a = isState && anOp != "sls" && anOp != "ls" && anOp != "reload"
                            && anOp != "dimbasic" && anOp != "save"
                          ? units_arg(in)
                          : std::string();
  const std::string b = anOp == "scu" || anOp == "check" ? units_arg(in) : std::string();
  units_run(out, [&](std::ostream& res) {
    if (!isState) units_api_values(anOp, in, res);
    else if (anOp == "sls") UnitsAPI::SetLocalSystem(units_api_system(aCode));
    else if (anOp == "reload") UnitsAPI::Reload();
    else if (anOp == "save") UnitsAPI::Save();
    else if (anOp == "scu") UnitsAPI::SetCurrentUnit(a.c_str(), b.c_str());
    else if (anOp == "cu") res << units_hex(UnitsAPI::CurrentUnit(a.c_str()));
    else if (anOp == "check") res << UnitsAPI::Check(a.c_str(), b.c_str());
    else if (anOp == "dim") units_dims(res, UnitsAPI::Dimensions(a.c_str()));
    else if (anOp == "dimbasic") units_api_dims(res);
    res << " ls=" << static_cast<int>(UnitsAPI::LocalSystem());
  });
}

static void units_methods_op(std::istream& in, std::ostream& out)
{
  std::string anOp;
  in >> anOp;
  const double      aFirst  = anOp == "fromstr" ? 0.0 : units_real(in);
  const std::string aText   = anOp == "fromstr" ? units_arg(in) : std::string();
  const double      aSecond = anOp == "scale" || anOp == "byfactor" || anOp == "dumpf" || anOp == "fromstr"
                                || anOp == "casset"
                                ? units_real(in)
                                : 0.0;
  const auto unit = [](double theCode) { return static_cast<UnitsMethods_LengthUnit>(static_cast<int>(theCode)); };
  units_run(out, [&](std::ostream& res) {
    if (anOp == "factor") res << units_bits(UnitsMethods::GetLengthFactorValue(static_cast<int>(aFirst)));
    else if (anOp == "scale") res << units_bits(UnitsMethods::GetLengthUnitScale(unit(aFirst), unit(aSecond)));
    else if (anOp == "byfactor") res << UnitsMethods::GetLengthUnitByFactorValue(aFirst, unit(aSecond));
    else if (anOp == "dump") res << units_hex(UnitsMethods::DumpLengthUnit(unit(aFirst)));
    else if (anOp == "dumpf") res << units_hex(UnitsMethods::DumpLengthUnit(aFirst, unit(aSecond)));
    else if (anOp == "fromstr") res << UnitsMethods::LengthUnitFromString(aText.c_str(), aSecond != 0.0);
    else if (anOp == "casget") res << units_bits(UnitsMethods::GetCasCadeLengthUnit(unit(aFirst)));
    else if (anOp == "casset") UnitsMethods::SetCasCadeLengthUnit(aFirst, unit(aSecond));
    else if (anOp == "cascode") UnitsMethods::SetCasCadeLengthUnit(static_cast<int>(aFirst));
    res << " cas=" << units_bits(UnitsMethods::GetCasCadeLengthUnit());
  });
}

static bool units_objects_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "units_token") units_token_op(in, out);
  else if (op == "units_measure") units_measure_op(in, out);
  else if (op == "units_unit") units_unit_op(in, out);
  else if (op == "units_explore") units_explore_op(in, out);
  else if (op == "units_system") units_system_op(in, out);
  else return false;
  return true;
}

static bool units_api_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "units_api") units_api_op(in, out);
  else if (op == "units_methods") units_methods_op(in, out);
  else return false;
  return true;
}
