# Conformance Examples

Each folder is one small explanation, written in every language Codimate has
a Skin for. Each must write film data identical, byte for byte, to the
`expected.film.json` beside it (ADR 0031): together they are the behavioural
specification of the Authoring Surface. A new feature starts here, as an
example that fails in every language.

    python tools/conformance.py            # check every example, every language
    python tools/conformance.py --bless    # write expected.film.json from Python
                                           # (only for a new example, after reading it)

An example script takes one argument, the path to write its film data to.
