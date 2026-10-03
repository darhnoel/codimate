"""The science kit: marks, and bodies in space."""

import math
import shutil

import support  # noqa: F401  (puts `codimate` on the import path)
import codimate as cm
from codimate import science


def _engine():
    """Is there an Engine to measure with? Text and formulas are measured by
    it, so the checks that need a measurement stand down without one — the
    rest are plain arithmetic and always run."""
    try:
        cm.measure("a", 12)
    except Exception:
        return False
    return True


def _shapes(scene):
    return scene._shapes


def _by_kind(scene, kind):
    return [s for s in _shapes(scene).values() if s.kind == kind]


# ---------------------------------------------------------------- Camera


def test_the_target_lands_on_the_middle_of_the_canvas():
    cm.canvas(1280, 720)
    cam = science.Camera(target=(10.0, 20.0, 30.0))
    (x, y), depth = cam.project((10.0, 20.0, 30.0))
    assert (round(x, 6), round(y, 6)) == (640.0, 360.0)
    assert abs(depth - cam.distance) < 1e-9, "the target is `distance` away"


def test_a_centre_overrides_where_the_target_lands():
    cm.canvas(1280, 720)
    cam = science.Camera(centre=(200.0, 100.0))
    x, y = cam.at((0.0, 0.0, 0.0))
    assert (round(x, 6), round(y, 6)) == (200.0, 100.0)


def test_up_in_space_is_up_on_screen():
    cm.canvas(1280, 720)
    x, y = science.Camera().at((0.0, 0.0, 100.0))
    assert abs(x - 640) < 1e-6, "the vertical axis stays vertical"
    assert y < 360, "screen y grows down, so up in space is a smaller y"


def test_a_unit_of_space_is_scale_pixels_at_the_target():
    cm.canvas(1280, 720)
    cam = science.Camera(azimuth=30.0, scale=2.0)
    right = (-math.sin(math.radians(30)), math.cos(math.radians(30)), 0.0)
    x, y = cam.at(tuple(100.0 * c for c in right))
    assert abs((x - 640) - 200.0) < 1e-6, "100 units * scale 2"
    assert abs(y - 360) < 1e-6


def test_nearer_things_look_bigger():
    cm.canvas(1280, 720)
    cam = science.Camera(azimuth=0.0, elevation=0.0)
    # Looking down -x from +x: space's y is screen-right, space's +x is toward us.
    far = cam.at((-400.0, 100.0, 0.0))[0] - 640
    near = cam.at((400.0, 100.0, 0.0))[0] - 640
    assert near > far > 0, "the same sideways step covers more screen when closer"


def test_a_very_far_camera_has_no_perspective():
    cm.canvas(1280, 720)
    cam = science.Camera(azimuth=0.0, elevation=0.0, distance=1e9, scale=1.0)
    near = cam.at((400.0, 100.0, 0.0))[0] - 640
    far = cam.at((-400.0, 100.0, 0.0))[0] - 640
    assert abs(near - far) < 1e-3, "orthographic: depth does not change size"


def test_the_default_view_is_isometric():
    """All three axes 120 degrees apart on screen — the engineering drawing."""
    cm.canvas(1280, 720)
    cam = science.Camera(distance=1e7, scale=1.0)
    c = cam.at((0.0, 0.0, 0.0))
    angles = []
    for axis in ((100.0, 0.0, 0.0), (0.0, 100.0, 0.0), (0.0, 0.0, 100.0)):
        x, y = cam.at(axis)
        angles.append(math.degrees(math.atan2(y - c[1], x - c[0])) % 360)
    angles.sort()
    gaps = [angles[1] - angles[0], angles[2] - angles[1],
            360 - angles[2] + angles[0]]
    assert all(abs(g - 120) < 0.1 for g in gaps), gaps


def test_a_surface_faces_the_camera_only_on_the_near_side():
    cam = science.Camera()
    toward = tuple(p / math.sqrt(sum(q * q for q in cam.position))
                   for p in cam.position)
    near = tuple(100 * t for t in toward)
    far = tuple(-v for v in near)
    assert cam.facing(near, toward)
    assert not cam.facing(far, tuple(-t for t in toward))


