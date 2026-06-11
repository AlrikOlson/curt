s = "aaabbcccca"
result = ""
i = 0
while i < len(s):
    char = s[i]
    count = 1
    while i + count < len(s) and s[i + count] == char:
        count += 1
    result += char + str(count)
    i += count
print(result)
