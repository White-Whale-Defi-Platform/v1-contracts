import json
import base64

from terra_sdk.client.localterra import LocalTerra
from terra_sdk.util.contract import read_file_as_b64, get_code_id
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract


client = LocalTerra()
deployer = client.wallets["test1"]
std_fee = StdFee(4000000, "1000000uusd")

balance = client.bank.balance(deployer.key.acc_address)
print(balance)


def send_msg(msg):
    tx = deployer.create_and_sign_tx(
        msgs=[msg], fee=std_fee
    )
    return client.tx.broadcast(tx)


def get_contract_address(result):
    log = json.loads(result.raw_log)
    contract_address = ''
    for entry in log[0]['events'][0]['attributes']:
        if entry['key'] == 'contract_address':
            contract_address = entry['value']
    return contract_address


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
    "token_code_id": 12
})
print(f'instantiated {contract_address}')

result = client.wasm.contract_query(contract_address, {
    "asset": {}
})
print(result)
lp_token_address = result["liquidity_token"]


result = client.wasm.contract_query(contract_address, {
    "pool": {}
})
print(result)

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


result = client.wasm.contract_query(lp_token_address, {
    "token_info": {}
})
print(result)

# msg = base64.b64encode(bytes(json.dumps({"withdraw_liquidity": {}}), 'ascii')).decode()
# result = execute_contract(contract_addr=lp_token_address, execute_msg={
#     "send": {
#         "contract": contract_address,
#         "amount": "100000",
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
