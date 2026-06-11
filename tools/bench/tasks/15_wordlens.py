s = "pack my box with five dozen liquor jugs"
ws = s.split()
longest = max(ws, key=len)
print(longest)
print(sum(len(w) for w in ws) // len(ws))
