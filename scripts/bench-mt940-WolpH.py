#!/usr/bin/env python3

import timeit
setup = """
from pathlib import Path
import mt940
input_str = Path("tests/data/mt940/full/danskebank/MT940_DK_Example.sta").read_text()
"""
print(timeit.timeit("mt940.parse(input_str)", setup, number=10))
