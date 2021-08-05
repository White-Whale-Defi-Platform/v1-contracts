from typing import Union
from attr import dataclass
from terra_sdk.client.lcd import LCDClient
from terra_sdk.core.coin import Coin


@dataclass
class NativeToken:
    denom: str = ''


@dataclass
class Token:
    contract_addr: str = ''


def get_tobin_tax(client: LCDClient, denom: str):
    whitelist = client.oracle.parameters()["whitelist"]
    for entry in whitelist:
        if entry["name"] == denom:
            return float(entry["tobin_tax"])
    return None


def get_market_swap_rate(client: LCDClient, offer: Coin, ask_denom: str) -> float:
    return client.market.swap_rate(offer_coin=offer, ask_denom=ask_denom).amount


def get_terraswap_simulation_msg(offer: Union[Token,NativeToken], amount: int):
    try:
        return {
            "simulation": {
                "offer_asset": {
                    "info": { "token": { "contract_addr": offer.contract_addr } },
                    "amount": str(amount)
                }
            }
        }
    except:
        return {
            "simulation": {
                "offer_asset": {
                    "info": { "native_token": { "denom": offer.denom } },
                    "amount": str(amount)
                }
            }
        }

def get_terraswap_rate(client: LCDClient, offer: Union[Token,NativeToken], amount: int, pool_address: str) -> float:
    result = client.wasm.contract_query(pool_address, get_terraswap_simulation_msg(offer=offer, amount=amount))
    return int(result['return_amount'])
