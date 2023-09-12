# Vanity Hash Generator (pyshop task3)

CLI tool to generate SHA-256 hashes with n zeros in the end by bruteforcing.

## Building
```bash
cargo build 
```

## Building docs
```bash
cargo doc --open --lib --document-private-items
```

## Usage
```bash
cd targer/release
./hash -N <number of digest zeros> -F <number of hashes> -W <number of workers>
```
For more detailed help use `--help`

## Benchmark
```bash
cd target/release
hyperfine --warmup 1  --parameter-scan worker 1 15 -D 1 './hash -N 3 -F 10 -W {worker}' --runs 50 --show-output --export-markdown ../../results.md --shell none --sort mean-time
```
