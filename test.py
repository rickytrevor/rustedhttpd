import os

for i in range(0, 16):
    os.system("ab -n 100000 -c 100 http://127.0.0.1:8453/ &")