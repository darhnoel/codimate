"""Scaled dot-product attention, on real GPT-2 weights.

    python python/examples/attention/main.py

    Attention(Q, K, V) = softmax( Q Kᵀ / √dₖ ) V

The animation follows one query — the word "sat" — through the left half of
that formula, then fills in the rest of the matrix.

Nothing here is illustrative. The Q and K vectors in `weights.py` were taken
from a real GPT-2 after a full forward pass to layer 4, and every number on
screen is worked out from them at render time. Head 3 of that layer learned to
look back at the subject of a sentence: "sat" attends 0.96 to "cat". Head 11,
right beside it, learned only ever to look one token back.

    weights.py     Q and K, extracted once from GPT-2
    attention.py   the arithmetic — no Codimate in it
    layout.py      where things sit
    view.py        the drawing
"""

from dataclasses import dataclass, field

import codimate as cm

import attention as A
import view


# --- the state --------------------------------------------------------------


@dataclass
class Walk:
    head: str = "subject"
    stage: str = "sentence"
    query: "int | None" = None
    keys: int = 0                   # how many keys have been scored
    rows: set = field(default_factory=set)
    caption: str = ""


# --- the algorithm ----------------------------------------------------------


@cm.trace()
def explain(walk):
    """Three acts. Each owns the screen, so there is one thing to look at.

    Every act is emitted twice: once to move, once to hold still. Without the
    hold the scene is always mid-transition, and text — which cannot be
    interpolated, only swapped — spends the whole time showing its previous
    value. A short move and a long hold means each act settles and can be read.
    """

    def act(stage=None, caption=None, **rest):
        if stage:
            walk.stage = stage
        if caption is not None:
            walk.caption = caption
        for k, v in rest.items():
            setattr(walk, k, v)
        cm.emit("move")
        cm.emit("hold")

    act()

    act("formula", "we will follow the left half, for a single word")
    act("query", "a token may look at itself and at what came before it", query=2)

    walk.stage, walk.caption = "dot", "one dot product per key it can see"
    for k in range(3):
        walk.keys = k + 1
        cm.emit("score")
    cm.emit("hold")

    act("scale", "divided by √dₖ — the step the formula is named for")
    act("softmax", "softmax turns them into weights that sum to 1")
    act("compare", "without the division it saturates, and nothing can learn")

    walk.rows.add(walk.query)
    act("matrix", "“sat” attends 0.96 to “cat”")

    walk.stage, walk.caption = "rows", "every query, the same way"
    for i in range(A.N):
        walk.rows.add(i)
        cm.emit("row")
    cm.emit("hold")

    act("whole", "every word after the subject looks back at it")
    act("coda", "same sentence, same arithmetic — two heads, two different habits",
        head="previous")


# --- the view ---------------------------------------------------------------


def attention_view(frame):
    scene = cm.Scene()
    view.draw(scene, frame.state)
    return scene


cm.explain(
    trace=explain(Walk()),
    view=attention_view,
    timing=cm.Timing(
        default=2.2,
        events={"move": 0.75, "hold": 2.3, "score": 0.7, "row": 0.45},
        opening=1.4, final_hold=3.2,
    ),
).render("results/attention.mp4", fps=60, scale=1.5)

print("wrote results/attention.mp4")
