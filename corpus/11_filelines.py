for ln in open("log.txt"):
    if "ERR" in ln: print(ln, end="")
