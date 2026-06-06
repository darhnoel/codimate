# Transformer Ubiquitous Language

This glossary is local to `examples/transformer`. Use these words when naming
state, steps, view helpers, motion, and comments for the Transformer diagram.

## Story

The animation explains the Transformer as three phases:

```text
Encoder -> Decoder -> Cross
```

- **Encoder phase**: highlights the encoder stack from input embedding up
  through self-attention, add-and-norm, feed-forward, and add-and-norm.
- **Decoder phase**: highlights the decoder stack from shifted output embedding
  through masked self-attention, cross-attention, feed-forward, linear, and
  softmax.
- **Cross phase**: highlights encoder output flowing into decoder
  cross-attention.

## Components

- **Embedding block**: the bottom token representation block. Encoder uses
  `Input Embedding`; decoder uses `Output Embedding`.
- **Positional encoding**: the sine-wave side input added into the embedding.
- **Attention block**: an orange attention layer.
- **Self-attention**: attention inside one stack. Encoder uses normal
  multi-head attention; decoder uses masked multi-head attention.
- **Cross-attention**: decoder attention that receives encoder output.
- **Feed-forward block**: the blue dense transformation inside a stack.
- **Add & Norm block**: the yellow residual merge and normalization block after
  each sublayer.
- **Linear block**: projection before output probabilities.
- **Softmax block**: final probability conversion.
- **Stack container**: the dark rounded frame around repeated encoder or decoder
  layers.
- **Nx label**: the repeated-layer marker for a stack.

## Flow Lines

- **Main flow**: the vertical arrow path through a stack.
- **QKV fan-in**: three short arrows feeding an attention block.
- **Residual arrow**: the side path from a sublayer into the next Add & Norm.
  It must start at the source block edge, bend around the side, and point into
  the Add & Norm block.
- **Cross bridge**: the horizontal path from encoder output into decoder
  cross-attention.
- **Pulse**: a moving dot that shows active signal flow along a connection.
- **Highlight outline**: a temporary outline around the active block.

## Code Mapping

```text
TransformerState          diagram options, such as show_pulses
TransformerPhase          Encoder, Decoder, Cross
TransformerTrace          ordered phases
TransformerTiming         duration per phase
TransformerMotion         reusable motion/easing choices
build_base_scene          static diagram
build_encoder_scene       encoder phase overlay
build_decoder_scene       decoder phase overlay
build_cross_scene         cross-attention overlay
vertical_conn             main vertical stack flow
qkv_arrows                QKV fan-in arrows
residual_left/right       side residual arrows
cross_bridge              encoder-to-decoder bridge
vpulse                    vertical pulse on stack flow
```

## Naming Rules

- Use **phase** for the large animation chapters: `Encoder`, `Decoder`, `Cross`.
- Use **block** for rectangular Transformer components.
- Use **flow** or **connection** for ordinary arrows.
- Use **residual arrow** for the side Add & Norm connection; do not call it a
  skip line in this example.
- Use **cross bridge** only for encoder-to-decoder cross-attention.
- Use **pulse** only for moving signal dots, not for static arrows.