def test_a_point_behind_a_sphere_is_hidden_and_beside_or_before_is_not():
    cam = science.Camera()
    pos = cam.position
    toward = tuple(p / math.sqrt(sum(q * q for q in pos)) for p in pos)
    behind = tuple(-300 * t for t in toward)
    before = tuple(300 * t for t in toward)
    beside = (0.0, 0.0, 500.0)
    assert cam.hidden_by(behind, (0.0, 0.0, 0.0), 100.0)
    assert not cam.hidden_by(before, (0.0, 0.0, 0.0), 100.0)
    assert not cam.hidden_by(beside, (0.0, 0.0, 0.0), 100.0)


def test_a_ring_far_outside_a_body_is_hidden_only_where_the_body_covers_it():
    """The mistake it replaces: "is this on the far half of its own orbit" hid
    about half of a big ring that a small planet never covered."""
    cam = science.Camera(azimuth=0.0, elevation=0.0)
    ring = [(420 * math.cos(math.radians(a)), 420 * math.sin(math.radians(a)), 0.0)
            for a in range(0, 360, 5)]
    hidden = sum(cam.hidden_by(p, (0.0, 0.0, 0.0), 160.0) for p in ring)
    assert 0 < hidden < len(ring) * 0.2, f"{hidden} of {len(ring)}"


def test_moved_changes_only_what_it_is_told():
    cam = science.Camera(azimuth=10.0, distance=900.0)
    moved = cam.moved(azimuth=80.0)
    assert (moved.azimuth, moved.distance) == (80.0, 900.0)
    assert cam.azimuth == 10.0, "the original is untouched"


def test_a_dolly_backs_away_with_the_same_lens_so_things_shrink():
    cm.canvas(1280, 720)
    cam = science.Camera(azimuth=0.0, elevation=0.0, distance=1000.0, scale=1.0)
    far = cam.dolly(2000.0)
    assert abs(far.focal - cam.focal) < 1e-9, "the lens did not change"
    assert abs(far.scale - 0.5) < 1e-9
    side = lambda c: c.at((0.0, 100.0, 0.0))[0] - 640  # noqa: E731
    assert abs(side(far) - side(cam) / 2) < 0.1, "half the size at twice the distance"


def test_moving_the_camera_keeps_the_size_and_only_changes_the_perspective():
    cam = science.Camera(azimuth=0.0, elevation=0.0, distance=1000.0, scale=1.0)
    assert cam.moved(distance=2000.0).scale == 1.0


def test_a_camera_refuses_nonsense():
    for bad in ({"elevation": 120.0}, {"distance": 0.0}, {"scale": -1.0}):
        try:
            science.Camera(**bad)
        except ValueError:
            continue
        raise AssertionError(f"{bad} should have been refused")


# --------------------------------------------------------------- icosphere


def test_an_icosphere_has_the_counts_a_geodesic_sphere_has():
    for n in range(4):
        verts, faces, edges = science.icosphere(n)
        assert len(faces) == 20 * 4 ** n
        assert len(verts) == 10 * 4 ** n + 2
        assert len(edges) == 30 * 4 ** n
        assert len(verts) - len(edges) + len(faces) == 2, "Euler: it is a sphere"


def test_every_icosphere_vertex_is_on_the_unit_sphere():
    verts = science.icosphere(2)[0]
    assert all(abs(math.sqrt(x * x + y * y + z * z) - 1) < 1e-12
               for x, y, z in verts)


def test_an_icosphere_is_built_once_per_detail():
    assert science.icosphere(2) is science.icosphere(2)


def test_an_unreasonable_detail_is_refused():
    try:
        science.icosphere(9)
    except ValueError:
        return
    raise AssertionError("detail 9 is 5 million faces")


# ------------------------------------------------------------------- Light


def test_light_comes_from_a_direction_or_from_a_point():
    far = science.Light(direction=(0.0, 0.0, 2.0))
    assert far.toward((5.0, 5.0, 5.0)) == (0.0, 0.0, 1.0), "a unit vector, anywhere"

    sun = science.Light(source=(0.0, 0.0, 0.0))
    ux, uy, uz = sun.toward((10.0, 0.0, 0.0))
    assert (round(ux, 9), uy, uz) == (-1.0, 0.0, 0.0), "toward the source"
    assert sun.toward((0.0, 10.0, 0.0)) != sun.toward((10.0, 0.0, 0.0)), \
        "a point source lights each place from its own side"


