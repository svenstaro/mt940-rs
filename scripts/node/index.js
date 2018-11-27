const Benchmark = require("benchmark");
const mt940 = require('mt940-js');
const fs = require('fs');

let suite = new Benchmark.Suite;

let input = fs.readFileSync('../../tests/data/mt940/full/danskebank/MT940_FI_Example.sta');

async function run_benchmark() {
    let result = suite
        .add("parse", () => mt940.read(input))
        .run();
    await result

    let runtime_ms = result[0].stats.mean * 1000;
    console.log(`Run took ${runtime_ms}ms`);
}

run_benchmark();
