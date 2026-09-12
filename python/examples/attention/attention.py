"""Scaled dot-product attention, worked out from the committed Q and K.

    Attention(Q, K, V) = softmax( Q Kᵀ / √dₖ ) V

Everything below is the left half of that: the dot products, the scaling, the
mask and the softmax. Pure arithmetic, no Codimate, no dependencies.
"""

import math

from weights import D_K, K_PREVIOUS, K_SUBJECT, Q_PREVIOUS, Q_SUBJECT, WORDS

HEADS = {
    "subject": (Q_SUBJECT, K_SUBJECT, "layer 4, head 3"),
    "previous": (Q_PREVIOUS, K_PREVIOUS, "layer 4, head 11"),
}
N = len(WORDS)


def dot(a, b):
    return sum(a[d] * b[d] for d in range(D_K))


def raw_scores(head, query):
    """Q·K for one query against every key it is allowed to see.

    GPT-2 is causal: a token may attend to itself and to everything before it,
    never to what comes after. That is the "masked" attention of the paper's
    decoder, and it is why the upper triangle stays dark.
    """
    Q, K, _ = HEADS[head]
    return [dot(Q[query], K[j]) for j in range(query + 1)]


def scale(scores):
    """The ÷√dₖ the paper's formula is named for.

    Not decoration. Without it these scores are large enough that softmax
    saturates to one-hot and almost no gradient flows back.
    """
    return [s / math.sqrt(D_K) for s in scores]


def softmax(scores):
    top = max(scores)
    weights = [math.exp(s - top) for s in scores]
    total = sum(weights)
    return [w / total for w in weights]


def row(head, query, scaled=True):
    """The finished attention weights for one query."""
    scores = raw_scores(head, query)
    return softmax(scale(scores) if scaled else scores)


def matrix(head):
    return [row(head, i) for i in range(N)]