# ------------------------------------------------------------------ Sphere


def test_a_sphere_spins_about_an_axis_that_leans_with_its_tilt():
    upright = science.Sphere(centre=(0, 0, 0), r=10)
    assert upright.axis == (0.0, 0.0, 1.0)
    leaning = science.Sphere(centre=(0, 0, 0), r=10, tilt=90.0)
    ax, ay, az = leaning.axis
    assert abs(ax) < 1e-9 and abs(ay + 1) < 1e-9 and abs(az) < 1e-9


def test_a_point_on_the_equator_turns_with_the_spin_angle():
    ball = science.Sphere(centre=(100.0, 0.0, 0.0), r=10.0)
    x, y, z = ball.on_equator(0.0)
    assert (round(x, 9), round(y, 9), round(z, 9)) == (110.0, 0.0, 0.0)
    x, y, z = ball.on_equator(90.0, reach=2.0)
    assert (round(x, 9), round(y, 9), round(z, 9)) == (100.0, 20.0, 0.0)


def test_a_sphere_has_a_default_mesh_detail_by_style():
    assert science.Sphere((0, 0, 0), 1.0, style="wire").levels == 2
    assert science.Sphere((0, 0, 0), 1.0, style="solid").levels == 3
    assert science.Sphere((0, 0, 0), 1.0, detail=1).levels == 1


def test_a_sphere_refuses_a_style_it_does_not_have():
    try:
        science.Sphere((0, 0, 0), 1.0, style="shiny")
    except ValueError:
        return
    raise AssertionError("an unknown style should be refused")


# ------------------------------------------------------------------- Orbit


def test_an_orbit_is_closest_at_zero_and_furthest_at_half_a_turn():
    orbit = science.Orbit(a=1000.0, e=0.2)
    near = math.dist(orbit.point(0.0), orbit.around)
    far = math.dist(orbit.point(180.0), orbit.around)
    assert abs(near - 800.0) < 1e-6, "a(1 - e)"
    assert abs(far - 1200.0) < 1e-6, "a(1 + e)"


def test_periapsis_and_node_turn_the_orbit_without_changing_its_shape():
    plain = science.Orbit(a=1000.0, e=0.2)
    turned = plain.moved(periapsis=103.0)
    x, y, _ = turned.point(0.0)
    assert abs(math.degrees(math.atan2(y, x)) - 103.0) < 1e-6, "closest approach turns"
    assert abs(math.hypot(x, y) - 800.0) < 1e-6, "and is as close as ever"
    tilted = plain.moved(inclination=10.0, node=90.0)
    x, y, z = tilted.point(90.0)       # a quarter round: the furthest from the nodes
    assert abs(z) > 100.0, "still leaves the plane"
    assert abs(tilted.point(0.0)[0]) < 1e-6 and tilted.point(0.0)[1] > 0, \
        "the line of nodes now points along +y"


def test_the_body_orbited_is_at_a_focus_not_the_middle():
    orbit = science.Orbit(a=1000.0, e=0.5, around=(5.0, 6.0, 7.0))
    path = orbit.path(720)
    nearest = min(math.dist(p, orbit.around) for p in path)
    farthest = max(math.dist(p, orbit.around) for p in path)
    assert abs(nearest - 500.0) < 1.0 and abs(farthest - 1500.0) < 1.0
    middle = tuple(sum(p[k] for p in path[:-1]) / (len(path) - 1) for k in range(3))
    assert abs(middle[0] - (5.0 - 500.0)) < 1.0, "the centre is off to one side"


def test_a_body_goes_quicker_near_the_sun_than_far_from_it():
    """Equal steps of mean anomaly are equal steps of time. Kepler's second
    law says the ground covered in them is not equal."""
    orbit = science.Orbit(a=1000.0, e=0.5)
    near = math.dist(orbit.point(5.0), orbit.point(0.0))
    far = math.dist(orbit.point(185.0), orbit.point(180.0))
    assert near > far * 2, (near, far)


def test_a_circle_is_stepped_evenly():
    orbit = science.Orbit(a=100.0)
    steps = [math.dist(orbit.point(a + 10), orbit.point(a)) for a in (0, 90, 200)]
    assert max(steps) - min(steps) < 1e-9
    assert all(abs(math.dist(orbit.point(a), orbit.around) - 100) < 1e-9
               for a in (0, 33, 140))


