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
             "An iron bolt sinks. An iron ship floats. Why?"),
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

    "steel": ("Now make it iron",
              "Same box. Same volume V. Only the material has changed."),
    "steelforce": ("Now make it iron",
                   "Even fully under, it cannot displace enough."),
    "sinks": ("Now make it iron",
              "So down it goes."),

    "question": ("Then how can iron ever float?",
                 "The metal is the same metal. Only where it sits changes."),
    "hollowing": ("Make the box hollow",
                  "Same box, same volume V. The walls thin, and the inside "
                  "becomes air."),
    "rising": ("Now it can float",
               "Enough iron has gone that the water can carry what is left."),
    "floats": ("Iron, floating",
               "Same outside as every other box. Only {left:.0f}% of the "
               "iron is still there."),
    "average": ("Iron and air together",
                "What floats a thing is the average over its outside, not "
                "the density of the stuff."),

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

    "water": "WATER", "ice_word": "ICE", "iron_word": "IRON",
    "displaced": "water displaced",
    "of_v": "= {mark:.3g} V",
    "under": "{share:.1%} under",
    "above": "{share:.1%} above",
    "net": "net push up",
    "hand": "hand",
    "submerged": "{share:.1%} SUBMERGED",

    "water_name": "water", "ice_name": "ice",
    "hollow_name": "hollow iron", "block_name": "solid iron",
    "floats": "floats", "even": "neutral", "sinks": "sinks",
}

# -------------------------------------------------------------------- Khmer

# Science vocabulary in Cambodia largely follows French, so "density" stays
# ដង់ស៊ីតេ rather than being calqued. Numerals are left Western so that the
# bars, the brackets and the algebra all carry the same digits — a chart with
# ១,០០០ on the axis and 1000 in the formula would be two different claims.
# Written by the author, and left exactly as written. The groupings below are
# the author's original ones; which *slot* a line fills is decided by the scene
# table further down, because several of these "titles" are verbose enough to
# be subtitles by the rule at the top of this file.
KM = {
    # titles
    "hook": "ប៊ូឡុង​ដែក​តូច​មួយ​លិច តើ​ហេតុអ្វីបានជា​នាវា​ដែក​ដ៏​ធំ​សម្បើម"
            "បែរជា​អណ្ដែត​ទៅវិញ?",
    "simpler": "យើង​ចាប់ផ្ដើម​គិត​ពី​រឿង​ងាយៗ​សិន",
    "waterbox": "ស្រមៃ​ថា​យើង​មាន​ទឹកដែល​មាន​ទំហំ​មួយ​ប្រអប់"
                "ដែល​កម្រាស់​នៃ​សម្បក​ខាងក្រៅ​អាច​ចោល​បាន ទំលាក់​ចូល​ទៅ​ក្នុងទឹក",
    "neutral": "ដោយសារ​វា​ជា​រូបធាតុ​តែមួយ វា​មិន​អណ្ដែត ហើយក៏​មិន​លិច​ដែរ",
    "ice": "ឥឡូវ​យើង​ប្ដូរ​វា​ជា​ទឹកកក​វិញ",
    "push": "ជាមួយនឹង​ទំហំ​ដដែល ទាល់តែ​យើង​ប្រើ​កម្លាំង​រុញ​សង្កត់​បន្តិច"
            "ទើប​វា​អាច​លិច​ស្មើ​ផ្ទៃ​ទឹក",
    "release": "យើង​សម្រេច​ថា​លែងដៃ",
    "stop": "វា​នឹង​ឈប់​ក្រោយពេល​យើង​លែងដៃ​បាន​មួយសន្ទុះ",
    "steel": "ឥឡូវ​យើង​ប្ដូរ​វា​ជា​ដែក​វិញ",
    "question": "តើ​នាវា​ដែក​អាច​អណ្ដែត​បាន​ដោយសារអ្វី?",
    "spread": "យើង​យក​ដែក​ដដែល ប៉ុន្តែ​ពង្រីក​មាឌ​របស់​វា",
    "average": "មាន​ទាំង​ដែក​និង​ខ្យល់​រួមគ្នា",
    "chart": "តាម​តារាង​នេះ​យើង​អាច​សង្កេតឃើញ​ថា​ម៉ាសមាឌ​នៃ​អង្គធាតុ\n"
             "ជា​អ្នក​កំណត់​លក្ខណៈ​នៃ​ការ​អណ្ដែត ឬ​លិច​របស់​វា",


    # captions
    "hook2": "លោហៈ​មាន​ម៉ាស​ដូចគ្នា បាតុភូត​មិន​ដូចគ្នា",
    "one_box": "ប្រអប់​មួយ​មាន​មាឌ V",
    "climbs": "មាឌ​ទឹក​កើនឡើង​តាម​មាឌ​ប្រអប់​ដែល​លិច​ចូល​ទៅ",
    "weighs": "ទឹកដែល​វា​រុញ​ចេញ មានទម្ងន់​ស្មើនឹង​ទម្ងន់​ប្រអប់",
    "same_box": "ប្រអប់​ដដែល​ហើយ​មាន​មាឌ V ដដែល",
    "less_mass": "ប្រអប់​មាន​មាឌ​ដដែល តែ​មាន​ម៉ាស​តិច​ជាង​មុន",
    "full_under": "ប្រអប់​រុញ​ទឹក​ចេញ​មួយ​ខ្នាតមាឌ V តែ​ទម្ងន់​ដែល​ទឹក​អាច​ទ្រ​ប្រអប់​បាន​មាន​ទំហំ​តូចជាង",
    "shrink": "ពេល​ប្រអប់​អណ្ដែត​ឡើង ទឹកដែល​ត្រូវ​បាន​រុញ​ចេញ​ក៏​ត្រូវ​បាន​កាត់បន្ថយ"
              " ដូច្នេះ​កម្លាំង​រុញ​ឡើង​ក៏​ថយ​ចុះ",
    "why": "កម្ពស់​ដែល​យើង​អាច​មើលឃើញ​នេះ​មិនមែន​កើតឡើង​ដោយចៃដន្យ​នោះ​ទេ​៖ "
           "វា​ជា​ជម្រៅ​ដែល​ប្រអប់​រុញ​ទឹក​ចេញ​ដើម្បី​រក្សា​លំនឹង",
    "not_enough": "ចំពោះ​ដែក​តាន់​វិញ ទោះ​ត្រូវ​លិច​ទាំងស្រុង​ក្ដី ក៏​មិន​អាច​រុញ​ទឹក​ចេញ​គ្រប់គ្រាន់​ដើម្បី​រក្សា​លំនឹង​បាន​ឡើយ",
    "sinks_now": "ដូច្នេះ​វា​ក៏​លិច​ចុះក្រោម",
    "bigger": "this needs to be changed",
    "spreading": "ដែក​ដដែល ប៉ុន្តែ​យើង​ពង្រីក​មាឌ​វា",
    "hollowing": "...ហើយ​ចោះ​ឲ្យ​ប្រហោង ប៉ុន្តែ​រក្សា​ម៉ាស​ឲ្យ​នៅ​ដដែល",
    "unchanged": "ដែក​មិន​ប្រែប្រួល។ ខាងក្រៅ​ធំជាង​មុន {spread:.0f} ដង",
    # The unit is the one edit: it is typeset by `unit()` now, on its own
    # line under the ship, so the sentence no longer spells it out.
    "density": "ដង់ស៊ីតេ​មធ្យម​ស្រាល​ជាង​ទឹក ដូច្នេះ​វា​អណ្ដែត។",
    "said": "វត្ថុ​មួយ​អណ្ដែត​បាន នៅពេល​ទម្ងន់​ទឹកដែល​វា​រុញ​ចេញ ស្មើនឹង​ទម្ងន់​របស់​វា"
}

