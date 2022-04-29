#!/usr/bin/env python3
# Parses Python Stable ABI symbol definitions from the manifest in the CPython repository located at https://github.com/python/cpython/blob/main/Misc/stable_abi.txt
# and produces a definition file following the format described at https://docs.microsoft.com/en-us/cpp/build/reference/module-definition-dot-def-files.
import sys

print("LIBRARY python3.dll")
print("EXPORTS")

count = 0

for line in sys.stdin:
    if line.startswith("function"):
        is_data = False
    elif line.startswith("data"):
        is_data = True
    else:
        continue

    count += 1
    name = line.split()[1]

    if is_data:
        print(f"{name} DATA")
    else:
        print(name)

assert count >= 859
