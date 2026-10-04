"""The pen a scene draws with: a `Look` says how, a `Stage` puts shapes on the
screen under names that carry the scene, and fades them in and out."""

from dataclasses import dataclass

from palette import INK


@dataclass(frozen=True)
class Look:
    """How something is drawn. Grouped because it always travels together:
    colour, opacity, stroke, a type size, an arrow's head, and — for a formula —
    how much of it is written on. `edge` draws a ring instead of a disc."""

    color: str = INK
    op: float = 1.0
    w: float = 3.0
    layer: int = None
    size: float = 24.0
    head: float = 18.0
    edge: str = ""
    reveal: float = 1.0


def clamp(x, lo=0.0, hi=1.0):
    return max(lo, min(hi, x))


def ramp(u, a, b):
    """0 until `a`, 1 from `b`, easing in between."""
    x = clamp((u - a) / (b - a))
    return x * x * (3 - 2 * x)


class Stage:
    """One scene's pen. Names carry the scene, so a shape that is in scene 3 is
    not mistaken for one in scene 4; opacity carries the fade in and out."""

    def __init__(self, scene, k, op):
        self.scene, self.k, self.op = scene, k, op

    def _key(self, name):
        return (f"s{self.k}", name)

    def _done(self, handle, look):
        return handle.on(layer=look.layer, opacity=clamp(look.op) * self.op)

    def _paint(self, handle, look):
        if look.edge:
            handle.fill("none", edge=look.edge, edge_w=look.w)
        else:
            handle.fill(look.color)
        return self._done(handle, look)

    def circle(self, name, at, r, look):
        return self._paint(self.scene.circle(self._key(name), r=r, at=at), look)

    def rect(self, name, at, size, look):
        w, h = size
        return self._paint(self.scene.rect(self._key(name), w=w, h=h, at=at), look)

    def ring(self, name, at, radii, look):
        """An ellipse outline: a loop of wire, a field line round a current."""
        handle = self.scene.arc(self._key(name), r=radii, sweep=(0, 360), at=at)
        return self._done(handle.fill("none", edge=look.color, edge_w=look.w), look)

    def disc(self, name, at, radii, look):
        """An ellipse filled in: a surface stretched across a loop."""
        handle = self.scene.arc(self._key(name), r=radii, sweep=(0, 360), at=at)
        return self._done(handle.fill(look.color).round(1), look)

    def line(self, name, a, b, look):
        return self._done(
            self.scene.line(self._key(name), start=a, end=b, w=look.w).fill(look.color),
            look,
        )

    def arrow(self, name, a, b, look):
        return self._done(
            self.scene.arrow(
                self._key(name), start=a, end=b, w=look.w, head=look.head
            ).fill(look.color),
            look,
        )

    def poly(self, name, points, look):
        return self._paint(self.scene.polygon(self._key(name), points), look)

    def curve(self, name, points, look):
        return self._done(
            self.scene.curve(self._key(name), points, w=look.w).fill(look.color), look
        )

    def text(self, name, content, at, look):
        return self._done(
            self.scene.text(self._key(name), content, size=look.size, at=at).fill(
                look.color
            ),
            look,
        )

    def formula(self, name, latex, at, look):
        return self._done(
            self.scene.formula(self._key(name), latex, size=look.size, at=at)
            .fill(look.color)
            .write(reveal=look.reveal),
            look,
        )
