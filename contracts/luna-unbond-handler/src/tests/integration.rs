use crate::state::State;
use crate::tests::common::TEST_CREATOR;
use crate::tests::common_integration::{init_contracts, mock_app};
use cosmwasm_std::Addr;
use terra_multi_test::Executor;
use white_whale::luna_vault::luna_unbond_handler::msg::{ExecuteMsg, QueryMsg};

#[test]
fn contract_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let base_contracts = init_contracts(&mut app);

    // update state of unbond handler
    app.execute_contract(
        sender.clone(),
        base_contracts.unbond_handler.clone(),
        &ExecuteMsg::UpdateState {
            owner: Some("unbonder".to_string()),
            expiration_time: Some(100),
            memory_contract: None,
        },
        &[],
    )
    .unwrap();

    let state: State = app
        .wrap()
        .query_wasm_smart(&base_contracts.unbond_handler, &QueryMsg::State {})
        .unwrap();

    assert_eq!(state.owner.unwrap(), "unbonder".to_string());
}
