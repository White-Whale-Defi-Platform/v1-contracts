# Tests covered

## Unit tests

- Contract instantiation -> src/tests/instantiate.rs
- Messages
  - ExecuteMsg::SetAdmin -> src/tests/instantiate.rs
    - unsuccessful -> unauthorized
    - successful -> authorized
  - ExecuteMsg::UpdateContractAddresses -> src/tests/interact.rs
  - ExecuteMsg::UpdateAssetAddresses -> src/tests/interact.rs
    - unsuccessful -> unauthorized
    - successful -> authorized
- Queries -> Tested in other dapp integration tests
  - QueryMsg::QueryAssets
  - QueryMsg::QueryContracts

---

# Coverage

`commands.rs`: 68%
`contract.rs`: 91%
`queries.rs`: 100%
