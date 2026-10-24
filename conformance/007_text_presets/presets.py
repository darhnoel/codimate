"""Text presets: words that rise into place, letters (Khmer and emoji kept
whole), and a blurred title that resolves, each with its own rules."""

import sys

import codimate as cm

T = cm.text
title = T.words_up_blur("title", "Explain anything", style=T.TextPresetStyle(size=56),
                        at=(640, 160))
words = T.words_up("words", "One idea per moment", at=(640, 360))
letters = T.letters_down("letters", "ពន្យល់ 👩‍💻 ok", style=T.TextPresetStyle(
    size=40, fill="#ffd60a", distance=30), at=(640, 540))
STATES = {"title": ("hidden", "blurred", "shown"),
          "words": ("hidden", "shown", "shown"),
          "letters": ("hidden", "hidden", "shown")}


def walk(state, emit):
    for step in (1, 2):
        state["step"] = step
        emit("reveal")


def view(frame):
    scene = cm.Scene()
    step = frame.state["step"]
    for preset in (title, words, letters):
        preset.draw(scene, state=STATES[preset.name][step])
    return scene


motion = [*title.rules(), *words.rules(stagger=0.2), *letters.rules(ease="spring")]
cm.explain(trace=cm.trace(walk, {"step": 0}), view=view, motion=motion).export_web(
    sys.argv[1])
