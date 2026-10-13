// SPDX-License-Identifier: GPL-2.0-only
// Captures the actual Go aa8297d1 handle methods and EntityPolyline; not a production path.
package main

import (
	"fmt"
	"oblikovati.org/api/types"
	"oblikovati.org/kernel/geom"
	"oblikovati.org/math"
	"oblikovati.org/model/sketch"
)

func main() {
	for index := 0; index < 6; index++ {
		emitHandled(index)
	}
}
func emitHandled(index int) {
	points := []math.Point2{math.P2(0, 0), math.P2(.2, 1), math.P2(2, 1.5), math.P2(3, -.5), math.P2(1, -2)}
	owner := sketch.NewSketches().Add(sketch.XYPlane())
	spline := owner.Splines().AddByPoints(points, index >= 3)
	spline.FitMethod = []types.SplineFitMethod{types.SplineFitSmooth, types.SplineFitChord, types.SplineFitSweet}[index%3]
	for _, spec := range []struct {
		slot int
		end  math.Point2
	}{{0, math.P2(.5, .25)}, {2, math.P2(1.75, 2)}, {4, math.P2(1, -2)}} {
		handle, err := owner.SplineHandles().Activate(spline, spec.slot)
		if err != nil {
			panic(err)
		}
		handle.End.SetPosition(spec.end)
	}
	curve, stations := naturalCurve(points, index)
	emitStations(index, spline, curve, stations)
	samples, _ := sketch.EntityPolyline(spline)
	for slot, point := range samples {
		span, step := slot/8, slot%8
		parameter := 1.0
		if span+1 < len(stations) {
			parameter = stations[span] + (stations[span+1]-stations[span])*float64(step)/8
		}
		fmt.Printf("%d,sample,%d,%.17g,%.17g,%.17g\n", index, slot, parameter, point.X, point.Y)
	}
}
func naturalCurve(points []math.Point2, index int) (geom.BSplineCurve2d, []float64) {
	method := geom.FitParameterization(index % 3)
	if index >= 3 {
		curve, stations, err := geom.NewClosedFittedBSplineCurve2d(points, method)
		if err != nil {
			panic(err)
		}
		return curve, stations
	}
	curve, stations, err := geom.NewFittedBSplineCurve2dParam(points, method)
	if err != nil {
		panic(err)
	}
	return curve, stations
}
func emitStations(index int, spline *sketch.Spline, curve geom.BSplineCurve2d, stations []float64) {
	for slot, parameter := range stations {
		source := slot % len(spline.Points)
		derivative := curve.TangentAt(stations[source])
		for _, handle := range spline.Handles() {
			if handle.FitIndex != source {
				continue
			}
			direction, valid := handle.TangentDirection()
			if valid {
				derivative = direction.Scale(math.Scalar(handle.Weight() * float64(derivative.Length())))
			}
		}
		point := spline.Points[source].Position()
		fmt.Printf("%d,station,%d,%.17g,%.17g,%.17g,%.17g,%.17g\n", index, slot, parameter, point.X, point.Y, derivative.X, derivative.Y)
	}
}
