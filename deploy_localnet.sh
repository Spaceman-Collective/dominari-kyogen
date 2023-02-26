#!/usr/bin/env bash 

anchor build
# Deploy Registry
solana program deploy --program-id localkeys/registry-keypair.json target/deploy/registry.so --url localhost
# Deploy Kyogen Action Bundle
solana program deploy --program-id localkeys/kyogen-keypair.json target/deploy/kyogen.so --url localhost
#  Deploy Structures Action Bundle
solana program deploy --program-id localkeys/structures-keypair.json target/deploy/structures.so --url localhost