def test_an_inclined_orbit_leaves_the_plane():
    orbit = science.Orbit(a=100.0, inclination=30.0)
    top = max(p[2] for p in orbit.path())
    assert abs(top - 50.0) < 0.1, "100 * sin 30"


def test_an_orbit_path_closes_on_itself():
    path = science.Orbit(a=100.0, e=0.3).path(90)
    assert len(path) == 91
    assert math.dist(path[0], path[-1]) < 1e-9


def test_an_orbit_refuses_what_is_not_an_ellipse():
    for bad in ({"a": 0.0}, {"a": 10.0, "e": 1.0}, {"a": 10.0, "e": -0.1}):
        try:
            science.Orbit(**bad)
        except ValueError:
            continue
        raise AssertionError(f"{bad} should have been refused")


# ------------------------------------------------------------------- World


def _world(**kw):
    cm.canvas(1280, 720)
    return science.World(science.Camera(), **kw)


def test_a_world_names_every_shape_and_names_none_twice():
    scene = cm.Scene()
    science.World(science.Camera()).draw(
        scene, {"a": science.Sphere((0, 0, 0), 100.0, detail=1),
                "b": science.Sphere((400, 0, 0), 50.0, style="wire", detail=1)})
    names = list(_shapes(scene))
    assert len(names) == len(set(names)) and names
    assert all(n.startswith("world/") for n in names)


def test_only_the_faces_that_look_at_the_camera_are_drawn():
    scene = cm.Scene()
    _world().draw(scene, {"a": science.Sphere((0, 0, 0), 100.0, detail=2)})
    total = len(science.icosphere(2)[1])
    shown = len(_by_kind(scene, "polygon"))
    assert 0.4 * total < shown < 0.6 * total, f"{shown} of {total}"


def test_the_faces_drawn_are_the_near_half_not_just_half_of_them():
    """Half of them is easy to get right with the wrong half — the far side, seen
    through the front. Depth says which half it was."""
    world = _world()
    ball = science.Sphere((0, 0, 0), 100.0, detail=2)
    centre_depth = world.camera.project(ball.centre)[1]
    items = world._body(science.Light(), "a", ball)
    assert items and all(depth < centre_depth for depth, *_ in items), \
        "every face drawn is nearer than the middle of the ball"


def test_a_wire_sphere_draws_every_edge_so_it_is_hollow():
    scene = cm.Scene()
    wire = science.Sphere((0, 0, 0), 100.0, style="wire", detail=2)
    _world().draw(scene, {"a": wire})
    assert len(_by_kind(scene, "line")) == len(science.icosphere(2)[2])


def test_a_nearer_body_is_painted_over_a_further_one():
    cam = science.Camera()
    pos = cam.position
    unit = tuple(p / math.sqrt(sum(q * q for q in pos)) for p in pos)
    near = science.Sphere(tuple(300 * u for u in unit), 100.0, detail=1)
    far = science.Sphere(tuple(-300 * u for u in unit), 100.0, detail=1)
    scene = cm.Scene()
    science.World(cam).draw(scene, {"far": far, "near": near})
    layers = {"far": [], "near": []}
    for name, shape in _shapes(scene).items():
        layers[name.split("/")[1]].append(shape.layer)
    assert max(layers["far"]) < min(layers["near"]), \
        "every face of the far body sits under every face of the near one"


def test_each_shape_gets_its_own_layer_and_the_next_free_one_is_returned():
    scene = cm.Scene()
    ball = science.Sphere((0, 0, 0), 50.0, detail=1)
    free = _world(layer=50).draw(scene, {"a": ball})
    layers = sorted(s.layer for s in _shapes(scene).values())
    assert layers == list(range(51, 51 + len(layers)))
    assert free == 51 + len(layers)


def test_a_flat_body_is_one_colour_and_a_solid_one_is_shaded():
    flat, solid = cm.Scene(), cm.Scene()
    world = _world()
    sun = science.Sphere((0, 0, 0), 100.0, "#ffcc33", "flat", detail=2)
    earth = science.Sphere((0, 0, 0), 100.0, "#58c4dd", detail=2)
    world.draw(flat, {"sun": sun})
    world.draw(solid, {"earth": earth})
    assert {s.color for s in _by_kind(flat, "polygon")} == {"#ffcc33"}
    assert len({s.color for s in _by_kind(solid, "polygon")}) > 5


