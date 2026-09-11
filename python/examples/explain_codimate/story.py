"""What is being explained: three moments of a bubble sort, and a playhead.

The concept, separate from any decision about how it looks.
"""

# Each moment is the values in order, the pair the event is about, and its name.
MOMENTS = (
    ((3, 1, 4, 2), (), "start"),
    ((3, 1, 4, 2), (3, 1), "compare"),
    ((1, 3, 4, 2), (1, 3), "swap"),
    ((1, 3, 4, 2), (3, 4), "compare"),
)
SEGMENTS = tuple((MOMENTS[i + 1][2], d) for i, d in enumerate((0.6, 0.9, 0.6)))
TOTAL = sum(d for _, d in SEGMENTS)
TICKS = 28


class Clock:
    """A playhead somewhere on a timeline of segments."""

    def __init__(self):
        self.t = 0.0

    def locate(self):
        """Which segment holds t, and how far through it?

        This is what `Explanation::resolve` does in the Engine, written out.
        """
        start = 0.0
        for index, (_, duration) in enumerate(SEGMENTS):
            if self.t < start + duration or index == len(SEGMENTS) - 1:
                return index, start, duration, (self.t - start) / duration
            start += duration
        raise AssertionError("unreachable")
