#!/usr/bin/env bash 

# clockwork localnet \
#    --bpf-program GN5Ww5qa8ej4evFCJxMhV6AFEPKhD1Drdu8qYYptVgDJ deps/core_ds.so
anchor build

solana-test-validator -r \
    --bpf-program GN5Ww5qa8ej4evFCJxMhV6AFEPKhD1Drdu8qYYptVgDJ deps/core_ds.so \
    --bpf-program 7Vpu3mY18uA2iWBhAyKc72F9xs1SaMByV5KaPpuLhFQz target/deploy/registry.so \
    --bpf-program CTQCiB97LrAjAtHy1eqGwqGiy2mjefBXR762nrDhWYTL target/deploy/kyogen.so \
    --bpf-program 4Bo4cgr4RhGpXJsQUV4KENCf3HJwPveFsPELJGGN9GkR target/deploy/structures.so