def test_the_side_facing_the_light_is_brighter():
    cam = science.Camera()
    toward_camera = tuple(p / math.sqrt(sum(q * q for q in cam.position))
                          for p in cam.position)
    scene = cm.Scene()
    science.World(cam, light=science.Light(direction=toward_camera)).draw(
        scene, {"a": science.Sphere((0, 0, 0), 100.0, "#ffffff", detail=3)})

    def bright(shape):
        return sum(int(shape.color[i:i + 2], 16) for i in (1, 3, 5))

    def off_centre(shape):
        xs, ys = shape.points[0::2], shape.points[1::2]
        return math.hypot(sum(xs) / len(xs) - 640, sum(ys) / len(ys) - 360)

    faces = sorted(_by_kind(scene, "polygon"), key=off_centre)
    assert bright(faces[0]) > bright(faces[-1]) + 150, "lit in the middle, dark rim"


def test_light_from_a_point_moves_the_lit_side_with_the_body():
    """A planet right of its sun is lit on its left, and the other way round."""
    cam = science.Camera(azimuth=0.0, elevation=0.0)   # space +y is screen right

    def lit_side(centre):
        scene = cm.Scene()
        science.World(cam, light=science.Light(source=(0.0, 0.0, 0.0))).draw(
            scene, {"p": science.Sphere(centre, 100.0, "#ffffff", detail=3)})
        brightest = max(_by_kind(scene, "polygon"),
                        key=lambda s: sum(int(s.color[i:i + 2], 16) for i in (1, 3, 5)))
        x = sum(brightest.points[0::2]) / (len(brightest.points) / 2)
        return x - cam.at(centre)[0]

    assert lit_side((0.0, 600.0, 0.0)) < -20, "planet on the right, lit on its left"
    assert lit_side((0.0, -600.0, 0.0)) > 20, "planet on the left, lit on its right"


def test_an_orbit_is_cut_where_it_would_run_across_the_body_it_circles():
    body = science.Sphere((0.0, 0.0, 0.0), 160.0, detail=1)
    world = _world()
    plain, cut = cm.Scene(), cm.Scene()
    ring = science.Orbit(a=170.0, color="#4a5568")
    world.draw(plain, {"e": body}, {"ring": ring})
    world.draw(cut, {"e": body}, {"ring": ring.moved(hide=("e",))})
    drawn = lambda s: [x for x in _shapes(s) if "/ring/" in x]  # noqa: E731
    assert 0 < len(drawn(cut)) < len(drawn(plain))


# -------------------------------------------------------------- RingArrows


def test_arrows_round_a_body_seen_from_above_are_all_in_sight():
    scene = cm.Scene()
    world = science.World(science.Camera(elevation=90.0))
    body = science.Sphere((0.0, 0.0, 0.0), 100.0)
    shown = science.RingArrows(count=6).draw(scene, world, "spin", body, 10.0)
    assert shown == 6


def test_arrows_round_a_body_seen_from_the_side_lose_the_ones_behind_it():
    scene = cm.Scene()
    world = science.World(science.Camera(elevation=0.0))
    body = science.Sphere((0.0, 0.0, 0.0), 100.0)
    shown = science.RingArrows(count=6).draw(scene, world, "spin", body, 0.0)
    assert 0 < shown < 6, shown
    assert len(_by_kind(scene, "polygon")) == shown, "an arrow is one shape"


def test_ring_arrows_default_to_above_the_world():
    scene = cm.Scene()
    world = science.World(science.Camera(elevation=90.0))
    body = science.Sphere((0, 0, 0), 50.0)
    science.RingArrows(count=2).draw(scene, world, "spin", body, 0.0)
    assert all(s.layer == science.ABOVE for s in _shapes(scene).values())


# ------------------------------------------------------------------ marks


def test_a_tag_anchored_by_its_top_has_the_plate_top_there():
    if not _engine():
        return
    box = science.tag("density", at=cm.at(x=300, top=100), formula=False)
    assert abs(box.top - 100) < 1e-9 and box.x == 300
    scene = cm.Scene()
    box.draw(scene, "d")
    plate, text = _shapes(scene)["d/plate"], _shapes(scene)["d/text"]
    assert plate.y == text.y == box.y, "one shared centre for the plate and its text"


