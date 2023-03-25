#!/usr/bin/env bash

# Check if sol-arc/ repo exists.
# If it doesn't then clone
if [ ! -d "./sol-arc" ] 
then
    git clone https://github.com/JumpCrypto/sol-arc.git
    cd sol-arc
    anchor build
    cd ../
fi

DEVNET_URL=$1
#REGISTRY="FqATVPaCvgi5jGbNeoG2sPd4bb9NoyirK2LjVSgErjJ7"
#KYOGEN="3no6yedMWm6Ru5fLCYnbhExEFMUetsAjWJrqYoykAV24"
#STRUCTURES="9MMTJ6xda4hBLDkmH328S6XuW39mKyjTEg2aqpnfc6xk"
#CORE="EVzNjnCa1w3Nd1m2TVNiKM7HLnbH8A1JdJzoF99EehrX"

echo "Make sure to anchor build core ds with the localhost address!"
anchor build
solana config set -u $DEVNET_URL

solana program deploy --program-id ./localkeys/core-ds-keypair.json sol-arc/target/deploy/core_ds.so
solana program deploy --program-id ./localkeys/registry-keypair.json ./target/deploy/registry.so
solana program deploy --program-id ./localkeys/kyogen-keypair.json ./target/deploy/kyogen.so
solana program deploy --program-id ./localkeys/structures-keypair.json ./target/deploy/structures.so