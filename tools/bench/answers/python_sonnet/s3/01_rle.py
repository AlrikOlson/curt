s = "aaabbcccca"
result = ""
i = 0
while i < len(s):
    ch = s[i]
    count = 0
    while i < len(s) and s[i] == ch:
        count += 1
        i += 1
    result += ch + str(count)
print(result)