KM_LABELS_CARD = "គោលការណ៍​អា​ក​ស៊ី​ម៉ែ​ត"     # the author's, same as the card

# Mine, not the author's — short Khmer titles for the scenes where both of the
# author's lines are verbose, and subtitles for the three that had none. Every
# one of these is a placeholder to be replaced; nothing the author wrote is in
# here. `untranslated()` and the render both leave the author's lines alone.
DRAFT = {
    "t_waterbox": "ប្រអប់​ទឹក ក្នុងទឹក",
    "t_neutral": "មិន​អណ្ដែត មិន​លិច",
    "t_push": "សង្កត់​ឲ្យ​លិច",
    "t_chart": "ម៉ាសមាឌ ធៀប​នឹង​ទឹក",
    "s_question": "ឥលូវយើងយកលោហៈដដែល ហើយកែឆ្នៃរូបរាងរបស់វា",

    # The reshape section. The four lines that used to be here were mine from
    # the first pass, not the author's, and they described a tank that grew
    # and a slab that balanced on the surface — neither of which happens now.
    "t_hollow": "ដោយចោះប្រអប់ឲ្យមានប្រហោងខាងក្នុង",
    "s_hollowing": "ប្រអប់​ដដែល​មាន​មាឌ V ដដែល​តែ​មាន​ជញ្ជាំង​ស្ដើង "
                   "ហើយ​ខាងក្នុង​ក្លាយជា​ខ្យល់",
    "t_rising": "ឥឡូវ​វា​អាច​អណ្ដែត",
    "s_rising": "ដោយសារដែក​បាត់​ទៅមួយភាគធំ ទឹក​ក៏អាច​ទ្រ​អ្វី​ដែល​នៅសល់​បាន",
    "t_floats": "ដែកក៏អាច​អណ្ដែត",
    "s_floats": "ទំហំប៉ុនប្រអប់​ឯទៀតមែន ប៉ុន្តែម៉ាសរបស់​ដែក​នៅសល់​ត្រឹម {left:.0f}% ប៉ុណ្ណោះ",
}

