# Vanity Hash Generator (pyshop task3)

## About
CLI tool to generate SHA-256 hashes with n zeros in the end by bruteforcing.

## Building
```
$ cargo build 
```

## Building docs
```
$ cargo doc --open --lib --document-private-items
```

## Running
```
$ cd targer/release
$ ./hash -N <number of digest zeros> -F <number of hashes> -W <number of workers>
```

## Benchmark
```
$ cd target/release
$ hyperfine --warmup 1  --parameter-scan worker 1 15 -D 1 './hash -N 3 -F 10 -W {worker}' --runs 50 --show-output --export-markdown ../../results.md --shell none --sort mean-time
```
