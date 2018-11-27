#!/bin/bash
pipenv install

setup='
from pathlib import Path
import mt940
input_str = Path("../../tests/data/mt940/full/danskebank/MT940_FI_Example.sta").read_text()
'
stmt='mt940.parse(input_str)'
pipenv run python -m timeit -s "$setup" "$stmt"
