# Attention, on real GPT-2 weights

```bash
.venv/bin/python python/examples/attention/main.py
```

The mechanism the paper is named for:

```text
Attention(Q, K, V) = softmax( Q Kᵀ / √dₖ ) V
```

The animation follows one query — the word "sat" — through the left half of
that formula, then fills in the rest of the matrix, then runs the same
arithmetic on a different head.

## The numbers are real

The Q and K vectors in `weights.py` came out of an actual GPT-2, after a full
forward pass through the four layers beneath layer 4. Every number on screen is
worked out from them at render time: the dot products, the scaling, the mask
and the softmax. Nothing is illustrative and nothing is a stored result.

`extract.py` is the script that produced them. It needs the network and numpy
once; the example itself needs neither, and `attention.py` imports nothing but
`math`. It does not download GPT-2 — `safetensors` stores a JSON header of byte
offsets, so the tensors that matter arrive by HTTP range request: about 7 MB of
a 500 MB file.

## What it teaches

**Attention lets a word look back at the word it depends on.** Layer 4, head 3
learned to find the subject. "sat" attends **0.96** to "cat"; "on" attends
0.89 to it; by the end the whole `cat` column is lit. Nobody told it to do
that — it is what the weights do on this sentence.

**The ÷√dₖ is the formula's contribution, and it is not decoration.** The
animation shows the same row computed both ways:

| | The | cat | sat |
|---|---|---|---|
| Q · K | −13.61 | 14.04 | −22.26 |
| ÷ √64 | −1.70 | 1.76 | −2.78 |
| softmax | 0.03 | **0.96** | 0.01 |
| **without the division** | 0.00 | **1.00** | 0.00 |

Without the scaling the row saturates to one-hot, and almost no gradient is
left to learn from. With it, the head is confident but still differentiable.

**A token cannot see what comes after it.** The upper triangle stays dark
throughout. GPT-2 is causal, which is the "masked" attention of the paper's
decoder, and it is why row *n* has *n+1* numbers rather than six.

**Different heads learn completely different things.** The coda re-runs the
same code on head 11 of the same layer. It has learned nothing about subjects:
it attends one token backwards, every time, at 1.00. That is a real component
of induction heads — and it is the argument for having many heads, made without
having to explain multi-head attention.

## Two things the animation had to be built around

**Every act moves, then holds still.** Text cannot be interpolated, only
swapped, so the Engine shows a shape's previous text for the whole of a
transition. If every moment is a transition, a caption spends its whole life
describing the act you have just left. Emitting each act twice — a short move
and a longer hold — lets the scene settle, and then everything on it agrees.

**One act owns the screen.** The working-out and the matrix never appear
together. An earlier version had the sentence, the formula, the grid, the
labels and the arithmetic all up at once, which is a lot of places to look and
no reason to pick one.

## What this deliberately does not cover

The `× V` that turns attention weights into an output vector, multi-head
attention, positional encoding, and the encoder-decoder architecture. V is
64-dimensional, so the weighted sum can only be drawn as strips blending —
a picture of arithmetic rather than arithmetic you can check.

## Finding the head took looking

Layer 0 is the obvious place to start and it is the wrong one. Its heads attend
almost entirely to themselves, or to the first token — the "attention sink".
Real behaviour, but it says nothing about meaning. The heads used here were
found by running all twelve layers and reading off which tokens attended to
which.

```text
weights.py     Q and K, extracted once
extract.py     the script that produced them — network and numpy, once
attention.py   the arithmetic — imports only `math`
layout.py      where things sit
view.py        the drawing
```

## Try changing

| Change | What happens |
|---|---|
| `walk.query = 5` | follow "mat" instead; its attention is far more diffuse |
| `scaled=False` in `A.row` | the whole matrix collapses to one-hot |
| drop the mask in `raw_scores` | tokens see the future, and the causal story disappears |
| a different head in `extract.py` | most heads are far less tidy than these two |
