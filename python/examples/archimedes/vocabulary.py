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
    "because": ("It neither rises nor sinks",
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
KM_SCENES = {
    "hook": ("ដែកមួយលិច ដែកមួយអណ្ដែត",
             "ប៊ូឡុងដែកតូចមួយលិច តើហេតុអ្វីបានជានាវាដែកដ៏ធំសម្បើម"
             "បែរជាអណ្ដែតទៅវិញ?"),
    "hook2": ("ដែកមួយលិច ដែកមួយអណ្ដែត",
              "លោហៈមានម៉ាសដូចគ្នា បាតុភូតមិនដូចគ្នា"),

    "simpler": ("ចាប់ផ្ដើមពីរឿងងាយ",
                "យើងចាប់ផ្ដើមគិតពីរឿងងាយៗសិន៖ ប្រអប់មួយ មាឌ V"),
    "waterbox": ("ប្រអប់ទឹក ក្នុងទឹក",
                 "ស្រមៃថាយើងមានទឹកមួយប្រអប់ ហើយទំលាក់វាចូលទៅក្នុងទឹក"),
    "displace": ("ប្រអប់ទឹក ក្នុងទឹក",
                 "មាឌទឹកកើនឡើងតាមមាឌប្រអប់ដែលចូលទៅ"),
    "neutral": ("មិនអណ្ដែត មិនលិច",
                "ដោយសារវាជារូបធាតុតែមួយ វាមិនអណ្ដែត ហើយក៏មិនលិចដែរ"),
    "because": ("មិនអណ្ដែត មិនលិច",
                "ទឹកដែលវារុញចេញ មានទម្ងន់ស្មើនឹងទម្ងន់ប្រអប់"),

    "ice": ("ប្ដូរជាទឹកកក",
            "ប្រអប់ដដែល មាឌ V ដដែល ប្ដូរតែរូបធាតុប៉ុណ្ណោះ"),
    "lighter": ("ប្ដូរជាទឹកកក",
                "មាឌដដែល តែមានម៉ាសតិចជាង"),

    "push": ("សង្កត់ឲ្យលិច",
             "ជាមួយនឹងទំហំដដែល ទាល់តែយើងប្រើកម្លាំងរុញសង្កត់បន្តិច "
             "ទើបវាអាចលិចស្មើផ្ទៃទឹក"),
    "more": ("សង្កត់ឲ្យលិច",
             "រុញទឹកចេញពេញមាឌ តែទម្ងន់ត្រូវទ្រតិចជាង"),

    "release": ("លែងដៃ",
                "យើងសម្រេចថាលែងដៃ ហើយកម្លាំងរុញឡើងធ្វើឲ្យវាងើបឡើង"),
    "shrink": ("លែងដៃ",
               "ពេលវាងើបផុតពីទឹក វារុញទឹកចេញតិចជាង "
               "ដូច្នេះកម្លាំងរុញឡើងថយចុះ"),

    "stop": ("វាឈប់ត្រឹមនេះ",
             "វានឹងឈប់ក្រោយពេលយើងលែងដៃបានមួយសន្ទុះ"),
    "why": ("វាឈប់ត្រឹមនេះ",
            "មិនមែនកម្ពស់ចៃដន្យទេ៖ ជាជម្រៅដែលវារុញទឹកចេញ"
            "ស្មើនឹងទម្ងន់ខ្លួនឯង"),
    "derive": ("ហេតុអ្វីត្រឹមជម្រៅនេះ",
               "ដាក់កម្លាំងរុញឡើងស្មើនឹងទម្ងន់ចុះ រួចសម្រួល"),
    "percent": ("ហេតុអ្វីត្រឹមជម្រៅនេះ",
                "ចម្លើយគឺជាផលធៀបនៃដង់ស៊ីតេ គ្មានអ្វីផ្សេងទៀតទេ"),

    "steel": ("ប្ដូរជាដែក",
              "ប្រអប់ដដែល មាឌ V ដដែល ប្ដូរតែរូបធាតុប៉ុណ្ណោះ"),
    "steelforce": ("ប្ដូរជាដែក",
                   "ទោះលិចទាំងស្រុង ក៏រុញទឹកចេញមិនគ្រប់គ្រាន់"),
    "sinks": ("ប្ដូរជាដែក",
              "ដូច្នេះវាលិចចុះ"),

    "question": ("ហេតុអ្វីនាវាដែកអណ្ដែត?",
                 "តើនាវាដែកអាចអណ្ដែតបានដោយសារអ្វី? លោហៈមិនប្រែ "
                 "ប្រែតែរូបរាង"),
    "bigger": ("អាងធំជាង",
               "ទឹកដដែល តែដាក់ក្នុងអាងដែលនាវាអាចផ្ទុកបាន"),
    "spreading": ("ដែកដដែល ពង្រីកមាឌ",
                  "យើងយកដែកដដែល ប៉ុន្តែពង្រីកមាឌរបស់វា"),
    "hollowing": ("ដែកដដែល ពង្រីកមាឌ",
                  "ហើយចោះឲ្យប្រហោង ដើម្បីឲ្យខាងក្នុងភាគច្រើនជាខ្យល់"),
    "lowering": ("ដែកដដែល ពង្រីកមាឌ",
                 "ឥឡូវដាក់លោហៈដដែលនោះត្រឡប់ចូលក្នុងទឹកវិញ"),
    "ships": ("ដែកដដែល ពង្រីកមាឌ",
              "ដែកមិនប្រែប្រួល។ ខាងក្រៅធំជាងមុន {spread:.0f} ដង"),
    "average": ("មានទាំងដែកនិងខ្យល់រួមគ្នា",
                "អ្វីដែលធ្វើឲ្យនាវាអណ្ដែត គឺដង់ស៊ីតេមធ្យម "
                "មិនមែនដង់ស៊ីតេលោហៈទេ"),

    "chart": ("ម៉ាសមាឌ ធៀបនឹងទឹក",
              "ឥទ្ធិពលនៃម៉ាសមាឌនៃអង្គធាតុដែលលិច ឬអណ្ដែតក្នុងទឹក"),
    "law": ("គោលការណ៍អាកស៊ីម៉ែត",
            "កម្លាំងរុញឡើង ស្មើនឹងទម្ងន់ទឹកដែលត្រូវបានរុញចេញ"),
    "said": ("គោលការណ៍អាកស៊ីម៉ែត",
             "វត្ថុមួយអណ្ដែត នៅពេលវារុញទឹកចេញស្មើនឹងទម្ងន់ខ្លួន "
             "មុនពេលលិចអស់"),
}

KM_LABELS = {
    "lang": "km",
    "out": "results/archimedes-km.mp4",
    "card": "គោលការណ៍អាកស៊ីម៉ែត",

    "water": "ទឹក", "ice_word": "ទឹកកក", "steel_word": "ដែក",
    "displaced": "ទឹកដែលត្រូវបានរុញចេញ",
    "of_v": "= {mark:.3g} V",
    "under": "លិច {share:.1%}",
    "above": "លើ {share:.1%}",
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


def _every_language_says_everything():
    for name, (labels, scenes) in VOCABULARIES.items():
        for mine, theirs, what in ((labels, EN_LABELS, "label"),
                                   (scenes, EN_SCENES, "scene")):
            assert set(mine) == set(theirs), (name, what,
                                              sorted(set(theirs) ^ set(mine)))
        # A format string that lost its placeholder would silently print the
        # template, so the braces are checked to match rather than exist.
        for key, line in labels.items():
            assert line.count("{") == EN_LABELS[key].count("{"), (name, key)
        for key, (title, sub) in scenes.items():
            want = EN_SCENES[key]
            assert title.count("{") == want[0].count("{"), (name, key)
            assert sub.count("{") == want[1].count("{"), (name, key)
            # The split only means something if it is kept. A title that
            # runs on is a subtitle that got lost, and this film has had
            # several — so a title may not be a sentence, whatever its
            # length. (A short subtitle is fine: "So down it goes.")
            assert len(title) <= 40, (name, key, len(title), title)
            assert not title.endswith((".", "។")), (name, key, title)
    return True


assert _every_language_says_everything()
