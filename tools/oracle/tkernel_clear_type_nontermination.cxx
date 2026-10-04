// OCCT 3d097a0: mixed-type Clear cannot advance past a foreign alert.
#include <Message_Alert.hxx>
#include <Message_AlertExtended.hxx>
#include <Message_CompositeAlerts.hxx>
#include <Message_Report.hxx>
#include <iostream>
#include <string>

int main(int argc, char** argv)
{
  if (argc != 3) return 2;
  const bool reportMode = std::string(argv[1]) == "report";
  const bool foreign = std::string(argv[2]) == "foreign";
  occ::handle<Message_Alert> alert = foreign
    ? occ::handle<Message_Alert>(new Message_Alert())
    : occ::handle<Message_Alert>(new Message_AlertExtended());
  Message_Report report;
  Message_CompositeAlerts composite;
  if (reportMode) report.AddAlert(Message_Info, alert);
  else composite.AddAlert(Message_Info, alert);
  std::cout << "entered\n" << std::flush;
  if (reportMode) report.Clear(STANDARD_TYPE(Message_AlertExtended));
  else composite.Clear(STANDARD_TYPE(Message_AlertExtended));
  const int remaining = reportMode ? report.GetAlerts(Message_Info).Size()
                                   : composite.Alerts(Message_Info).Size();
  std::cout << "returned " << remaining << '\n';
}
