"""Everything the film says, in English and in Khmer. No Codimate here.

The physics does not change between languages and neither does the drawing, so
the words are the only thing that forks. Keeping them in one file rather than
copying the example means a fix to the *picture* cannot land in one language
and not the other.

**Every scene is a title and a subtitle, and they have different jobs.** The
title names what the scene is *for*, in a few words, and holds still while the
scene plays. The subtitle describes what is happening in front of you, and
changes as it happens. A sentence in the title slot is a subtitle that got
lost; a two-word subtitle is usually a title that did.

Units are never words. `kg/m^3` is typeset as mathematics wherever it appears,
so the exponent is an exponent in both languages rather than an abbreviation
that has to be spelt differently in each.

Khmer needs no special handling in the view: `cm.measure` goes through the
engine's real fonts including fallback, so a label plate sized from it fits
Khmer as readily as ASCII. What Khmer *does* need is room — its sentences run
much longer — so titles and subtitles are shrunk to fit at draw time rather
than being trusted to a size chosen here.
"""

# ------------------------------------------------------------------ English

EN_SCENES = {
    # key             title (what the scene is for)   subtitle (what happens)
    "hook": ("Two answers from one metal",
             "A steel bolt sinks. A steel ship floats. Why?"),
    "hook2": ("Two answers from one metal",
              "Same metal. Opposite answers."),

    "simpler": ("Start simpler",
                "One box, of volume V, held above the water."),
    "waterbox": ("A box of water, in water",
                 "Lower it in, and watch the surface rather than the box."),
    "displace": ("A box of water, in water",
                 "The water climbs by exactly the volume that went in."),
    "neutral": ("It neither rises nor sinks",
                "Made of the same stuff as the water around it, it just "
                "stays."),
    "why_neutral": ("It neither rises nor sinks",
                    "The water it displaces weighs what the box weighs."),

    "ice": ("Now make it ice",
            "Same box. Same volume V. Only the material has changed."),
    "lighter": ("Now make it ice",
                "Same volume, less mass."),

    "push": ("Hold it under",
             "It takes a hand to keep it level with the surface."),
    "more": ("Hold it under",
             "Full volume displaced, but less weight to carry."),

    "release": ("Let go",
                "With nothing holding it, the push up wins and it rises."),
    "shrink": ("Let go",
               "As it leaves the water it displaces less, so the push up "
               "falls."),

    "stop": ("It stops here",
             "It settles, and then it stays exactly there."),
    "why": ("It stops here",
            "Not a random height: the depth where it displaces its own "
            "weight."),
    "derive": ("Why that depth and no other",
               "Set the push up equal to the weight down, and cancel."),
    "percent": ("Why that depth and no other",
                "The answer is a ratio of densities, and nothing else."),

    "steel": ("Now make it steel",
              "Same box. Same volume V. Only the material has changed."),
    "steelforce": ("Now make it steel",
                   "Even fully under, it cannot displace enough."),
    "sinks": ("Now make it steel",
              "So down it goes."),

    "question": ("Then how can a steel ship float?",
                 "Nothing about the metal changes. Only its shape does."),
    "bigger": ("A bigger tank",
               "The same water, in something a hull will fit in."),
    "spreading": ("Same steel, spread out",
                  "The block widens. Not one gram is added or taken away."),
    "hollowing": ("Same steel, spread out",
                  "And it is hollowed out, so most of the inside is air."),
    "lowering": ("Same steel, spread out",
                 "Now put the very same metal back in the water."),
    "ships": ("Same steel, spread out",
              "The steel never changed. The outside got {spread:.0f} times "
              "bigger."),
    "average": ("Steel and air together",
                "What floats a ship is its average density, not the metal's."),

    "chart": ("Denser than water, or not",
              "Four cases, on one scale, against the only number that "
              "decides."),
    "law": ("Archimedes' Principle",
            "The push up is the weight of the fluid pushed aside."),
    "said": ("Archimedes' Principle",
             "A thing floats when it can displace its own weight before it "
             "is all the way under."),
}

EN_LABELS = {
    "lang": "en",
    "out": "results/archimedes.mp4",
    "card": "Archimedes' Principle",

    "water": "WATER", "ice_word": "ICE", "steel_word": "STEEL",
    "displaced": "water displaced",
    "of_v": "= {mark:.3g} V",
    "under": "{share:.1%} under",
    "above": "{share:.1%} above",
    "net": "net push up",
    "hand": "hand",
    "submerged": "{share:.1%} SUBMERGED",

    "water_name": "water", "ice_name": "ice",
    "ship_name": "steel ship", "block_name": "steel block",
    "floats": "floats", "even": "neutral", "sinks": "sinks",
}

# -------------------------------------------------------------------- Khmer

