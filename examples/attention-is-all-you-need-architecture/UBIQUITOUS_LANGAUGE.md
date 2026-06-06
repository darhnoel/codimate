# Ubiquitous Language

This example explains the original Transformer architecture from
“Attention Is All You Need” as a progressive architecture story.

## Scope

- Explain the original encoder-decoder Transformer shape from the paper.
- Reveal one concept per scene.
- Keep the full architecture hidden until the final reveal.
- Do not include BERT, GPT, encoder-only models, or decoder-only models.

## Core Terms

- **Translation problem**: A sequence-to-sequence task where an input sequence is
  transformed into an output sequence.
- **Encoder**: The left-side stack that reads the input sequence and produces
  memory.
- **Decoder**: The right-side stack that consumes shifted output tokens and
  encoder memory to produce output probabilities.
- **Input Embedding**: The bottom encoder layer that converts input tokens into
  vectors.
- **Output Embedding**: The bottom decoder layer that converts previous output
  tokens into vectors.
- **Positional Encoding**: Order information added to embeddings.
- **Encoder Block**: A repeated block containing Multi-Head Attention, Add &
  Norm, Feed Forward, and Add & Norm.
- **Decoder Block**: A repeated block containing Masked Multi-Head Attention,
  Add & Norm, Multi-Head Attention, Add & Norm, Feed Forward, and Add & Norm.
- **Self-Attention**: A token-to-token attention operation within the same
  sequence.
- **Masked Self-Attention**: Decoder self-attention that prevents each position
  from seeing future output tokens.
- **Cross-Attention**: Decoder attention that looks back at encoder memory.
- **Residual Path**: A bypass connection that carries old information around a
  sublayer before Add & Norm.
- **Add & Norm**: Residual addition followed by normalization.
- **Feed Forward**: Per-token refinement after attention.
- **Linear**: The projection from decoder output to token scores.
- **Softmax**: The conversion from token scores to output probabilities.

## Scene Rule

One scene introduces one concept, one main visual change, and one short
explanation.
