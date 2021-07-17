from terra_sdk.client.lcd import LCDClient
from terra_sdk.core.coin import Coin


def get_tobin_tax(client: LCDClient, denom: str):
    whitelist = client.oracle.parameters()["whitelist"]
    for entry in whitelist:
        if entry["name"] == denom:
            return float(entry["tobin_tax"])
    return None


def get_market_swap_rate(client: LCDClient, offer: Coin, ask_denom: str) -> float:
    return client.market.swap_rate(offer_coin=offer, ask_denom=ask_denom).amount


def get_terraswap_simulation_msg(offer: Coin):
    return {
        "simulation": {
            "offer_asset": {
                "info": { "native_token": { "denom": offer.denom } },
                "amount": str(offer.amount)
            }
        }
    }

def get_terraswap_rate(client: LCDClient, offer: Coin, pool_address: str) -> float:
    result = client.wasm.contract_query(pool_address, get_terraswap_simulation_msg(offer=offer))
    return int(result['return_amount'])
