import threading
def work(i): print(f"job {i}")
for i in range(4):
    threading.Thread(target=work, args=(i,)).start()
