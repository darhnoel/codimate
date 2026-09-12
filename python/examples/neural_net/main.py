"""A signal moving forward through a neural network.

    python python/examples/neural_net/main.py

Nothing on screen moves house — neurons stay where they are. So the names here
follow the **place**, not the thing: `("neuron", layer, index)`. The only
things that travel are the pulses, and they travel because the trace says
where they are at each moment, not because anything asked for an animation.
"""

import codimate as cm

LAYERS = (3, 4, 2)
COLUMN_X = (260, 640, 1020)

DIM = "#243044"
LIVE = "orange"
RESTING = "#1b2436"
CHARGED = "#4a9eff"


# --- the state --------------------------------------------------------------


class Net:
    """Who has fired, what is carrying signal, and where that signal is."""

    def __init__(self, layers):
        self.layers = layers
        self.fired = set()      # (layer, index)
        self.carrying = set()   # ((layer, i), (layer + 1, j))
        self.signal_at = None   # None | "source" | "target"

    def inputs_of(self, node):
        layer, _ = node
        return [(layer - 1, i) for i in range(self.layers[layer - 1])]

    def every_edge(self):
        for layer in range(len(self.layers) - 1):
            for i in range(self.layers[layer]):
                for j in range(self.layers[layer + 1]):
                    yield ((layer, i), (layer + 1, j))


# --- the algorithm ----------------------------------------------------------


@cm.trace()
def forward_pass(net):
    for i in range(net.layers[0]):
        net.fired.add((0, i))
    cm.emit("inputs")

    for layer in range(1, len(net.layers)):
        for j in range(net.layers[layer]):
            node = (layer, j)

            net.carrying = {(src, node) for src in net.inputs_of(node)}
            net.signal_at = "source"
            cm.emit("send", node=node)

            net.signal_at = "target"
            net.fired.add(node)
            cm.emit("arrive", node=node)

    net.carrying = set()
    net.signal_at = None
    cm.emit("done")


# --- the view ---------------------------------------------------------------


def places(net):
    """One Slot per neuron, keyed by where it is — neurons do not move."""
    return {
        (layer, i): slot
        for layer, count in enumerate(net.layers)
        for i, slot in enumerate(cm.column(count, gap=44, w=64, x=COLUMN_X[layer]))
    }


def network(frame):
    scene = cm.Scene()
    net = frame.state
    at = places(net)

    scene.text("title", "Forward Pass", x=cm.width() / 2, y=80, size=40, color="grey")

    for src, dst in net.every_edge():
        live = (src, dst) in net.carrying
        scene.line(("edge", src, dst), start=at[src], end=at[dst],
                   w=3.0 if live else 1.0, color=LIVE if live else DIM)

    for node, slot in at.items():
        charged = node in net.fired
        scene.group(("neuron", node), slot).circle(
            "body", r=30, color=CHARGED if charged else RESTING, layer=5)

    # A pulse sits at the source end of its edge, then at the target end. It
    # exists in both moments, so the Engine makes it travel. Nothing here
    # mentions movement.
    for src, dst in net.carrying:
        here = at[src] if net.signal_at == "source" else at[dst]
        scene.circle(("pulse", src, dst), x=here.x, y=here.y, r=9, color=LIVE, layer=9)

    return scene


# --- motion and timing ------------------------------------------------------

# No motion rules: a straight line is the default, and it is what a signal
# travelling down a wire should do.
cm.explain(
    trace=forward_pass(Net(LAYERS)),
    view=network,
    timing=cm.Timing(
        default=0.4,
        events={"inputs": 0.7, "send": 0.18, "arrive": 0.55, "done": 0.7},
        opening=0.9,
        final_hold=1.6,
    ),
).render("results/neural_net.mp4", fps=60, scale=1.5)

print("wrote results/neural_net.mp4")