# Science vocabulary in Cambodia largely follows French, so "density" stays
# ដង់ស៊ីតេ rather than being calqued. Numerals are left Western so that the
# bars, the brackets and the algebra all carry the same digits — a chart with
# ១,០០០ on the axis and 1000 in the formula would be two different claims.
# Written by the author, and left exactly as written. The role each line
# plays is the author's too: these were already split into titles and captions
# before the scene table existed, so the table below only points at them.
KM = {
    # titles
    "hook": "ប៊ូឡុងដែកតូចមួយលិច តើហេតុអ្វីបានជានាវាដែកដ៏ធំសម្បើម"
            "បែរជាអណ្ដែតទៅវិញ?",
    "simpler": "យើងចាប់ផ្ដើមគិតពីរឿងងាយៗសិន",
    "waterbox": "ស្រមៃថាយើងមានទឹកដែលមានទំហំទំហំមួយប្រអប់"
                "ដែលសម្បកខាងក្រៅអាចចោលបានទំលាក់ចូលទៅក្នុងទឹក",
    "neutral": "ដោយសារវាជារូបធាតុតែមួយ វាមិនអណ្ដែត ហើយក៏មិនលិចដែរ",
    "ice": "ឥឡូវប្ដូរវាជាទឹកកកវិញ",
    "push": "ជាមួយនឹងទំហំដដែលទាល់តែយើងប្រើកម្លាំងរុញសង្កត់បន្តិច"
            "ទើបវាអាចលិចស្មើផ្ទៃទឹក",
    "release": "យើងសម្រេចថាលែងដៃ",
    "stop": "វានឹងឈប់ក្រោយពេលយើងលែងដៃបានមួយសន្ទុះ",
    "steel": "ឥឡូវប្ដូរវាជាដែកវិញ",
    "question": "តើនាវាដែកអាចអណ្ដែតបានដោយសារអ្វី?",
    "spread": "យើងយកដែកដដែល ប៉ុន្តែពង្រីកមាឌរបស់វា",
    "average": "មានទាំងដែកនិងខ្យល់រួមគ្នា",
    "chart": "ឥទ្ធិពលនៃម៉ាសមាឌនៃអង្គធាតុដែលលិច ឬអណ្ដែតក្នុងទឹក",

    # captions
    "hook2": "លោហៈមានម៉ាសដូចគ្នា បាតុភូតមិនដូចគ្នា",
    "one_box": "ប្រអប់មួយ។ មាឌ V។",
    "climbs": "មាឌទឹកកើនឡើងតាមមាឌប្រអប់ដែលចូលទៅ",
    "weighs": "ទឹកដែលវារុញចេញ មានទម្ងន់ស្មើនឹងទម្ងន់ប្រអប់",
    "same_box": "ប្រអប់ដដែល។ មាឌ V ដដែល។",
    "less_mass": "មាឌដដែល តែមានម៉ាសតិចជាង។",
    "full_under": "រុញទឹកចេញពេញមាឌ តែទម្ងន់ត្រូវទ្រតិចជាង។",
    "shrink": "ពេលវាងើបផុតពីទឹក វារុញទឹកចេញតិចជាង "
              "— ដូច្នេះកម្លាំងរុញឡើងថយចុះ។",
    "why": "មិនមែនកម្ពស់ចៃដន្យទេ៖ "
           "ជាជម្រៅដែលវារុញទឹកចេញស្មើនឹងទម្ងន់ខ្លួនឯង។",
    "not_enough": "ទោះលិចទាំងស្រុង ក៏រុញទឹកចេញមិនគ្រប់គ្រាន់។",
    "sinks_now": "ដូច្នេះវាលិចចុះ។",
    "bigger": "អាងធំជាង — ទឹកដដែល។",
    "spreading": "ដែកដដែល តែពង្រីកសន្ធឹង...",
    "hollowing": "...ហើយចោះឲ្យប្រហោង។ មិនបន្ថែម មិនបន្ថយសូម្បីមួយក្រាម។",
    "unchanged": "ដែកមិនប្រែប្រួល។ ខាងក្រៅធំជាងមុន {spread:.0f} ដង។",
    # The unit is the one edit: it is typeset by `unit()` now, on its own
    # line under the ship, so the sentence no longer spells it out.
    "density": "ដង់ស៊ីតេមធ្យមស្រាលជាងទឹក ដូច្នេះវាអណ្ដែត។",
    "said": "វត្ថុមួយអណ្ដែត នៅពេលវារុញទឹកចេញស្មើនឹងទម្ងន់ខ្លួន "
            "មុនពេលលិចអស់។",
}

KM_LABELS_CARD = "គោលការណ៍អាកស៊ីម៉ែត"     # the author's, same as the card

