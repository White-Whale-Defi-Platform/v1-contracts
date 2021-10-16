import base64
import json

import pathlib
import sys
# temp workaround
sys.path.append('/workspaces/devcontainer/terra-sdk-python')
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.core.auth import StdFee
from white_whale.deploy import get_deployer
from white_whale.address.bombay.anchor import anchor_money_market, aust
from white_whale.address.bombay.terra import seignorage
from white_whale.address.bombay.terraswap import pools
from white_whale.address.bombay.white_whale import community_fund, war_chest


mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
# mnemonic = "flavor addict aunt cream vendor grocery prison chicken fee brother desert science shoulder isolate eyebrow zero rough together delay dose cliff world sugar viable"
std_fee = StdFee(6900000, "3500000uusd")

deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=std_fee)

def get_contract_address(result):
    log = json.loads(result.raw_log)
    contract_address = ''
    for entry in log[0]['events'][0]['attributes']:
        if entry['key'] == 'contract_address':
            contract_address = entry['value']
    return contract_address


def deploy():
    # store profit check contract
    print("store contract")
    code_id = deployer.store_contract(contract_name="profit_check")
    print(f"stored {code_id} {type(code_id)}")

    print("instantiate contract")
    profit_check_address = deployer.instantiate_contract(code_id=code_id, init_msg={
        "vault_address": deployer.wallet.key.acc_address,
        "denom": "uusd"
    })
    print(f'instantiated profit check {profit_check_address}')

    # store stablecoin vault contract
    print("store contract")
    code_id = deployer.store_contract(contract_name="stablecoin_vault")
    print(f"stored {code_id} {type(code_id)}")
    print("instantiate contract")
    contract_address = deployer.instantiate_contract(code_id=code_id, init_msg={
        "pool_address": pools['UST'].contract_address,
        "asset_info": {
            "native_token": { "denom": "uusd" }
        },
        "aust_address": aust,
        "anchor_money_market_address": anchor_money_market,
        "seignorage_address": seignorage,
        "profit_check_address": profit_check_address,
        "slippage": "0.01",
        "token_code_id": 148,
        "community_fund_addr": community_fund,
        "warchest_addr": war_chest,
        "warchest_fee": "0.001",
        "community_fund_fee": "0.005",
        "max_community_fund_fee": "1000000",
        "denom": "uusd",
        "anchor_min_withdraw_amount": "100000000"
    }, get_contract_address=get_contract_address)
    print(f'instantiated {contract_address}')

    # configure stablecoin vault as vault in profit_check
    result = deployer.execute_contract(contract_addr=profit_check_address, execute_msg={
        "set_vault": {
            "vault_address": contract_address
        }
    })
    print(result)

    return contract_address, profit_check_address


# deploy()
# exit()
# profit_check_address = "terra150y2gjkvk7cr26dxjjxrayc4xh4qgvle96s0ts"
contract_address = "terra1my6hhy0msz4vaq93smcr7ts59manxx5hlurn7h"

# result = client.wasm.contract_query(contract_address, {
#     "config": {}
# })
# print(result)
# result = deployer.execute_contract(contract_address, {
#     "set_admin": {
#         "admin": "terra1f6nthhyvtjalucnzdwwajp7mnhm5tpn5l46sed"
#     }
# })
# print(result)

# print(client.chain_id)
# profit_check_address = "terra1jc9sxkxcrmmgeak6wmn44403la3paz60v3n7fa"
# contract_address = "terra14uqjlrg5efah459xkstxavf3wr7ku8s0j5h328"
# deploy(config)
# if True:
#     exit()

# # result = client.wasm.contract_query(contract_address, {
# #     "pool": {}
# # })
# # print(result)
# contract_address = "terra1y2e2qdgkysnl3z020lkzdsxdkkc6wwqd4r5u6f"
# contract_address, profit_check_address = deploy(config, profit_check_address)


# result = client.wasm.contract_query(contract_address, {
#     "last_balance": {}
# })
# print(result)

# result = client.wasm.contract_query(contract_address, {
#     "vault": {}
# })
# print(result)


# result = client.wasm.contract_query(profit_check_address, {
#     "vault_address": {}
# })
# print(result)

# result = execute_contract(contract_addr=profit_check_address, execute_msg={
#     "set_vault": {
#         "vault_address": contract_address
#     }
# })
# print(result)

# result = client.wasm.contract_query(profit_check_address, {
#     "vault_address": {}
# })
# print(result)

# result = execute_contract(contract_addr=profit_check_address, execute_msg={
#     "set_vault": {
#         "vault_address": contract_address
#     }
# })
# print(result)

# result = client.wasm.contract_query(contract_address, {
#     "slippage": {}
# })
# print(result)

# result = execute_contract(contract_addr=contract_address, execute_msg={
#     "set_slippage": {
#         "slippage": "0.005"
#     }
# })
# print(result)

# result = client.wasm.contract_query(contract_address, {
#     "slippage": {}
# })
# print(result)

# result = client.wasm.contract_query(profit_check_address, {
#     "vault": {}
# })
# print(result)

# contract_address = "terra17fvgcyj0n92px30xt0qdhmnhjmuj6wyuya9tzd"

# result = client.wasm.contract_query("terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal", {
#     "state": {}
# })
# print(result)

# result = client.wasm.contract_query("terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal", {
#     "epoch_state": {}
# })
# print(result)

# result = client.treasury.tax_rate()
# print(f'tax = {result}')

result = deployer.client.wasm.contract_query(contract_address, {
    "config": {}
})
lp_token_address = result["liquidity_token"]

result = deployer.client.wasm.contract_query(lp_token_address, {
    "balance": {
        "address": deployer.wallet.key.acc_address
    }
})
lp_balance = int(result["balance"])
print(f'lp {lp_balance}')

# amount = 1100*1000*1000
amount = 1000*1000*1000
result = deployer.execute_contract(contract_addr=contract_address, execute_msg={
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
# result = deployer.execute_contract(contract_addr=lp_token_address, execute_msg={
#     "send": {
#         "contract": contract_address,
#         "amount": str(lp_balance),
#         "msg": msg
#     }
# })
# print(result)

result = deployer.client.wasm.contract_query(contract_address, {
    "pool": {}
})
print(result)

result = deployer.client.wasm.contract_query(lp_token_address, {
    "balance": {
        "address": deployer.wallet.key.acc_address
    }
})
print(result)
exit()
# contract_address = "terra1fg79czuq76nt699g96q2z9767gufpz8xx4s8k4"

# res = requests.get("https://fcd.terra.dev/v1/txs/gas_prices")
# client.gas_prices = Coins(res.json())

# result = execute_contract(contract_addr=contract_address, execute_msg={
#     "anchor_deposit": {
#         "amount": {
#             "denom": "uusd",
#             "amount": str(int(amount/2))
#         }
#     }
# })
# print(result)

# result = execute_contract(contract_addr=contract_address, execute_msg={
#     "provide_liquidity": {
#         "asset": {
#             "info": {
#                 "native_token": { "denom": "uusd" }
#             },
#             "amount": str(amount)
#         }
#     }
# }, coins=str(amount) + "uusd")
# print(result)



# result = client.wasm.contract_query(contract_address, {
#     "pool": {}
# })
# print(result)

# result = client.wasm.contract_query(lp_token_address, {
#     "balance": {
#         "address": deployer.wallet.key.acc_address
#     }
# })
# print(result)

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
# #         "address": deployer.wallet.key.acc_address
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
