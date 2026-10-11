# Martensite selection and scene fitting

`AisViewController::fit_all_auto` implements the complete native
`AIS_ViewController::FitAllAuto` at OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. It obtains bounds from the actual
`AisInteractiveContext` selection and fits the actual `V3dView` camera. For
example, `controller.fit_all_auto(&context, &view)?` fits selection first;
another call with that same fitted camera switches to the whole scene.

`context.bounding_box_of_selection(Some(&view))?` filters infinite objects and
objects hidden by their view affinity. Passing `None` omits the affinity filter.
Global owners use their object's presentation bounds. Other owners are grouped
and deduplicated per selectable object, then bounds are collected from the
object's activated sensitive geometry with its transformation. The detached
selection walk preserves the context's public selection cursor.

For a nonvoid selection, automatic fitting evaluates selection and scene on
camera copies with margin 0.01 and native line enlargement. It compares center,
scale and distance against one millionth of the selected bounds' diagonal.
Scale and distance use strict comparisons; a zero diagonal therefore does not
satisfy the toggle condition. Camera contents are copied back without replacing
the view's camera handle. A void selection invokes ordinary scene FitAll.
Native false FitMinMax results remain tolerated; typed operation errors propagate.

`context.fit_selected(&view, 0.05, false)?` fits selected bounds with an explicit
margin and update policy. `context.fit_selected_default(&view)?` uses native
0.01/true defaults. A void selection returns before validating the margin or
changing the camera, preserving the native failure order.

Run `cargo test -p rustycad-tkv3d --all-targets --locked` locally. The committed
native assets contain 3,072 snapshots across 512 actual context/view scenes,
each with an initial frame, three FitAllAuto calls, default FitSelected and
explicit FitSelected. Cases exercise global owners, Y/Z trihedron part owners,
mixed owners, duplicate selection insertion, zero-diagonal point bounds,
transformations, infinite objects,
per-view visibility, and orthographic/perspective cameras. Every snapshot
compares selected bounds both with and without a view, scene bounds, selection
count and 19 camera values. Additional Rust regressions cover cursor retention,
margin failure order, toggling and camera handle identity. Native generation
uses a retained CPU scene driver locally; Rust replay has no OCCT dependency.

The deprecated no-argument BoundingBoxOfSelection wrapper is not claimed.
General controller frame dispatch, render application and XR remain separate
work; the controller and context classes remain partial. Windows/macOS testing
is deferred.
