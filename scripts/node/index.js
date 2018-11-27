const Benchmark = require("benchmark");
const mt940 = require('mt940-js');
const fs = require('fs');

const input = fs.readFileSync('../../tests/data/mt940/full/danskebank/MT940_FI_Example.sta');

const suite = new Benchmark.Suite;
suite
    .add("parse", () => {
        mt940.read(input);
    })
    .on("complete", () => {
        const runtime_ms = suite[0].stats.mean * 1000;
        console.log(`Run took ${runtime_ms}ms`);
    })
    .run();
