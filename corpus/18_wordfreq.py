import sys, collections
words = open(sys.argv[1]).read().lower().split()
for w, n in collections.Counter(words).most_common(10):
    print(w, n)
