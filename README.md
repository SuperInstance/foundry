<div align="center">
  <img src=".github/assets/banner.png" alt="Foundry banner" />

&nbsp;

[![Github Actions][gha-badge]][gha-url] [![Telegram Chat][tg-badge]][tg-url] [![Telegram Support][tg-support-badge]][tg-support-url]

[gha-badge]: https://img.shields.io/github/actions/workflow/status/foundry-rs/foundry/test.yml?branch=master&style=flat-square
[gha-url]: https://github.com/foundry-rs/foundry/actions
[tg-badge]: https://img.shields.io/endpoint?color=neon&logo=telegram&label=chat&style=flat-square&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Ffoundry_rs
[tg-url]: https://t.me/foundry_rs
[tg-support-badge]: https://img.shields.io/endpoint?color=neon&logo=telegram&label=support&style=flat-square&url=https%3A%2F%2Ftg.sumanjay.workers.dev%2Ffoundry_support
[tg-support-url]: https://t.me/foundry_support

**[Install](https://getfoundry.sh/getting-started/installation)**
| [Docs][foundry-docs]
| [Benchmarks](https://www.getfoundry.sh/benchmarks)
| [Developer Guidelines](./docs/dev/README.md)
| [Contributing](./CONTRIBUTING.md)
| [Crate Docs](https://foundry-rs.github.io/foundry)

</div>

---

Blazing fast, portable and modular toolkit for Ethereum application development, written in Rust.

- [**Forge**](https://getfoundry.sh/forge) — Build, test, fuzz, debug and deploy Solidity contracts.
- [**Cast**](https://getfoundry.sh/cast) — Swiss Army knife for interacting with EVM smart contracts, sending transactions and getting chain data.
- [**Anvil**](https://getfoundry.sh/anvil) — Fast local Ethereum development node.
- [**Chisel**](https://getfoundry.sh/chisel) — Fast, utilitarian and verbose Solidity REPL.

![Demo](.github/assets/demo.gif)

## Installation

```sh
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

See the [installation guide](https://getfoundry.sh/getting-started/installation) for more details.

To verify a downloaded release archive or container image, see [Verifying Releases](./SECURITY.md#verifying-releases).

## Getting Started

Initialize a new project, build and test:

```sh
forge init counter && cd counter
forge build
forge test
```

Interact with a live network:

```sh
cast block-number --rpc-url https://eth.merkle.io
cast balance vitalik.eth --ether --rpc-url https://eth.merkle.io
```

Fork mainnet locally:

```sh
anvil --fork-url https://eth.merkle.io
```

Read the [Foundry Docs][foundry-docs] to learn more.

## Contributing

Contributions are welcome and highly appreciated. To get started, check out the [contributing guidelines](./CONTRIBUTING.md).

Join our [Telegram][tg-url] to chat about the development of Foundry.

## Support

Having trouble? Check the [Foundry Docs][foundry-docs], join the [support Telegram][tg-support-url], or [open an issue](https://github.com/foundry-rs/foundry/issues/new).

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in these crates by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
</sub>

## What SuperInstance adds

This fork ships one extra crate: **Gas Guardian** (`crates/gas-guardian`).
It does two things — detect gas regressions between commits, and flag deployments where one function eats your entire gas budget.

4 tests, 0 dependencies beyond `serde`.

### Catching a regression

Someone replaced a `mapping` lookup with an array scan in `transferFrom()`.
Gas went from 45K to 540K — a 12× regression. Gas Guardian compares snapshots and surfaces it:

```
commit a3f2  transferFrom  45,000 gas
commit b7c1  transferFrom  540,000 gas   ← 12× regression
```

The `find_regressions()` function takes two `GasSnapshot` structs and returns every function where gas increased:

```rust
use gas_guardian::{GasSnapshot, GasMeasurement, find_regressions};

let before = GasSnapshot {
    id: "a3f2".into(),
    measurements: vec![GasMeasurement { label: "transferFrom".into(), gas_used: 45_000 }],
    total_budget: None,
};
let after = GasSnapshot {
    id: "b7c1".into(),
    measurements: vec![GasMeasurement { label: "transferFrom".into(), gas_used: 540_000 }],
    total_budget: None,
};

let regressions = find_regressions(&before, &after);
assert_eq!(regressions.len(), 1);
assert!((regressions[0].factor - 12.0).abs() < 0.01);
// regressions[0].description = "Gas regression in transferFrom: 45000 → 540000 (12× increase)"
```

### Budget conservation

Give a snapshot a `total_budget`. When any single function exceeds 80% of that budget, `budget_overrun()` returns `true`:

```rust
let snapshot = GasSnapshot {
    id: "deploy-42".into(),
    measurements: vec![
        GasMeasurement { label: "deploy".into(), gas_used: 900_000 },
    ],
    total_budget: Some(1_000_000),
};

assert!(snapshot.budget_overrun());  // 90% of budget consumed by one function
```

### What's in the crate

| Type | What it does |
|------|-------------|
| `GasSnapshot` | A snapshot of gas usage per function, with optional total budget |
| `GasRegression` | A single regression: function name, before/after gas, multiplier |
| `ConservationReport` | Full analysis: total gas, hottest function, overrun flag, regression count |
| `find_regressions()` | Compare two snapshots → list of regressions |

All types implement `Serialize`/`Deserialize` so you can dump reports to JSON.

---

[foundry-docs]: https://getfoundry.sh
