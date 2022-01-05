# Tests covered

## Unit tests

- Contract instantiation -> src/tests/instantiate.rs
- Queries
  - BaseQueryMsg::Config -> src/tests/query.rs
- Messages
  - BaseExecuteMsg::UpdateConfig -> src/tests/msg.rs
    - unsuccessful -> unauthorized
    - successful -> with treasury_address
    - successful -> with trader
    - successful -> with memory
    - successful -> with trader, trader & memory
    - successful -> with no parameters
  - BaseExecuteMsg::SetAdmin -> src/tests/msg.rs
    - unsuccessful -> unauthorized
    - successful

---

**Coverage: 92%**
