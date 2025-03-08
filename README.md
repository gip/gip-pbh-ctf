# gip-pbh-ctf

On PBH-enabled Worldchain, there are four invariants that should always hold true when the World Chain Builder is producing blocks. It is important to note that if the builder block is not selected, these conditions are not enforced.

The goal of this work is to validate these invariants by attempting to break them. This README is work-in-progress and will be updated with new findings.

Please refer to this [repo](https://github.com/gip/pbh-ctf) for more information.

## Finging #0: Transaction / Block Issue

Starting on or about March 5th, transaction execution started to behave differently. The [video](https://www.loom.com/share/79d1c6c4a0964b0990066fa1dcbe0296?sid=292761e9-2fbf-46be-9f5d-7aafd9feabb5) shows the following:
* At 2025-03-08T04:35:50, `RUST_LOG=info cargo run -- --n 1 --iterations 10` sends transaction [0x2a276ba1c7651d7106ba3c8c2f1b79a91792932a97abd1f4c5dd8beca40f5851](https://worldchain-sepolia.explorer.alchemy.com/tx/0x2a276ba1c7651d7106ba3c8c2f1b79a91792932a97abd1f4c5dd8beca40f5851). It is executed right away.
* At 2025-03-08T04:36:04, same commands submits a new transaction [0x2822e865ff9daadffab11c1fb64ea24fc10b3cb178c52e060c39d9cf9c73a116](https://worldchain-sepolia.explorer.alchemy.com/tx/0x2822e865ff9daadffab11c1fb64ea24fc10b3cb178c52e060c39d9cf9c73a116). It is *not* executed at once.
* After 1 minute, the command `RUST_LOG=info cargo run -- --n 1 --iterations 10` is used to re-send a transaction. The result is `already known`. Transaction has clearly not been executed.
* To unlock the transaction, the user retries a number of times. At around 2025-03-08T04:38:01, thanks to the "reminders", the transaction executes. 

No congestion on the chain so really not sure what that is happening. I've tried with different gas / priority values.

The findings 1 and 2 can be reproduced when the transaction executes as they should. Please contact me for any questions.

## Finding #1: PBH Ordering Rules

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
RUST_LOG=info cargo run -- --n 50 -r
```

The code will create 50 transactions be increasing nonce n0, n1, ..., n49. The transactions will then by submitted starting with n1, n2,..., n49 and finally n0. When n0 is submitted, all the transactions can be executed. The code will create blocks similar to [testnet block 10263444](https://worldchain-sepolia.explorer.alchemy.com/block/10263444?tab=txs).
Obviously PBH transaction are not ordered before non-PBH transactions. It seems that ordering by nonce takes priority hence breaking the stated invariant. 

## Finding #2: Block Without Stamping

No single PBH UserOp or PBH transaction can exceed `pbhGasLimit` (fixed at 15_000_000 here). 

Using the code but increasing the number of iterations in the inner contract to a higher value (4000), and setting the max gas value to 16_000_000 for the PBH contract, the following issues were encoutered:
* Delay in transaction execution (sometimes 20 mins) even though the nonce was current and the blocks generated were empty, see #2
* Blocks without stamping, see for instance block [10290165](https://worldchain-sepolia.explorer.alchemy.com/block/10290165)
* Inconsistent failures, see for instance tx [0xa12bb11d8a479bc1fe69ae087b2808e4e5bd28fbd577bb8802b970021e48e509](https://worldchain-sepolia.explorer.alchemy.com/tx/0xa12bb11d8a479bc1fe69ae087b2808e4e5bd28fbd577bb8802b970021e48e509) (Also, this transaction consumes more than 15_000_000, breaking the invariant that PBH transactions can go above the `pbhGasLimit` amount. Even the inner transaction goes above that amount, hitting 15_168_670, so well above the 15_000_000 limit - but by not much, do that might be ok).

