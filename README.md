# White Whale

A novel decentralised arbitrage platform built on the [Terra](https://terra.money) blockchain.

## Contracts

| Name                                                       | Description                                  |
| ---------------------------------------------------------- | -------------------------------------------- |
| [`stablecoin-vault`](contracts/stablecoin-vault)           | UST vault contract                           |
| [`stable-arb-terra`](contracts/stable-arb-terra)           | UST arbitrage contract (using Terraswap LPs) |
| [`stable-arb-astro`](contracts/stable-arb-astro)           | UST arbitrage contract (using Terraswap LPs) |
| [`profit-check`](contracts/profit-check)                   | Profit checker for the UST vault             |
| [`treasury`](contracts/treasury/treasury)                  | Treasury contract                            |
| [`vesting`](contracts/vesting)                             | $WHALE vesting contract                      |

## Treasury contracts

Relative path: ../contracts/treasury

| Name                                                       | Description                                      |
| ---------------------------------------------------------- | ------------------------------------------------ |
| [`treasury`](contracts/treasury/treasury)                  | Treasury contract, acts as proxy                 |
| [`memory`](contracts/treasury/memory)     | address store for address translation             |

## DApp contracts

Relative path: ../contracts/treasury/dapps

| Name                                                       | Description                                      |
| ---------------------------------------------------------- | ------------------------------------------------ |
| [`dapp-template`](contracts/treasury/dapps/dapp-template)     | Template dapp that tests all the base_dapp functionality           |
| [`terraswap-dapp`](contracts/treasury/dapps/terraswap)     | Terraswap message generator contract             |
| [`astroport-dapp`](contracts/treasury/dapps/astroport)     | Astroport message generator contract             |
| [`vault-dapp`](contracts/treasury/dapps/vault)     | Vault contract, allows depositing and withdrawing on treasury contract             |


## Tokenomics contracts

Relative path: ../contracts/tokenomics

| Name                                                           | Description                                      |
| -------------------------------------------------------------- | ------------------------------------------------ |
| [`airdrop`](contracts/tokenomics/airdrop)                      | Airdrop contract                                 |
| [`lp_emissions_proxy`](contracts/tokenomics/lp_emissions_proxy)| Rewards generator proxy for liquidity providers  |
| [`lp_emissions`](contracts/tokenomics/lp_emissions)            | $WHALE LP emissions contract                     |

## Running this contract

You will need Rust 1.44.1+ with wasm32-unknown-unknown target installed.

You can run unit tests on this on each contracts directory via :

```
cargo test
```

Or for a production-ready (compressed) build, run the following from the repository root:

```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.3
```

The optimized contracts are generated in the artifacts/ directory.
