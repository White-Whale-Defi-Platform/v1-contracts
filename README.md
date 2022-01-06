# White Whale

A novel decentralised arbitrage platform built on the [Terra](https://terra.money) blockchain.

Documentation link: https://white-whale-defi-platform.github.io/docs/

Attention:
The contracts that we wish to be audited are flagged with **AUDIT**

## Contracts

| Name                                                       | Description                                  |
| ---------------------------------------------------------- | -------------------------------------------- |
| [`stablecoin-vault`](contracts/stablecoin-vault) **AUDIT**          | UST vault contract                           |
| [`stable-arb-terra`](contracts/stable-arb-terra) **AUDIT**          | UST arbitrage contract (using Terraswap LPs) |
| [`stable-arb-astro`](contracts/stable-arb-astro)           | UST arbitrage contract (using Terraswap LPs) |
| [`profit-check`](contracts/profit-check)         **AUDIT**          | Profit checker for the UST vault             |
| [`vesting`](contracts/vesting)                             | $WHALE vesting contract                      |

## Treasury contracts

Relative path: ../contracts/treasury

| Name                                                       | Description                                      |
| ---------------------------------------------------------- | ------------------------------------------------ |
| [`treasury`](contracts/treasury/treasury)       **AUDIT**           | Treasury contract, acts as proxy                 |
| [`memory`](contracts/treasury/memory)   **AUDIT**  | address store for address translation             |

## DApp contracts

Relative path: ../contracts/treasury/dapps

| Name                                                       | Description                                      |
| ---------------------------------------------------------- | ------------------------------------------------ |
| [`dapp-template`](contracts/treasury/dapps/dapp-template)  **AUDIT**   | Template dapp that tests all the base_dapp functionality           |
| [`terraswap-dapp`](contracts/treasury/dapps/terraswap)  **AUDIT**   | Terraswap message generator contract             |
| [`astroport-dapp`](contracts/treasury/dapps/astroport)     | Astroport message generator contract             |
| [`vault-dapp`](contracts/treasury/dapps/vault)   **AUDIT**  | Vault contract, allows depositing and withdrawing on treasury contract             |


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
