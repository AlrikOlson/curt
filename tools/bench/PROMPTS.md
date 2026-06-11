# token-bench task prompts (language-neutral)

Fifteen held-out tasks — disjoint from corpus/ and from the cheatsheet
experiment's 10. Each prompt is given verbatim to the model with one
sentence substituted for `<LANG>` ("curt" / "Python" / "Go" / "Rust").
Grading is mechanical (`grade_bench.py`): the program is executed and its
stdout compared line-by-line against the frozen `.expected`, with numeric
tokens compared numerically (so `86` ≡ `86.0` across languages).

---

## 01_rle
Write a <LANG> program. Run-length encode the string "aaabbcccca": for
each maximal run, output the character followed by the run length, all
concatenated on one line (expected shape: letterNumber pairs, e.g. for
"aab" you would print "a2b1").

## 02_vowels
Write a <LANG> program. Count the vowels (a,e,i,o,u) in
"the quick brown fox jumps over the lazy dog" and print the count.

## 03_gcd
Write a <LANG> program. Compute gcd(252, 105) with Euclid's algorithm
(loop, not recursion) and print it.

## 04_binary
Write a <LANG> program. For each of the numbers 13 and 64, print its
binary representation (no leading zeros, no prefix) on its own line,
computed with a loop (no built-in base conversion).

## 05_dedup
Write a <LANG> program. Remove duplicates from the list [3, 1, 3, 2, 1, 4]
keeping first occurrences in order, and print the result as numbers
joined by single spaces on one line.

## 06_stats
Write a <LANG> program. For the list [4, 8, 15, 16, 23] print three
lines: the minimum, the maximum, and the arithmetic mean (the mean is not
an integer).

## 07_matrix_diag
Write a <LANG> program. Print the sum of the main diagonal of the 3x3
matrix [[5,1,2],[3,6,4],[7,8,9]].

## 08_invoice
Write a <LANG> program. Given invoice items: widget (qty 4, unit price
2.5), gizmo (qty 2, unit price 7.25), bolt (qty 10, unit price 0.1) —
print the grand total (sum of qty*price) on one line, then the name of
the item with the highest line total (qty*price) on the next line.

## 09_topwords
Write a <LANG> program. In the sentence
"the cat sat on the mat with the cat", find the 2 most frequent words and
print each as "word count" on its own line, most frequent first.

## 10_parse_pairs
Write a <LANG> program. Parse the string "a=1,b=22,c=333" (comma-separated
key=value pairs) and print the sum of the integer values.

## 11_collatz_max
Write a <LANG> program. For n from 1 to 10 inclusive, count the Collatz
steps to reach 1 (n -> n/2 if even else 3n+1). Print the n with the most
steps and its step count as "n steps" on one line (the maximum is unique).

## 12_celsius
Write a <LANG> program. Convert the Celsius temperatures
[12.5, 30.0, -5.0] to Fahrenheit (F = C*9/5 + 32) and print the maximum
Fahrenheit value.

## 13_describe
Write a <LANG> program. Define a function `describe` that takes a value
which may be an integer, a float, or a string, and returns: "int <v+1>"
for an integer, "float <v>" for a float, "str <length>" for a string.
Print describe(42), describe(3.5), describe("abc") on three lines.

## 14_rect_lib
Write a <LANG> program structured as a small reusable library plus usage:
define a rectangle type with float width and height, and three functions —
area, perimeter, and scale (returns a new rectangle scaled by a factor).
For a 3.0 x 4.0 rectangle print three lines: its area, its perimeter, and
the area of the rectangle scaled by 2.0.

## 15_wordlens
Write a <LANG> program. For the sentence
"pack my box with five dozen liquor jugs" print the longest word, then on
the next line the integer part of the average word length.
