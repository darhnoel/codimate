"""Everything the film says, in English and in Khmer. No Codimate here.

The physics does not change between languages and neither does the drawing, so
the words are the only thing that forks. Keeping them in one file rather than
copying the example means a fix to the *picture* cannot land in one language
and not the other.

Khmer needs no special handling in the view: `cm.measure` goes through the
engine's real fonts including fallback, so a label plate sized from it fits
Khmer as readily as ASCII. What Khmer does need is room — its sentences run
longer, so the caption size is part of the vocabulary rather than the layout.
"""

ENGLISH = {
    "lang": "en",
    "out": "results/archimedes.mp4",
    "say_size": 21,
    "card": "Archimedes' Principle",

    # titles
    "hook": "A steel bolt sinks. A steel ship floats. Why?",
    "simpler": "Start simpler",
    "waterbox": "A box of water, in water",
    "neutral": "It neither rises nor sinks",
    "ice": "Now make it ice",
    "push": "Hold it under",
    "release": "Let go",
    "stop": "It stops here",
    "steel": "Now make it steel",
    "question": "Then how can a steel ship float?",
    "spread": "Same steel, spread out",
    "average": "Steel and air together",
    "chart": "Denser than water, or not",

    # captions
    "hook2": "Same metal. Opposite answers.",
    "one_box": "One box. Volume V.",
    "climbs": "The water climbs by exactly the volume that went in.",
    "weighs": "The water it displaces weighs what the box weighs.",
    "same_box": "Same box. Same volume V.",
    "less_mass": "Same volume, less mass.",
    "full_under": "Full volume displaced, but less weight to carry.",
    "shrink": "As it leaves the water, it displaces less — "
              "so the push up falls.",
    "why": "Not a random height: the depth where it displaces its own weight.",
    "not_enough": "Even fully under, it cannot displace enough.",
    "sinks_now": "So down it goes.",
    "bigger": "A bigger tank — the same water.",
    "spreading": "The same steel, spread out...",
    "hollowing": "...and hollowed out. Not one gram more or less.",
    "unchanged": "The steel never changed. The outside got {spread:.0f} "
                 "times bigger.",
    "density": "Average density {rho:,.0f} kg/m3 — lighter than water, "
               "so it floats.",
    "said": "A thing floats when it can displace its own weight "
            "before it is all the way under.",

    # labels on the picture
    "water": "WATER", "ice_word": "ICE", "steel_word": "STEEL",
    "displaced": "water displaced",
    "of_v": "= {mark:.3g} V",
    "under": "{share:.1%} under",
    "above": "{share:.1%} above",
    "net": "net push up",
    "hand": "hand",
    "submerged": "{share:.1%} SUBMERGED",
    "rho": "{rho:,.0f} kg/m3",

    # the closing chart
    "water_name": "water", "ice_name": "ice",
    "ship_name": "steel ship", "block_name": "steel block",
    "floats": "floats", "even": "neutral", "sinks": "sinks",
    "water_mark": "water  1,000 kg/m3",
}

# Khmer. Science vocabulary in Cambodia largely follows French, so Archimède
# becomes អាស៊ីម៉ែត and "density" stays ដង់ស៊ីតេ rather than being calqued.
# Numerals are left Western so that the bars, the brackets and the algebra all
# carry the same digits — a chart with ១,០០០ on the axis and 1000 in the
# formula would be two different claims to read.
KHMER = {
    "lang": "km",
    "out": "results/archimedes-km.mp4",
    "say_size": 20,
    "card": "គោលការណ៍អាស៊ីម៉ែត",

    "hook": "ប៊ូឡុងដែកលិច។ នាវាដែកអណ្ដែត។ ហេតុអ្វី?",
    "simpler": "ចាប់ផ្ដើមពីរឿងងាយ",
    "waterbox": "ប្រអប់ទឹក ក្នុងទឹក",
    "neutral": "វាមិនអណ្ដែត មិនលិច",
    "ice": "ឥឡូវប្ដូរជាទឹកកក",
    "push": "សង្កត់វាឲ្យលិចក្នុងទឹក",
    "release": "លែងដៃ",
    "stop": "វាឈប់ត្រឹមនេះ",
    "steel": "ឥឡូវប្ដូរជាដែក",
    "question": "ចុះហេតុអ្វីនាវាដែកអាចអណ្ដែតបាន?",
    "spread": "ដែកដដែល ពង្រីកសន្ធឹង",
    "average": "ដែកនិងខ្យល់រួមគ្នា",
    "chart": "ក្រាស់ជាងទឹក ឬអត់",

    "hook2": "លោហៈដដែល។ ចម្លើយផ្ទុយគ្នា។",
    "one_box": "ប្រអប់មួយ។ មាឌ V។",
    "climbs": "ទឹកឡើងខ្ពស់ស្មើនឹងមាឌដែលចូលទៅ។",
    "weighs": "ទឹកដែលវារុញចេញ មានទម្ងន់ស្មើនឹងទម្ងន់ប្រអប់។",
    "same_box": "ប្រអប់ដដែល។ មាឌ V ដដែល។",
    "less_mass": "មាឌដដែល តែម៉ាសតិចជាង។",
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
    "density": "ដង់ស៊ីតេមធ្យម {rho:,.0f} គ.ក./ម៉.គូប "
               "— ស្រាលជាងទឹក ដូច្នេះវាអណ្ដែត។",
    "said": "វត្ថុមួយអណ្ដែត នៅពេលវារុញទឹកចេញស្មើនឹងទម្ងន់ខ្លួន "
            "មុនពេលលិចអស់។",

    "water": "ទឹក", "ice_word": "ទឹកកក", "steel_word": "ដែក",
    "displaced": "ទឹកដែលរុញចេញ",
    "of_v": "= {mark:.3g} V",
    "under": "លិច {share:.1%}",
    "above": "លើ {share:.1%}",
    "net": "កម្លាំងរុញឡើងសុទ្ធ",
    "hand": "ដៃ",
    "submerged": "លិចក្នុងទឹក {share:.1%}",
    "rho": "{rho:,.0f} គ.ក./ម៉.គូប",

    "water_name": "ទឹក", "ice_name": "ទឹកកក",
    "ship_name": "នាវាដែក", "block_name": "ដុំដែក",
    "floats": "អណ្ដែត", "even": "ស្មើ", "sinks": "លិច",
    "water_mark": "ទឹក  1,000 គ.ក./ម៉.គូប",
}

VOCABULARIES = {"en": ENGLISH, "km": KHMER}


def pick(lang="en"):
    """The vocabulary for `lang`, checked against English for holes.

    A missing key would not fail until the beat that needed it, minutes into a
    render — so the languages are compared here instead, where it costs
    nothing to find out.
    """
    if lang not in VOCABULARIES:
        raise SystemExit(f"no words for {lang!r}; have {sorted(VOCABULARIES)}")
    return VOCABULARIES[lang]


def _every_language_says_everything():
    for name, words in VOCABULARIES.items():
        missing = set(ENGLISH) - set(words)
        assert not missing, (name, sorted(missing))
        extra = set(words) - set(ENGLISH)
        assert not extra, (name, sorted(extra))
        # A format string that lost its placeholder would silently print the
        # template, so the braces are checked to match rather than exist.
        for key, line in words.items():
            if isinstance(line, str):
                assert line.count("{") == ENGLISH[key].count("{"), (name, key)
    return True


assert _every_language_says_everything()
