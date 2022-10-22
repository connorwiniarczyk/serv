#!/usr/bin/python
import sys

for line in sys.stdin:
    if line == "0":
        exit(0)

    print(line)
