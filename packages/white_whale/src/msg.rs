use cosmwasm_std::{Coin, Decimal, Uint128};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::HandleMsg;


pub fn create_terraswap_msg(
    offer: Coin,
    belief_price: Decimal
) -> HandleMsg {
    let offer = Asset{
        info: AssetInfo::NativeToken{ denom: offer.denom.clone() },
        amount: offer.amount
    };
    HandleMsg::Swap{
        offer_asset: offer,
        belief_price: Some(belief_price),
        max_spread: Some(Decimal::from_ratio(Uint128(1), Uint128(100))),
        to: None,
    }
}
