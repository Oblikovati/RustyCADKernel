// SPDX-License-Identifier: GPL-2.0-only
// Go aa8297d1, controlCurveFor's five-point policy; runtime use is oracle capture only.
package main

import (
	"fmt"
	"oblikovati.org/kernel/geom"
	"oblikovati.org/math"
)

func main() {
	points := []math.Point2{{X: 0, Y: 0}, {X: 0.2, Y: 1}, {X: 2, Y: 1.5}, {X: 3, Y: -0.5}, {X: 1, Y: -2}}
	open, err := geom.NewBSplineCurve2dUniformWeights(3, points, []float64{0, 0, 0, 0, 0.5, 1, 1, 1, 1})
	if err != nil {
		panic(err)
	}
	closed, err := geom.NewClosedControlBSplineCurve2d(points)
	if err != nil {
		panic(err)
	}
	emit(0, open)
	emit(1, closed)
}
func emit(index int, curve geom.BSplineCurve2d) {
	fmt.Printf("%d,shape,0,%d,%d,%d,0\n", index, curve.Degree, len(curve.Ctrl), len(curve.Knots))
	for slot, pole := range curve.Ctrl {
		fmt.Printf("%d,pole,%d,%.17g,%.17g\n", index, slot, pole.X, pole.Y)
	}
	for slot, knot := range curve.Knots {
		fmt.Printf("%d,knot,%d,%.17g\n", index, slot, knot)
	}
	for slot := 0; slot <= 16; slot++ {
		emitSample(index, slot, curve)
	}
}
func emitSample(index, slot int, curve geom.BSplineCurve2d) {
	parameter := float64(slot) / 16
	point, tangent := curve.PointAt(parameter), curve.TangentAt(parameter)
	fmt.Printf("%d,sample,%d,%.17g,%.17g,%.17g,%.17g,%.17g\n", index, slot, parameter, point.X, point.Y, tangent.X, tangent.Y)
}
