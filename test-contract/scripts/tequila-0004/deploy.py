import json
import base64

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.util.contract import read_file_as_b64, get_code_id, get_contract_address
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())

client = LCDClient(url="https://tequila-lcd.terra.dev", chain_id="tequila-0004")
mnemonic = "main jar girl opinion train type cycle blood marble kitchen april champion amount engine crumble tunnel model vicious system student hood fee curious traffic"
deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))
std_fee = StdFee(4000000, "1000000uusd")

balance = client.bank.balance(deployer.key.acc_address)
print(balance)


def send_msg(msg):
    tx = deployer.create_and_sign_tx(
        msgs=[msg], fee=std_fee
    )
    return client.tx.broadcast(tx)

def store_contract(contract_name: str) -> str:
    bytes = read_file_as_b64(f"artifacts/{contract_name}.wasm")
    msg = MsgStoreCode(deployer.key.acc_address, bytes)
    result = send_msg(msg)
    return get_code_id(result)

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
code_id = store_contract(contract_name="test_contract")
print(f"stored {code_id} {type(code_id)}")

print("instantiate contract")
contract_address = instantiate_contract(code_id=code_id, init_msg={
    "pool_address": deployer.key.acc_address,
    "asset_info": {
        "native_token": { "denom": "uusd" }
    },
    "token_code_id": 6429
})
print(f'instantiated {contract_address}')

# contract_address = "terra1qppq9uu6xqz2c4my456gpxd89cwqsgc33fjy0t"
# result = client.wasm.contract_query(contract_address, {
#     "asset": {}
# })
# lp_token_address = result["liquidity_token"]
# print(result)

# result = client.wasm.contract_query(contract_address, {
#     "pool": {}
# })
# print(result)

# result = execute_contract(contract_addr=contract_address, execute_msg={
#     "provide_liquidity": {
#         "asset": {
#             "info": {
#                 "native_token": { "denom": "uusd" }
#             },
#             "amount": "1000000"
#         }
#     }
# }, coins="1000000uusd")
# print(result)


# result = client.wasm.contract_query(lp_token_address, {
#     "token_info": {}
# })
# print(result)

# msg = base64.b64encode(bytes(json.dumps({"withdraw_liquidity": {}}), 'ascii')).decode()
# result = execute_contract(contract_addr=lp_token_address, execute_msg={
#     "send": {
#         "contract": contract_address,
#         "amount": "99998",
#         "msg": msg
#     }
# })
# print(result)

result = client.wasm.contract_query(contract_address, {
    "asset": {}
})
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
print("bal")
print(result)
