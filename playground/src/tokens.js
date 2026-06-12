// o200k_base token counting, client-side. Verified token-for-token against
// the native `curt tokens` (tiktoken-rs o200k_base) across the whole corpus
// by tokcheck.mjs — 0 mismatches. This is the same ISA the language is
// optimized against; the meter is not an approximation.
import { Tiktoken } from "js-tiktoken/lite";
import o200k from "js-tiktoken/ranks/o200k_base";

const tk = new Tiktoken(o200k);

export function countTokens(text) {
  return tk.encode(text).length;
}
