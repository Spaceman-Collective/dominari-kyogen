#!/usr/bin/env bash

solana program deploy --program-id ./localkeys/core-ds-keypair.json ./target/deploy/core_ds.so
solana program deploy --program-id ./localkeys/registry-keypair.json ./target/deploy/registry.so
solana program deploy --program-id ./localkeys/kyogen-keypair.json ./target/deploy/kyogen.so
solana program deploy --program-id ./localkeys/structures-keypair.json ./target/deploy/structures.so