def test_a_bare_tag_draws_only_its_text_and_takes_the_same_room():
    if not _engine():
        return
    box = science.tag("density", at=cm.at(x=300, top=100), formula=False)
    bare = box.bare()
    scene = cm.Scene()
    bare.draw(scene, "d")
    assert sorted(_shapes(scene)) == ["d/text"], "no plate"
    assert (bare.x, bare.y, bare.w, bare.h) == (box.x, box.y, box.w, box.h)


def test_a_tag_is_its_content_plus_padding():
    if not _engine():
        return
    small = science.tag("hi", at=(0, 0), formula=False)
    big = science.tag("hello there", at=(0, 0), formula=False)
    wide, high = cm.measure("hi", 20)
    assert small.w > wide and small.h > high, "padding on every side"
    assert big.w > small.w


def test_a_formula_tag_is_measured_as_a_formula():
    if not _engine() or not shutil.which("typst"):
        return
    box = science.tag(r"\tau_{\text{wire}}", at=(100, 100))
    wide, high = cm.measure_math(r"\tau_{\text{wire}}", 20)
    assert box.w > wide and box.h > high


def test_a_tags_edges_are_the_middles_of_its_sides():
    box = science.Tag("x", x=100.0, y=50.0, w=40.0, h=20.0)
    assert box.edge("left") == (80.0, 50.0)
    assert box.edge("right") == (120.0, 50.0)
    assert box.edge("top") == (100.0, 40.0)
    assert box.edge("bottom") == (100.0, 60.0)
    try:
        box.edge("middle")
    except ValueError:
        return
    raise AssertionError("an unknown edge should be refused")


def test_a_leader_ends_on_the_plate_not_near_it():
    box = science.Tag("x", x=100.0, y=50.0, w=40.0, h=20.0)
    scene = cm.Scene()
    box.leader(scene, "pointer", start=(10.0, 200.0), side="left")
    line = _shapes(scene)["pointer"]
    assert (line.x, line.y) == (10.0, 200.0)
    assert (line.x2, line.y2) == (80.0, 50.0)


def test_a_tag_already_clear_of_another_does_not_move():
    a = science.Tag("a", x=100.0, y=100.0, w=60.0, h=24.0)
    b = science.Tag("b", x=300.0, y=100.0, w=60.0, h=24.0)
    moved = b.clear_of(a)
    assert (moved.x, moved.y) == (300.0, 100.0)


def test_overlapping_tags_are_pushed_apart_with_a_gap_between():
    a = science.Tag("a", x=100.0, y=100.0, w=60.0, h=24.0)
    b = science.Tag("b", x=120.0, y=110.0, w=60.0, h=24.0)
    moved = b.clear_of(a, gap=6.0)
    apart_x = abs(moved.x - a.x) - (a.w + b.w) / 2
    apart_y = abs(moved.y - a.y) - (a.h + b.h) / 2
    assert max(apart_x, apart_y) >= 6.0 - 1e-9, "clear of it, with the gap"
    assert (moved.w, moved.h, moved.content) == (b.w, b.h, b.content), "only moved"
    # Pushed along the line from the other's centre, the shortest honest way.
    assert (moved.x - a.x) * (b.y - a.y) - (moved.y - a.y) * (b.x - a.x) < 1e-6


def test_a_tag_slides_round_another_instead_of_jumping():
    """A label that follows a body must not pop when it meets another. Sweep one
    across the other a pixel at a time. The result moves about as far as the
    sweep does — more only when the pass is nearly dead centre, and then by
    (half the heights + gap) over how far off-centre it passes, never a jump."""
    anchor = science.Tag("a", x=200.0, y=100.0, w=80.0, h=24.0)
    for off_centre in (12.0, 20.0, 30.0, 60.0):
        last, worst = None, 0.0
        for i in range(0, 241):
            mover = science.Tag("m", x=80.0 + i, y=100.0 + off_centre,
                                w=60.0, h=24.0).clear_of(anchor)
            if last is not None:
                worst = max(worst, math.hypot(mover.x - last[0], mover.y - last[1]))
            last = (mover.x, mover.y)
        assert worst <= max(1.0, 30.0 / off_centre) + 0.05, (off_centre, worst)


