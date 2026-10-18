"""Putting it together: motion rules, timing, and the render call.

The parts:

- `timing` — Motion rules and timing: how a change moves, and how long it takes.
- `explanation` — The Explanation: the moments, put together, and the render call.
- `encoder` — Finding ffmpeg, and mixing a film's sounds for it.
"""

from typing import Callable

from .timing import (
    PATHS, Rule, Timing, ease,
)
from .explanation import (
    View, INDEX_ENTRY_BYTES, _Captured, _quoted, _plain, Explanation,
    explain,
)
from .encoder import (
    _mixing, _ffmpeg, _find_encoder,
)

__all__ = [
    'Explanation', 'INDEX_ENTRY_BYTES', 'PATHS', 'Rule', 'Timing', 'View',
    '_Captured', '_ffmpeg', '_find_encoder', '_mixing', '_plain',
    '_quoted', 'ease', 'explain',
]

# Set by the Previewer while it runs a script: `render` hands the Explanation
# here and stops the script, instead of drawing a video nobody asked for yet.
_capture: "Callable[[Explanation], None] | None" = None
