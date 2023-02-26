#!/usr/bin/env bash 

# clockwork localnet \
#    --bpf-program GN5Ww5qa8ej4evFCJxMhV6AFEPKhD1Drdu8qYYptVgDJ deps/core_ds.so

solana-test-validator -r 
    --bpf-program GN5Ww5qa8ej4evFCJxMhV6AFEPKhD1Drdu8qYYptVgDJ deps/core_ds.so
