"""The scenes, in the order the film shows them.

Each module has one function, `draw(s, u, t, D)`, that draws a single moment: `s` is
the pen, `u` how far through the scene it is (0 to 1), `t` the seconds since it began
and `D` its length. The number beside each scene is the least time its picture needs;
the film runs longer if the line takes longer to say."""

from . import ampere, constants, faraday, field, hole, patch, question, speed, wave

SCENES = (
    ("question", question.draw, 9.0),
    ("ampere", ampere.draw, 9.0),
    ("faraday", faraday.draw, 10.0),
    ("field", field.draw, 9.0),
    ("hole", hole.draw, 10.0),
    ("patch", patch.draw, 10.0),
    ("wave", wave.draw, 13.0),
    ("constants", constants.draw, 13.0),
    ("speed", speed.draw, 12.0),
)
