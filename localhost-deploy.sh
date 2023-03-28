#!/usr/bin/env bash

solana program deploy --program-id ./localkeys/core-devnet.json ./target/deploy/core_ds.so
solana program deploy --program-id ./localkeys/registry-devnet.json ./target/deploy/registry.so
solana program deploy --program-id ./localkeys/kyogen-devnet.json ./target/deploy/kyogen.so
solana program deploy --program-id ./localkeys/structures-devnet.json ./target/deploy/structures.so