# Which line each scene shows, in which slot. The title names what the scene
# is for; the subtitle says what is happening. Where the author wrote one
# verbose line and one short one, the short one is the title — that is the
# only judgement made here, and it moves nothing and edits nothing.
KM_SCENES = {
    "hook": (KM["hook2"], KM["hook"]),
    "hook2": (KM["hook2"], KM["hook"]),

    "simpler": (KM["simpler"], KM["one_box"]),
    "waterbox": (DRAFT["t_waterbox"], KM["waterbox"]),
    "displace": (DRAFT["t_waterbox"], KM["climbs"]),
    "neutral": (DRAFT["t_neutral"], KM["neutral"]),
    "why_neutral": (DRAFT["t_neutral"], KM["weighs"]),

    "ice": (KM["ice"], KM["same_box"]),
    "lighter": (KM["ice"], KM["less_mass"]),

    "push": (DRAFT["t_push"], KM["push"]),
    "more": (DRAFT["t_push"], KM["full_under"]),

    "release": (KM["release"], KM["shrink"]),
    "shrink": (KM["release"], KM["shrink"]),

    "stop": (KM["stop"], KM["why"]),
    "why": (KM["stop"], KM["why"]),
    "derive": (KM["stop"], KM["why"]),
    "percent": (KM["stop"], KM["why"]),

    "steel": (KM["steel"], KM["same_box"]),
    "steelforce": (KM["steel"], KM["not_enough"]),
    "sinks": (KM["steel"], KM["sinks_now"]),

    "question": (KM["question"], DRAFT["s_question"]),
    "hollowing": (DRAFT["t_hollow"], DRAFT["s_hollowing"]),
    "rising": (DRAFT["t_rising"], DRAFT["s_rising"]),
    "floats": (DRAFT["t_floats"], DRAFT["s_floats"]),
    "average": (KM["average"], KM["density"]),

    "chart": (DRAFT["t_chart"], KM["chart"]),
    "law": (KM_LABELS_CARD, KM["said"]),
    "said": (KM_LABELS_CARD, KM["said"]),
}

KM_LABELS = {
    "lang": "km",
    "out": "results/archimedes-km.mp4",
    "card": KM_LABELS_CARD,

    "water": "ទឹក", "ice_word": "ទឹកកក", "iron_word": "ដែក",
    "displaced": "ទឹកដែល​ត្រូវ​បាន​រុញ​ចេញ",
    "of_v": "= {mark:.3g} V",
    "under": "ផ្នែក​លិច​ក្នុងទឹក {share:.1%}",
    "above": "ផ្នែក​លេចឡើង {share:.1%}",
    "net": "កម្លាំង​រុញ​ឡើង​សរុប",
    "hand": "ដៃ",
    "submerged": "លិច​ក្នុងទឹក {share:.1%}",

    "water_name": "ទឹក", "ice_name": "ទឹកកក",
    "hollow_name": "ប្រអប់​ដែក​ប្រហោង", "block_name": "ដុំដែក​តាន់",
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


TITLE_LIMIT = 40        # characters; past this a title is a subtitle

ZWSP = "\u200b"          # where a Khmer word ends. Invisible, and not a letter


def plain(line):
    """`line` without the word marks, for counting or comparing."""
    return line.replace(ZWSP, "")


def spoken(line):
    """`line` as one run of ordinary text: no word marks, no stray spacing.

    A line written across two source lines carries a newline into the middle
    of it, and a hand-edited one picks up double spaces. Neither is visible on
    screen, because the caption is drawn a word at a time — but both reach
    `cm.measure` and the speech model, and the recording is cached by exactly
    this string. One helper, so the voice, the cache and the layout cannot
    disagree about what the line says.
    """
    return " ".join(plain(line).split())


def untranslated(lang):
    """Scene slots still carrying a draft, or nothing, for `lang`.

    Not an error — a translator fills these in their own time. It exists so
    that "which of these words are mine" is a question with an answer, rather
    than something to be worked out by reading two languages side by side.
    """
    _, scenes = VOCABULARIES[lang]
    drafts = set(DRAFT.values())
    return sorted(key for key, lines in scenes.items()
                  if any(not line.strip() or line in drafts
                         for line in lines))


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

            # The split only means something if it is kept, and it has come
            # apart twice: once when the titles grew into sentences, and again
            # when a translation's own grouping was mapped straight into the
            # slots. So the shape is checked rather than remembered.
            assert title.strip(), (name, key, "no title")
            assert sub.strip(), (name, key, "no subtitle")
            # Word marks are invisible, so they do not count as length.
            assert len(plain(title)) <= TITLE_LIMIT, (name, key, title)
    return True


assert _every_language_says_everything()