def test_the_first_tag_given_is_the_one_that_stays():
    a = science.Tag("a", x=100.0, y=100.0, w=60.0, h=24.0)
    b = science.Tag("b", x=100.0, y=100.0, w=60.0, h=24.0)
    c = science.Tag("c", x=105.0, y=102.0, w=60.0, h=24.0).clear_of(a, b)
    for other in (a, b):
        assert (abs(c.x - other.x) >= (c.w + other.w) / 2
                or abs(c.y - other.y) >= (c.h + other.h) / 2)


def test_a_tag_needs_a_vertical_anchor():
    if not _engine():
        return
    try:
        science.tag("x", at=cm.at(x=3), formula=False)
    except ValueError:
        return
    raise AssertionError("no y, top or bottom is ambiguous, not centred")


def test_a_glow_is_a_stack_of_fainter_wider_copies_under_a_sharp_core():
    scene = cm.Scene()
    science.Glow(color="#ffd23f", steps=6).dot(scene, "star", at=(100, 100), r=10)
    halos = sorted((s for n, s in _shapes(scene).items() if "/halo/" in n),
                   key=lambda s: s.r)
    assert len(halos) == 6 and len(_shapes(scene)) == 7
    assert [h.r for h in halos] == sorted({h.r for h in halos}), "all different sizes"
    assert halos[0].opacity > halos[-1].opacity, "fainter as they widen"
    core = _shapes(scene)["star/core"]
    assert core.r == 10 and core.layer > max(h.layer for h in halos)


def test_glow_strength_scales_the_halo_and_a_ring_leaves_the_middle_empty():
    full, half = cm.Scene(), cm.Scene()
    science.Glow().dot(full, "a", at=(0, 0), r=10)
    science.Glow(strength=0.5).dot(half, "a", at=(0, 0), r=10)
    halo = lambda scene: _shapes(scene)["a/halo/4"].opacity  # noqa: E731
    assert abs(halo(half) * 2 - halo(full)) < 1e-9

    ring = cm.Scene()
    science.Glow().ring(ring, "h", at=(0, 0), r=100)
    assert all(s.color == "none" for s in _shapes(ring).values()), "nothing is filled"


def test_a_wave_leaves_one_place_and_arrives_at_another():
    pts = science.wave((100.0, 300.0), (500.0, 300.0), cycles=2.5, amp=8.0, steps=40)
    assert len(pts) == 41
    assert pts[0] == (100.0, 300.0)
    assert abs(pts[-1][0] - 500.0) < 1e-9 and abs(pts[-1][1] - 300.0) < 1e-9
    assert max(abs(y - 300.0) for _, y in pts) <= 8.0 + 1e-9
    assert max(abs(y - 300.0) for _, y in pts) > 7.5, "it does reach its amplitude"


def test_a_wave_rides_a_slanted_line_perpendicular_to_it():
    pts = science.wave((0.0, 0.0), (300.0, 400.0), cycles=1.0, amp=10.0, steps=20)
    for x, y in pts:
        along = (x * 0.6 + y * 0.8)
        across = abs(-x * 0.8 + y * 0.6)
        assert -1e-9 <= along <= 500 + 1e-9 and across <= 10 + 1e-9


def test_a_wave_needs_somewhere_to_go():
    try:
        science.wave((1.0, 1.0), (1.0, 1.0))
    except ValueError:
        return
    raise AssertionError("a zero-length wave has no direction")


def test_a_bracket_is_a_span_two_ticks_and_a_label_on_the_chosen_side():
    if not _engine():
        return
    for side, expect in ((1, "right"), (-1, "left")):
        scene = cm.Scene()
        science.Bracket(side=side).draw(
            scene, "b", (300.0, 100.0), (300.0, 300.0), "0.9 V")
        shapes = _shapes(scene)
        assert {"b/span", "b/tick/0", "b/tick/1"} <= set(shapes)
        plate = shapes["b/label/0/plate"]
        assert (plate.x > 300) == (expect == "right"), (side, plate.x)
        assert abs(plate.y - 200) < 1e-6, "level with the middle of the span"


