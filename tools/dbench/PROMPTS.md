# Ceremony-suite task prompts (language-neutral)

Ten held-out tasks in error handling, JSON/record processing, file I/O,
and multi-step validation. Each prompt is given verbatim to the model.
Programs run with these data files in the working directory (`fixtures/`):

- `app.cfg` — `{"port": 8080, "name": "svc", "debug": true}`
- `orders.json` — `[{"id": 1, "amt": 50.5, "status": "paid"}, {"id": 2, "amt": 30.0, "status": "open"}, {"id": 3, "amt": 20.25, "status": "paid"}]`
- `users.txt` — five lines: `u1 alice 34` / `u2 bob 19` / `u3 carol 42` / `badline` / `u4 dave 27`

Grading is mechanical (`grade_dbench.py`): execute with fixtures in the
working directory, compare stdout to the frozen `.expected` with numeric
tokens compared numerically.

## e1_parse_fallback
Given the list of strings ["12", "x", "7", "-", "30"], print the sum of
the values that parse as integers (treating unparseable ones as 0), then
on the next line the count of unparseable entries.

## e2_missing_file
Attempt to read the JSON config file "missing.cfg" (it does NOT exist).
Print its "name" value, or "default" if the file or key is unavailable —
the program must not crash. Then read the JSON config "app.cfg" (it
exists) and print its "port" value (0 if missing). Two lines.

## e3_validate
For ages [34, -2, 19, 150, 42, 27], an age is valid when 0 <= age <= 120.
Print three lines: valid count, invalid count, sum of valid ages.

## r1_orders
Read "orders.json" (a JSON array of objects with id, amt, status). Print
the total amt of "paid" orders, then on the next line the count of "open"
orders.

## r2_project
Read "orders.json". Print the ids of orders with amt > 25, highest amt
first, joined by commas (e.g. 5,2).

## r3_report
Read "app.cfg" (JSON, has "name") and "orders.json". Print one line:
`<name>: <paid order count> paid, total <total paid amt>` (e.g.
`svc: 2 paid, total 70.75`).

## f1_users
Read "users.txt" — each valid line is "id name age" (3 whitespace-
separated fields, integer age); skip malformed lines. Print the count of
valid users, then the integer average of their ages.

## f2_adults
Read "users.txt" (same format/validity rules). Print the names of users
with age > 25, in file order, joined by single spaces.

## m1_oldest
Read "users.txt" (same rules). Print "name age" of the oldest valid user
(unique).

## m2_cfg_chain
Read "app.cfg" (has port, name, debug; may lack host). Print
`<mode> <host>:<port>` where mode is "debug" if the debug flag is true
else "prod", and host defaults to "localhost" when missing.
