import base64
import json
import requests

from terra_sdk.core import Coins
from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.util.contract import read_file_as_b64, get_code_id
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())

from config import tequila_config as config

client = LCDClient(url="https://tequila-lcd.terra.dev", chain_id="tequila-0004", gas_prices=Coins(requests.get("https://tequila-fcd.terra.dev/v1/txs/gas_prices").json()))
mnemonic = "main jar girl opinion train type cycle blood marble kitchen april champion amount engine crumble tunnel model vicious system student hood fee curious traffic"
deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))
std_fee = StdFee(4000000, "2500000uusd")

balance = client.bank.balance(deployer.key.acc_address)
print(balance)


def send_msg(msg):
    tx = deployer.create_and_sign_tx(
        msgs=[msg], fee=std_fee
    )
    estimated = client.tx.estimate_fee(tx, fee_denoms=["uusd"])
    print(f'estimated fee: {estimated}')
    return client.tx.broadcast(tx)

def store_contract(contract_name: str) -> str:
    bytes = read_file_as_b64(f"artifacts/{contract_name}.wasm")
    msg = MsgStoreCode(deployer.key.acc_address, bytes)
    result = send_msg(msg)
    return get_code_id(result)

def get_contract_address(result):
    log = json.loads(result.raw_log)
    contract_address = ''
    for entry in log[0]['events'][0]['attributes']:
        if entry['key'] == 'contract_address':
            contract_address = entry['value']
    return contract_address

def instantiate_contract(code_id: str, init_msg) -> str:
    msg = MsgInstantiateContract(
        owner=deployer.key.acc_address,
        code_id=code_id,
        init_msg=init_msg
    )
    result = send_msg(msg)
    print('result')
    print(result)
    return get_contract_address(result)

def execute_contract(contract_addr: str, execute_msg, coins=None):
    msg = MsgExecuteContract(
        sender=deployer.key.acc_address, 
        contract=contract_addr, 
        execute_msg=execute_msg,
        coins=coins
    ) if coins else MsgExecuteContract(
        sender=deployer.key.acc_address, 
        contract=contract_addr, 
        execute_msg=execute_msg
    ) 
    return send_msg(msg)


print("store contract")
code_id = store_contract(contract_name="stablecoin_vault")
print(f"stored {code_id} {type(code_id)}")

print("instantiate contract")
contract_address = instantiate_contract(code_id=code_id, init_msg={
    "pool_address": "terra156v8s539wtz0sjpn8y8a8lfg8fhmwa7fy22aff",
    "asset_info": {
        "native_token": { "denom": "uusd" }
    },
    "aust_address": config.aust_address,
    "anchor_money_market_address": config.money_market_address,
    "seignorage_address": config.seignorage_address,
    "slippage": "0.01",
    "token_code_id": 6429
})
print(f'instantiated {contract_address}')
# contract_address = "terra17fvgcyj0n92px30xt0qdhmnhjmuj6wyuya9tzd"

result = client.wasm.contract_query(contract_address, {
    "asset": {}
})
lp_token_address = result["liquidity_token"]
print(result)

result = client.wasm.contract_query(contract_address, {
    "pool": {}
})
print(result)

# result = client.wasm.contract_query("terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal", {
#     "state": {}
# })
# print(result)

# result = client.wasm.contract_query("terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal", {
#     "epoch_state": {}
# })
# print(result)

result = client.wasm.contract_query(lp_token_address, {
    "balance": {
        "address": deployer.key.acc_address
    }
})
print(result)
lp_balance = result["balance"]

# result = client.treasury.tax_rate()
# print(f'tax = {result}')

amount = 5*1000*1000
result = execute_contract(contract_addr=contract_address, execute_msg={
    "provide_liquidity": {
        "asset": {
            "info": {
                "native_token": { "denom": "uusd" }
            },
            "amount": str(amount)
        }
    }
}, coins=str(amount) + "uusd")
print(result)

result = client.wasm.contract_query(contract_address, {
    "pool": {}
})
print(result)

result = client.wasm.contract_query(lp_token_address, {
    "balance": {
        "address": deployer.key.acc_address
    }
})
print(result)

# contract_address = "terra1fg79czuq76nt699g96q2z9767gufpz8xx4s8k4"

# res = requests.get("https://fcd.terra.dev/v1/txs/gas_prices")
# client.gas_prices = Coins(res.json())

result = execute_contract(contract_addr=contract_address, execute_msg={
    "anchor_deposit": {
        "amount": {
            "denom": "uusd",
            "amount": str(int(amount/2))
        }
    }
})
print(result)

result = execute_contract(contract_addr=contract_address, execute_msg={
    "provide_liquidity": {
        "asset": {
            "info": {
                "native_token": { "denom": "uusd" }
            },
            "amount": str(amount)
        }
    }
}, coins=str(amount) + "uusd")
print(result)

# msg = base64.b64encode(bytes(json.dumps({"withdraw_liquidity": {}}), 'ascii')).decode()
# result = execute_contract(contract_addr=lp_token_address, execute_msg={
#     "send": {
#         "contract": contract_address,
#         "amount": lp_balance,
#         "msg": msg
#     }
# })
# print(result)

result = client.wasm.contract_query(contract_address, {
    "pool": {}
})
print(result)

result = client.wasm.contract_query(lp_token_address, {
    "balance": {
        "address": deployer.key.acc_address
    }
})
print(result)

# # result = client.wasm.contract_query(lp_token_address, {
# #     "token_info": {}
# # })
# # print(result)

# result = client.wasm.contract_query(contract_address, {
#     "asset": {}
# })
# print(result)


# result = client.wasm.contract_query(contract_address, {
#     "pool": {}
# })
# print(result)


# # result = client.wasm.contract_query(lp_token_address, {
# #     "balance": {
# #         "address": deployer.key.acc_address
# #     }
# # })
# # print("bal")
# # print(result)

# result = client.wasm.contract_query("terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl", {
#     "balance": {
#         "address": contract_address
#     }
# })
# print(result)

# result = client.bank.balance(contract_address)
# print(result)
# t = Coins(result)
# print(t["uusd"].amount)
