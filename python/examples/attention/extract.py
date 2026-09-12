"""Pull Q and K for one GPT-2 head out of the real model. Run once.

    pip install numpy
    python python/examples/attention/extract.py > weights.py

This needs the network and numpy. Nothing else in this example does — it is
here so the numbers in `weights.py` can be checked rather than believed.

It does not download GPT-2. `safetensors` stores a JSON header of byte offsets,
so the tensors that matter can be fetched with HTTP range requests: about 7 MB
of a 500 MB file, and individual embedding rows rather than the whole table.

Layer 4 is not arbitrary. Layer 0 attention turns out to be nearly all
self-attention and attention to the first token — real behaviour, but it says
nothing about meaning. The interpretable heads were found by running all twelve
layers and looking at which tokens attended to which.
"""

import json
import math
import struct
import urllib.request

import numpy as np

BASE = "https://huggingface.co/gpt2/resolve/main"
SAFETENSORS = f"{BASE}/model.safetensors"

SENTENCE = ["The", " cat", " sat", " on", " the", " mat"]
LAYER = 4
PICKED = {3: ("SUBJECT", "attends to the subject"),
          11: ("PREVIOUS", "attends one token back")}
D, D_K, N_HEADS = 768, 64, 12


def fetch(url, start=None, end=None):
    headers = {"Range": f"bytes={start}-{end}"} if start is not None else {}
    return urllib.request.urlopen(
        urllib.request.Request(url, headers=headers), timeout=120).read()


header_size = struct.unpack("<Q", fetch(SAFETENSORS, 0, 7))[0]
META = json.loads(fetch(SAFETENSORS, 8, 8 + header_size - 1))
DATA = 8 + header_size


def tensor(name, rows=None):
    info = META[name]
    shape, (a, b) = info["shape"], info["data_offsets"]
    if rows is None:
        return np.frombuffer(fetch(SAFETENSORS, DATA + a, DATA + b - 1),
                             dtype="<f4").reshape(shape)
    width = shape[1]
    return np.stack([
        np.frombuffer(fetch(SAFETENSORS, DATA + a + r * width * 4,
                            DATA + a + (r + 1) * width * 4 - 1), dtype="<f4")
        for r in rows])


def layer_norm(x, gain, bias):
    mean, spread = x.mean(-1, keepdims=True), x.std(-1, keepdims=True)
    return gain * (x - mean) / (spread + 1e-5) + bias


def main():
    vocab = json.loads(fetch(f"{BASE}/vocab.json"))
    ids = [vocab[w.replace(" ", "Ġ")] for w in SENTENCE]
    n = len(ids)

    x = tensor("wte.weight", ids) + tensor("wpe.weight", list(range(n)))

    saved = {}
    for layer in range(LAYER + 1):
        p = f"h.{layer}."
        h = layer_norm(x, tensor(p + "ln_1.weight"), tensor(p + "ln_1.bias"))
        qkv = h @ tensor(p + "attn.c_attn.weight") + tensor(p + "attn.c_attn.bias")
        Q, K, V = qkv[:, :D], qkv[:, D:2 * D], qkv[:, 2 * D:]

        outs = []
        for head in range(N_HEADS):
            s = slice(head * D_K, (head + 1) * D_K)
            if layer == LAYER and head in PICKED:
                saved[head] = (Q[:, s], K[:, s])
            scores = Q[:, s] @ K[:, s].T / math.sqrt(D_K)
            scores = np.where(np.tril(np.ones((n, n))) == 1, scores, -1e9)
            a = np.exp(scores - scores.max(-1, keepdims=True))
            outs.append((a / a.sum(-1, keepdims=True)) @ V[:, s])

        x = x + np.concatenate(outs, -1) @ tensor(p + "attn.c_proj.weight") \
            + tensor(p + "attn.c_proj.bias")
        h2 = layer_norm(x, tensor(p + "ln_2.weight"), tensor(p + "ln_2.bias"))
        f = h2 @ tensor(p + "mlp.c_fc.weight") + tensor(p + "mlp.c_fc.bias")
        f = 0.5 * f * (1 + np.tanh(math.sqrt(2 / math.pi) * (f + 0.044715 * f ** 3)))
        x = x + f @ tensor(p + "mlp.c_proj.weight") + tensor(p + "mlp.c_proj.bias")

    rows = lambda m: "\n".join(
        "    (" + ", ".join(f"{v:.4f}" for v in r) + ")," for r in m)

    print('"""Q and K from a real GPT-2, layer 4. Produced by extract.py."""')
    print(f"\nWORDS = {tuple(w.strip() for w in SENTENCE)!r}")
    print(f"\nD_K = {D_K}")
    for head, (name, note) in PICKED.items():
        Q, K = saved[head]
        print(f"\n# layer {LAYER}, head {head} — {note}")
        print(f"Q_{name} = (\n{rows(Q)}\n)")
        print(f"K_{name} = (\n{rows(K)}\n)")


if __name__ == "__main__":
    main()
