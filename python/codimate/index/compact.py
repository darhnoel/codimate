"""The compact index file: written in chunks, read a chunk at a time (ADR 0020)."""

from __future__ import annotations

import json
import lzma
import struct
import sys
import threading
from array import array
from collections import OrderedDict
from pathlib import Path
from .base import Index


MAGIC = b"CDIDX\x00\x00\x02"   # the last byte is the format version


QUANT = 16                      # boxes are kept to 1/16 of a pixel


CHUNK = 32                      # beats per chunk


PRESET = 6                      # lzma: the sizes at 6 and 9 are the same, 6 is faster


CACHED = 6                      # chunks a reader keeps decoded


assert array("i").itemsize == 4 and array("q").itemsize == 8, "int32 and int64 columns"


def _quantised(box):
    return None if box is None else tuple(round(v * QUANT) for v in box)


def _encode(base: dict, beats: list) -> bytes:
    """One chunk, before compression: what is on screen when it starts, then what
    each of its beats changes. Boxes are written as differences from the last box
    that shape had."""
    names, kinds = {}, {}

    def name_of(name):
        return names.setdefault(name, len(names))

    def kind_of(kind):
        return kinds.setdefault(kind, len(kinds))

    last = {n: q for n, (_, _, q) in base.items() if q is not None}
    b_name, b_kind, b_layer, b_has = (array("i") for _ in range(4))
    b_box = array("q")        # a shape zoomed far past the screen has a huge box
    for name, (kind, layer, q) in base.items():
        b_name.append(name_of(name))
        b_kind.append(kind_of(kind))
        b_layer.append(layer)
        b_has.append(q is not None)
        b_box.extend(q if q is not None else (0, 0, 0, 0))

    counts = array("i")
    s_name, s_kind, s_layer, s_has, gone_ids = (array("i") for _ in range(5))
    s_delta = array("q")
    for sets, gone in beats:
        counts.extend((len(sets), len(gone)))
        for name, kind, layer, q in sets:
            s_name.append(name_of(name))
            s_kind.append(kind_of(kind))
            s_layer.append(layer)
            s_has.append(q is not None)
            if q is None:
                s_delta.extend((0, 0, 0, 0))
            else:
                s_delta.extend(a - p for a, p in zip(q, last.get(name, (0, 0, 0, 0))))
                last[name] = q
        gone_ids.extend(name_of(n) for n in gone)

    head = json.dumps({"names": list(names), "kinds": list(kinds), "n": len(b_name),
                       "beats": len(beats), "sets": len(s_name), "gone": len(gone_ids)},
                      ensure_ascii=False).encode()
    columns = (b_name, b_kind, b_layer, b_has, b_box, counts, s_name, s_kind,
               s_layer, s_has, s_delta, gone_ids)
    if sys.byteorder == "big":
        for column in columns:
            column.byteswap()
    return struct.pack("<I", len(head)) + head + b"".join(c.tobytes() for c in columns)


def write(path, canvas, beats, chunk: int = CHUNK) -> str:
    """Write `beats`, as `Explanation.index()` yields them, to `path`.

    One chunk is held at a time, so a long film costs no more memory than a short
    one. The file appears under its own name only once it is complete.
    """
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    partial = path.with_name(path.name + ".part")
    table, light = [], []
    now: dict = {}                         # name -> (kind, layer, quantised box)
    pending, base = [], {}

    with partial.open("wb") as out:
        out.write(MAGIC)

        def flush():
            nonlocal pending, base
            if pending:
                blob = lzma.compress(_encode(base, pending), preset=PRESET)
                table.append([out.tell(), len(blob)])
                out.write(blob)
            pending, base = [], dict(now)

        for beat in beats:
            light.append({k: v for k, v in beat.items() if k != "shapes"})
            sets = [(s["name"], s["kind"], s["layer"], _quantised(s["box"]))
                    for s in beat["shapes"]["set"]]
            gone = list(beat["shapes"]["gone"])
            pending.append((sets, gone))
            for name, kind, layer, q in sets:
                now[name] = (kind, layer, q)
            for name in gone:
                now.pop(name, None)
            if len(pending) == chunk:
                flush()
        flush()

        footer = lzma.compress(json.dumps(
            {"version": 1, "canvas": canvas, "chunk": chunk, "quant": QUANT,
             "chunks": table, "beats": light}, ensure_ascii=False).encode(),
            preset=PRESET)
        out.write(footer)
        out.write(struct.pack("<Q", len(footer)))
    partial.replace(path)
    return str(path)


