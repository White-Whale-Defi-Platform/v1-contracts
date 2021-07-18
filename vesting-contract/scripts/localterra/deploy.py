from terra_sdk.client.localterra import LocalTerra
from terra_sdk.util.contract import read_file_as_b64, get_code_id, get_contract_address
from terra_sdk.core import Coin, Coins
from terra_sdk.core.auth import StdFee
from terra_sdk.core.bank import MsgSend
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())


client = LocalTerra()
deployer = client.wallets["test1"]
wallet2 = client.wallets["test2"]
std_fee = StdFee(4000000, "1000000uusd")

balance = client.bank.balance(deployer.key.acc_address)
print(balance)
print(client.bank.balance(wallet2.key.acc_address))

token_contract="terra1mwkg85mc3mv7lshgw4mn5dw3q6yku778glt46e"

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

def send_funds(from_: str, to_: str, coins: Coins):
    msg = MsgSend(
        from_address=from_,
        to_address=to_,
        amount=coins
    )
    return send_msg(msg)

def execute_contract(contract_addr: str, execute_msg):
    msg = MsgExecuteContract(
        sender=deployer.key.acc_address, 
        contract=contract_addr, 
        execute_msg=execute_msg
    )
    return send_msg(msg)

# print(client.bank.balance("terra1fsutmdfzvvw9khkl873yllacl5l8utf2428zxd"))
# # print(execute_contract(contract_addr=token_contract, execute_msg={
# #     "transfer": {
# #         "recipient": "terra1fsutmdfzvvw9khkl873yllacl5l8utf2428zxd",
# #         "amount": "100000"
# #     }
# # }))
# print(client.wasm.contract_query(token_contract, {
#     "balance": {
#         "address": "terra1fsutmdfzvvw9khkl873yllacl5l8utf2428zxd"
#     }
# }))
# print(execute_contract(contract_addr="terra1vl8ujeusn9clww0agcmq3pq2v0d4ta607xx4ux", execute_msg={
#     "claim": {
#         "amount": 100000
#     }
# }))
# print(client.wasm.contract_query(token_contract, {
#     "balance": {
#         "address": "terra1fsutmdfzvvw9khkl873yllacl5l8utf2428zxd"
#     }
# }))

# print(client.bank.balance("terra1fsutmdfzvvw9khkl873yllacl5l8utf2428zxd"))
# print(send_funds(from_=deployer.key.acc_address, to_="terra1fsutmdfzvvw9khkl873yllacl5l8utf2428zxd", coins="100000uusd"))
print("store contract")
code_id = store_contract(contract_name="vesting_contract")
print(f"stored {code_id} {type(code_id)}")
print("instantiate contract")
contract_address = instantiate_contract(code_id=code_id, init_msg={
    "recipient": deployer.key.acc_address,
    "token_contract": "terra1mwkg85mc3mv7lshgw4mn5dw3q6yku778glt46e",
    "vesting_schedule": 30000
})
print(f'instantiated {contract_address}')
# result = execute_contract(contract_addr="terra199d3u09j0n6ud2g0skevp93utgnp38kdxj778w", execute_msg={
#     "mint": {
#         "recipient": deployer.key.acc_address,
#         "amount": "50000000"
#     }
# })
# print(result)
# result = execute_contract(contract_addr="terra199d3u09j0n6ud2g0skevp93utgnp38kdxj778w", execute_msg={
#     "transfer": {
#         "recipient": wallet2.key.acc_address,
#         "amount": "50000000"
#     }
# })
# print(result)
# result = execute_contract(contract_addr=contract_address, execute_msg={

# })

# print("ExecuteMsg response")
# print(result)

