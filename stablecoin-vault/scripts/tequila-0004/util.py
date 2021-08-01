import requests

from terra_sdk.core.coins import Coins


def get_gas_prices() -> Coins:
    return Coins(requests.get("https://tequila-fcd.terra.dev/v1/txs/gas_prices").json())


def get_gas_prices_col_4() -> Coins:
    return Coins(requests.get("https://fcd.terra.dev/v1/txs/gas_prices").json())