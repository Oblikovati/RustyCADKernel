// Local OCCT 3d097a0 oracle for negation before conversion at the signed pan boundary.
#include <AIS_ViewController.hxx>
#include <iostream>
class TouchPanBoundary : public AIS_ViewController {
public:
  void run() {
    myUpdateStartPointPan = false;
    for (int index = 0; index < 2; ++index) {
      Aspect_Touch touch(double(index * 20), -1073741824., false);
      touch.To.SetValues(double(index * 20), 1073741824.);
      myTouchPoints.Add(size_t(index), touch);
    }
    flushGestures({}, {});
    std::cout << myGL.Panning.Delta.x() << ' ' << myGL.Panning.Delta.y() << '\n';
  }
};
int main() {
  TouchPanBoundary controller;
  controller.run();
}
