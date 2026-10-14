# Independent nested polynomial fixture

The original cubic is C(t)=(12t(1-t)^2,12t²(1-t)), 0<=t<=1,
with Bernstein poles (0,0),(4,0),(0,4),(0,0). Its only repeated
parameter image is its native closure: for 0<t<1, y/x=t/(1-t),
strictly increasing, and both coordinates are positive.

Jacobian's exact derivative and multiplication operations (full schemas, requests,
responses in jacobian-fixture.json) give:

- x y' - y x' = 144t²(1-t)².
- x' y'' - y' x'' = 288(3t²-3t+1)
  = 864(t-1/2)²+72 > 0.
- The incoming/outgoing closure tangents are (0,-12),(12,0), a positive
  quarter-turn, and the interior curve has no zero tangent.

Thus the Jordan curve is strictly convex, including its convex closure corner.
Its diagonal chord joins (0,0) to C(1/2)=(3/2,3/2), so the point (1,1)
is strictly interior. A homothety by 1/2 about this interior point puts the
entire inner closed curve strictly inside the outer domain:

I(t)=(1/2,1/2)+C(t)/2.

Its authored poles are (1/2,1/2),(5/2,1/2),(1/2,5/2),(1/2,1/2).
The inner native closure image (1/2,1/2) is strictly interior to the
outer curve; its outer winding is +1 for forward native traversal, -1 for
reversed traversal. The ring seed (1/4,1/4) is interior to the outer curve
and outside the inner curve (every inner coordinate is at least1/2).

Integrating the displayed polynomial exactly gives outer signed area
72(1/3-1/2+1/5)=12/5. Homothety gives inner area3/5 and annular area9/5.
These are fixture expectations, not acceptance derived from a volume check.
No runtime kernel classification, tessellation or OCCT execution defines them.

Positive source labels, candidate IDs and directed-wire IDs will be assigned by
authored semantic roles. Native result ordering will be used only to locate
explicitly matched occurrences, never to generate names.

The existing independent rectangle construction fixture remains unchanged:
outer8x8, inner4x4, ring area48. A height2 prism would have volume96 and
surface area192; this arithmetic does not extend current representation admission
to native lines or prove such a body.