class CompactIndex(Index):
    """A `.index.bin`, read lazily: the per-beat table at once, and a chunk of
    shapes only when a beat in it is asked for."""

    def __init__(self, path):
        self.path = Path(path)
        with self.path.open("rb") as f:
            if f.read(len(MAGIC)) != MAGIC:
                raise ValueError(f"{self.path.name} is not a codimate index of this "
                                 f"version: write it again with write_compact_index")
            f.seek(-8, 2)
            (length,) = struct.unpack("<Q", f.read(8))
            f.seek(-8 - length, 2)
            footer = json.loads(lzma.decompress(f.read(length)))
        self.canvas = footer["canvas"]
        self.beats = footer["beats"]
        self._chunk_size = footer["chunk"]
        self._quant = footer["quant"]
        self._table = footer["chunks"]
        self._cache: OrderedDict = OrderedDict()
        self._lock = threading.Lock()

    def _decoded(self, number: int):
        """(everything on screen when the chunk starts, what each beat changes)."""
        with self._lock:
            if number in self._cache:
                self._cache.move_to_end(number)
                return self._cache[number]
        offset, length = self._table[number]
        with self.path.open("rb") as f:
            f.seek(offset)
            raw = lzma.decompress(f.read(length))
        decoded = self._decode(raw)
        with self._lock:
            self._cache[number] = decoded
            while len(self._cache) > CACHED:
                self._cache.popitem(last=False)
        return decoded

    def _decode(self, raw: bytes):
        (hlen,) = struct.unpack_from("<I", raw, 0)
        head = json.loads(raw[4:4 + hlen])
        names, kinds = head["names"], head["kinds"]
        at = 4 + hlen

        def take(count, code="i"):
            nonlocal at
            column = array(code)
            width = column.itemsize
            column.frombytes(raw[at:at + width * count])
            at += width * count
            if sys.byteorder == "big":
                column.byteswap()
            return column

        n, nb, ns, ng = head["n"], head["beats"], head["sets"], head["gone"]
        b_name, b_kind, b_layer, b_has = (take(n) for _ in range(4))
        b_box = take(4 * n, "q")
        counts = take(2 * nb)
        s_name, s_kind, s_layer, s_has = (take(ns) for _ in range(4))
        s_delta = take(4 * ns, "q")
        gone_ids = take(ng)

        base, last = {}, {}
        for i in range(n):
            q = tuple(b_box[4 * i:4 * i + 4]) if b_has[i] else None
            base[names[b_name[i]]] = (kinds[b_kind[i]], b_layer[i], q)
            if q is not None:
                last[names[b_name[i]]] = q

        beats, s, g = [], 0, 0
        for b in range(nb):
            n_set, n_gone = counts[2 * b], counts[2 * b + 1]
            sets = []
            for i in range(s, s + n_set):
                name = names[s_name[i]]
                if s_has[i]:
                    before = last.get(name, (0, 0, 0, 0))
                    q = tuple(a + p for a, p in zip(s_delta[4 * i:4 * i + 4], before))
                    last[name] = q
                else:
                    q = None
                sets.append((name, kinds[s_kind[i]], s_layer[i], q))
            s += n_set
            beats.append((sets, [names[k] for k in gone_ids[g:g + n_gone]]))
            g += n_gone
        return base, beats

    def _shape(self, name, kind, layer, q):
        box = None if q is None else [v / self._quant for v in q]
        return {"name": name, "kind": kind, "layer": layer, "box": box}

    def shapes_at(self, beat: int) -> list:
        if not 0 <= beat < len(self.beats):
            raise IndexError(f"no beat {beat}: the index has {len(self.beats)}")
        number, within = divmod(beat, self._chunk_size)
        base, beats = self._decoded(number)
        now = dict(base)
        for sets, gone in beats[:within + 1]:
            for name, kind, layer, q in sets:
                now[name] = (kind, layer, q)
            for name in gone:
                now.pop(name, None)
        return [self._shape(name, *rest) for name, rest in now.items()]

    def changes(self, start: int = 0, stop: "int | None" = None):
        """Straight from the chunks, in the order they were written, so reading the
        whole index is linear in its size."""
        stop = len(self.beats) if stop is None else stop
        for number in range(start // self._chunk_size,
                            -(-stop // self._chunk_size)):
            _, beats = self._decoded(number)
            for within, (sets, gone) in enumerate(beats):
                k = number * self._chunk_size + within
                if start <= k < stop:
                    beat = dict(self.beats[k])
                    beat["shapes"] = {"set": [self._shape(*s) for s in sets],
                                      "gone": gone}
                    yield beat