# Which line each scene shows, in which slot. An empty subtitle is a scene the
# author has not written a caption for yet — the view simply draws no
# subtitle, the same as when these were `say=""`.
KM_SCENES = {
    "hook": (KM["hook"], KM["hook2"]),
    "hook2": (KM["hook"], KM["hook2"]),

    "simpler": (KM["simpler"], KM["one_box"]),
    "waterbox": (KM["waterbox"], KM["climbs"]),
    "displace": (KM["waterbox"], KM["climbs"]),
    "neutral": (KM["neutral"], KM["weighs"]),
    "why_neutral": (KM["neutral"], KM["weighs"]),

    "ice": (KM["ice"], KM["same_box"]),
    "lighter": (KM["ice"], KM["less_mass"]),

    "push": (KM["push"], KM["full_under"]),
    "more": (KM["push"], KM["full_under"]),

    "release": (KM["release"], KM["shrink"]),
    "shrink": (KM["release"], KM["shrink"]),

    "stop": (KM["stop"], KM["why"]),
    "why": (KM["stop"], KM["why"]),
    "derive": (KM["stop"], KM["why"]),
    "percent": (KM["stop"], KM["why"]),

    "steel": (KM["steel"], KM["same_box"]),
    "steelforce": (KM["steel"], KM["not_enough"]),
    "sinks": (KM["steel"], KM["sinks_now"]),

    "question": (KM["question"], ""),
    "bigger": (KM["question"], KM["bigger"]),
    "spreading": (KM["spread"], KM["spreading"]),
    "hollowing": (KM["spread"], KM["hollowing"]),
    "lowering": (KM["spread"], ""),
    "ships": (KM["spread"], KM["unchanged"]),
    "average": (KM["average"], KM["density"]),

    "chart": (KM["chart"], ""),
    "law": (KM_LABELS_CARD, KM["said"]),
    "said": (KM_LABELS_CARD, KM["said"]),
}

KM_LABELS = {
    "lang": "km",
    "out": "results/archimedes-km.mp4",
    "card": KM_LABELS_CARD,

    "water": "ទឹក", "ice_word": "ទឹកកក", "steel_word": "ដែក",
    "displaced": "ទឹកដែលត្រូវបានរុញចេញ",
    "of_v": "= {mark:.3g} V",
    "under": "ផ្នែកលិចក្នុងទឹក {share:.1%}",
    "above": "ផ្នែកលេចឡើង {share:.1%}",
    "net": "កម្លាំងរុញឡើងសរុប",
    "hand": "ដៃ",
    "submerged": "លិចក្នុងទឹក {share:.1%}",

    "water_name": "ទឹក", "ice_name": "ទឹកកក",
    "ship_name": "នាវាដែក", "block_name": "ដុំដែក",
    "floats": "អណ្ដែត", "even": "ស្មើ", "sinks": "លិច",
}

VOCABULARIES = {"en": (EN_LABELS, EN_SCENES), "km": (KM_LABELS, KM_SCENES)}


def unit(value):
    r"""A density, as mathematics: `821\,\mathrm{kg/m^3}`.

    Not as words. Spelt out, the unit has to be abbreviated differently in
    every language and the exponent stops being an exponent — គ.ក./ម៉.គូប says
    the same thing as kg/m3 only to someone who already knows which it is.
    Typeset, it is the same symbol everywhere, and the 3 is where it belongs.
    """
    return rf"{value:,.0f}".replace(",", "{,}") + r"\,\mathrm{kg/m^3}"


def pick(lang="en"):
    """The words for `lang`, checked against English for holes.

    A missing key would not fail until the beat that needed it, minutes into a
    render — so the languages are compared here instead, where it costs
    nothing to find out.
    """
    if lang not in VOCABULARIES:
        raise SystemExit(f"no words for {lang!r}; have {sorted(VOCABULARIES)}")
    labels, scenes = VOCABULARIES[lang]
    return labels, scenes


def untranslated(lang):
    """Scene slots with nothing in them yet, for `lang`.

    Not an error. A translator fills these in their own time, and a blank
    subtitle simply draws no subtitle — the film still plays.
    """
    _, scenes = VOCABULARIES[lang]
    return sorted(key for key, (title, sub) in scenes.items()
                  if not title.strip() or not sub.strip())


def _every_language_says_everything():
    for name, (labels, scenes) in VOCABULARIES.items():
        for mine, theirs, what in ((labels, EN_LABELS, "label"),
                                   (scenes, EN_SCENES, "scene")):
            assert set(mine) == set(theirs), (name, what,
                                              sorted(set(theirs) ^ set(mine)))
        # A format string that lost its placeholder would silently print the
        # template, so the braces are checked to match rather than exist.
        # A blank is exempt: it is a slot waiting for words, not a broken one.
        for key, line in labels.items():
            assert line.count("{") == EN_LABELS[key].count("{"), (name, key)
        for key, (title, sub) in scenes.items():
            want = EN_SCENES[key]
            for mine, theirs in ((title, want[0]), (sub, want[1])):
                assert not mine.strip() or \
                    mine.count("{") == theirs.count("{"), (name, key)
    return True


assert _every_language_says_everything()
