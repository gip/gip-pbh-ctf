# gip-pbh-ctf

On PBH-enabled Worldchain, there are four invariants that should always hold true when the World Chain Builder is producing blocks. It is important to note that if the builder block is not selected, these conditions are not enforced.

The goal of this work is to validate these invariants by attempting to break them. This README is work-in-progress and will be updated with new findings.

Please refer to this [repo](https://github.com/gip/pbh-ctf) for more information.

## Issue #1 PBH Ordering Rules

The invariant states
```
PBH Ordering Rules: All PBH transactions must be ordered before non-PBH transactions in a block, except for sequencer transactions (eg. setL1BlockValuesEcotone, Deposit transactions, etc).
```

To run the code two wallets are needed, although it is likely that this specific finding could also be found with a single account.

```bash
export PRIVATE_KEY_0=...
export PRIVATE_KEY_1=...
echo 0 > pbh.nonce # Or put the PBH nonce to start from
# Update the pbh_ctf.toml file
RUST_LOG=info cargo run
```

The code will create 50 transactions be increasing nonce n0, n1, ..., n49. The transactions will then by submitted starting with n1, n2,..., n49 and finally n0. When n0 is submitted, all the transactions can be executed. The code will create blocks similar to [testnet block 10263444](https://worldchain-sepolia.explorer.alchemy.com/block/10263444?tab=txs).
Obviously PBH transaction are not ordered before non-PBH transactions. It seems that ordering by nonce takes priority hence breaking the stated invariant. 

## Issue #2 PBH Gas per UserOp/Tx

No single PBH UserOp or PBH transaction can exceed `pbhGasLimit` (fixed at 15_000_000 here). 

Using the code but increasing the number of iterations in the inner contract to a higher value (4000), and setting the max gas value to 16_000_000 for the PBH contract, the following issues were encoutered:
* Delay in transaction execution (sometimes 20 mins) even though the nonce was current and the blocks generated were empty
* Blocks without stamping, see for instance block [10290165](https://worldchain-sepolia.explorer.alchemy.com/block/10290165)
* Inconsistent failures, see for instance tx [0xa12bb11d8a479bc1fe69ae087b2808e4e5bd28fbd577bb8802b970021e48e509](https://worldchain-sepolia.explorer.alchemy.com/tx/0xa12bb11d8a479bc1fe69ae087b2808e4e5bd28fbd577bb8802b970021e48e509). This transaction consumes more than 15_000_000, breaking the invariant that PBH transactions can go above the `pbhGasLimit` amount. Even the inner transaction goes above that amount, hitting 15_168_670, so well above the 15_000_000 limit.