def test_a_horizontal_bracket_puts_its_label_above_for_a_positive_side():
    if not _engine():
        return
    scene = cm.Scene()
    science.Bracket(side=1).draw(scene, "b", (100.0, 400.0), (500.0, 400.0), "r")
    assert _shapes(scene)["b/label/0/plate"].y < 400


def test_a_bracket_stacks_a_label_of_several_lines():
    if not _engine():
        return
    scene = cm.Scene()
    science.Bracket().draw(scene, "b", (300.0, 100.0), (300.0, 300.0), ["one", "two"])
    a, b = _shapes(scene)["b/label/0/plate"], _shapes(scene)["b/label/1/plate"]
    assert b.y > a.y and abs(a.x - b.x) < 1e-6


def test_a_bracket_needs_a_span():
    try:
        science.Bracket().draw(cm.Scene(), "b", (1.0, 1.0), (1.0, 1.0))
    except ValueError:
        return
    raise AssertionError("a bracket of nothing measures nothing")


def test_a_force_scale_is_one_linear_scale_with_a_ceiling():
    scale = science.ForceScale(px_per_unit=0.5, cap=100.0)
    assert scale.length(60.0) == (30.0, False)
    assert scale.length(-60.0) == (30.0, False), "a size, whatever its sign"
    assert scale.length(1000.0) == (100.0, True)
    assert science.ForceScale(px_per_unit=2.0).length(1000.0) == (2000.0, False)


def test_a_force_arrow_points_the_way_the_vector_does_with_y_up():
    scale = science.ForceScale(px_per_unit=1.0)
    for vector, where in (((0.0, 80.0), (400.0, 220.0)),
                          ((0.0, -80.0), (400.0, 380.0)),
                          ((80.0, 0.0), (480.0, 300.0))):
        scene = cm.Scene()
        tip = scale.arrow(scene, "f", at=(400.0, 300.0), vector=vector)
        assert tuple(round(c, 6) for c in tip) == where, (vector, tip)
        pts = _shapes(scene)["f"].points
        assert (round(pts[6], 6), round(pts[7], 6)) == where, "the head is the tip"


def test_a_capped_arrow_is_cut_at_the_cap_and_marked():
    scale = science.ForceScale(px_per_unit=1.0, cap=100.0)
    capped, plain = cm.Scene(), cm.Scene()
    tip = scale.arrow(capped, "f", at=(400.0, 300.0), vector=(0.0, 500.0))
    scale.arrow(plain, "f", at=(400.0, 300.0), vector=(0.0, 60.0))
    assert abs(tip[1] - 200.0) < 1e-9, "drawn 100 long, not 500"
    assert {"f/cut/0", "f/cut/10"} <= set(_shapes(capped))
    assert not any("/cut/" in n for n in _shapes(plain)), "only a capped one is broken"


def test_a_force_too_small_to_draw_draws_nothing():
    scene = cm.Scene()
    assert science.ForceScale(px_per_unit=0.01).arrow(
        scene, "f", at=(0.0, 0.0), vector=(0.0, 10.0)) is None
    assert not _shapes(scene)


def test_a_force_label_is_set_above_and_beside_the_tip():
    if not _engine() or not shutil.which("typst"):
        return
    scene = cm.Scene()
    tip = science.ForceScale(px_per_unit=1.0).arrow(
        scene, "f", at=(400.0, 300.0), vector=(0.0, -100.0), label="F_B")
    label = _shapes(scene)["f/label"]
    assert label.y < tip[1] and label.x > tip[0]


def test_the_kit_is_reachable_from_the_package():
    assert cm.science is science
    assert "science" in cm.__all__


if __name__ == "__main__":
    import sys

    sys.exit(support.run(globals()))


def test_a_star_is_dimmer_at_the_rim_and_its_mottle_turns_with_spin():
    def colours(spin):
        scene = cm.Scene()
        sun = science.Sphere((0, 0, 0), 100.0, "#ff8c1a", "star", spin=spin, detail=2)
        science.World(science.Camera(), light=science.Light()).draw(scene, {"sun": sun})
        return {s.item: s.color for s in _by_kind(scene, "polygon")}

    a, b = colours(0.0), colours(40.0)
    assert len(set(a.values())) > 5, "not one flat colour"
    assert a != b, "the surface turns"
