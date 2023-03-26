#!/usr/bin/env bash

solana program deploy --program-id ./devnet-keys/core-devnet.json ./target/deploy/core_ds.so
solana program deploy --program-id ./devnet-keys/registry-devnet.json ./target/deploy/registry.so
solana program deploy --program-id ./devnet-keys/kyogen-devnet.json ./target/deploy/kyogen.so
solana program deploy --program-id ./devnet-keys/structures-devnet.json ./target/deploy/structures.so