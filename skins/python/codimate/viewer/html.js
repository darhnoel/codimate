// Markup as tagged template strings — JSX without a compiler.
import { h } from "preact";
import htm from "htm";

export const html = htm.bind(h);
