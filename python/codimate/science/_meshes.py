"""The meshes bodies are made of. Private to the kit."""

from __future__ import annotations

from functools import lru_cache

from ._vectors import _cross, _dot, _sub, _unit


@lru_cache(maxsize=None)
def icosphere(detail: int):
    """``(vertices, faces, edges)`` of a unit geodesic sphere.

    Made by cutting an icosahedron's triangles into four, ``detail`` times:
    ``10 * 4**detail + 2`` vertices, ``20 * 4**detail`` faces. Computed once
    per detail and shared.

    Not a latitude/longitude grid, on purpose. That has two poles where every
    meridian meets, and seen near edge-on a wireframe of it shows two dense
    tufts that read as a peanut — finer lines do not help, because the poles
    are the problem. A geodesic mesh has no special points to look at.
    """
    if not 0 <= detail <= 6:
        raise ValueError(f"detail is 0 to 6, got {detail}")
    phi = (1 + 5 ** 0.5) / 2
    verts = [_unit(v) for v in (
        (-1, phi, 0), (1, phi, 0), (-1, -phi, 0), (1, -phi, 0),
        (0, -1, phi), (0, 1, phi), (0, -1, -phi), (0, 1, -phi),
        (phi, 0, -1), (phi, 0, 1), (-phi, 0, -1), (-phi, 0, 1))]
    faces = [(0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11),
             (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
             (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9),
             (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1)]
    cache = {}

    def middle(i, j):
        key = (min(i, j), max(i, j))
        if key not in cache:
            a, b = verts[i], verts[j]
            verts.append(_unit(((a[0] + b[0]) / 2, (a[1] + b[1]) / 2,
                                (a[2] + b[2]) / 2)))
            cache[key] = len(verts) - 1
        return cache[key]

    for _ in range(detail):
        cut = []
        for a, b, c in faces:
            ab, bc, ca = middle(a, b), middle(b, c), middle(c, a)
            cut += [(a, ab, ca), (b, bc, ab), (c, ca, bc), (ab, bc, ca)]
        faces = cut
    # Wind every face outward (counter-clockwise seen from outside), once, so
    # that whatever turns or stretches the ball by positive amounts keeps its
    # faces outward and nothing has to check them again.
    outward = []
    for a, b, c in faces:
        normal = _cross(_sub(verts[b], verts[a]), _sub(verts[c], verts[a]))
        outward.append((a, b, c) if _dot(normal, verts[a]) > 0 else (a, c, b))
    edges = sorted({(min(i, j), max(i, j)) for a, b, c in outward
                    for i, j in ((a, b), (b, c), (c, a))})
    return tuple(verts), tuple(outward), tuple(edges)
