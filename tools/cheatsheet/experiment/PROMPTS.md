# Held-out task prompts (cheatsheet experiment)

Ten tasks, none of which appear in corpus/. Each prompt is given to the
model verbatim. The model must answer with a curt program; grading is
mechanical (`grade.py`): parse (1 pt), check (1 pt), run-output equality
vs the frozen `.expected` (1 pt).

---

## 01_sumsq_evens

Write a curt program. Given the list `[3, 4, 7, 10, 5, 8]`, print the sum
of the squares of the even numbers (one number on one line).

## 02_top_cities

Write a curt program. Given this data:
cities: oslo pop 709, bergen pop 291, trondheim pop 212, stavanger pop 144
(as a list of records with fields `name` and `pop`), print the names of
the 2 cities with the highest pop, highest first, joined by a comma
(single line, e.g. `a,b`).

## 03_classify

Write a curt program. Define a function `tag` that takes an `int | str`
union value: for an int n it returns the string `int <n*2>`, for a str s
it returns `str <s uppercased>`. Apply it to `7`, then `"ok"`, then `12`,
printing each result on its own line.

## 04_word_lookup

Write a curt program. Count word frequencies in the sentence
`"the cat and the dog and the bird"`. Print the count of `"the"`, then on
the next line the count of `"fish"` (0 if absent — do not crash).

## 05_digital_root

Write a curt program. Compute the digital root of 9875 (repeatedly sum
decimal digits until a single digit remains) using a loop, and print it.

## 06_fizz

Write a curt program. For each integer i from 1 to 15 inclusive, print on
its own line: `fizzbuzz` if i is divisible by 15, else `fizz` if divisible
by 3, else `buzz` if divisible by 5, else the number itself.

## 07_rect_area

Write a curt program. Declare a record type `Rect` with float fields `w`
and `h`. Given rectangles (2.0 × 3.0) and (1.5 × 4.0), print the total
area of all rectangles (single line).

## 08_csv_sum

Write a curt program. Given the string `"14,3,27,6"`, split it on commas,
convert the pieces to integers, and print their sum.

## 09_product

Write a curt program. Print the product of `[2, 3, 4, 5]` using a fold.

## 10_first_neg

Write a curt program. Define a function `firstneg` returning the index of
the first negative number in a list, or `-1` if none. Print
`firstneg [5, 2, -7, 9]` and then `firstneg [1, 2, 3]` (two lines).
