# pbh-ctf

On PBH-enabled Worldchain, there are four invariants that should always hold true when the World Chain Builder is producing blocks. It is important to note that if the builder block is not selected, these conditions are not enforced.

The goal of this work is to validate these invariants by attempting to break them. This README is work-in-progress and will be updated with new findings.

Please refer to this [repo](https://github.com/gip/pbh-ctf) for more information.

## PBH Ordering Rules

The invariant states
```
PBH Ordering Rules: All PBH transactions must be ordered before non-PBH transactions in a block, except for sequencer transactions (eg. setL1BlockValuesEcotone, Deposit transactions, etc).
```

To run the code two wallets are needed, although it is likely that this specific finding could also be found with a single account.

```bash
export PRIVATE_KEY_0=...
export PRIVATE_KEY_1=...
echo 0 > pbh.nonce # Or put the PBH nonce to start from
RUST_LOG=info cargo run
```

The code will create 50 transactions be increasing nonce n0, n1, ..., n49. The transactions will then by submitted starting with n1, n2,..., n49 and finally n0. When n0 is submitted, all the transactions can be executed. The code will create blocks similar to [testnet block 10263444](https://worldchain-sepolia.explorer.alchemy.com/block/10263444?tab=txs).
Obviously PBH transaction are not ordered before non-PBH transactions. It seems that ordering by nonce takes priority hence breaking the invariant. 