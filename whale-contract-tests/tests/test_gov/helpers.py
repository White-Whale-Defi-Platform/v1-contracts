import requests
import json
import base64

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.core import Coins
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgExecuteContract
from white_whale.deploy import Deployer



def stake(deployer: Deployer, amount: int):
    staking_msg = {
        "stake_voting_tokens": {}
    }
    msg = base64.b64encode(bytes(json.dumps(staking_msg), 'ascii')).decode()
    result = deployer.execute_contract(WHALE_TOKEN, {
        "send": {
            "contract": CONTRACT_ADDRESS,
            "amount": str(amount),
            "msg": msg
        }})
    print(result)

def get_staked_amount(deployer: Deployer):
    result = client.wasm.contract_query(CONTRACT_ADDRESS, {
    "staker": { "address": deployer.wallet.key.acc_address }
    })
    return int(result["balance"])

def unstake_all(deployer: Deployer):
    result = client.wasm.contract_query(CONTRACT_ADDRESS, {
    "staker": { "address": deployer.wallet.key.acc_address }
    })
    amount = result["balance"]
    print(amount)
    result = deployer.execute_contract(CONTRACT_ADDRESS, {
        "withdraw_voting_tokens": {
            "amount": str(amount),
        }})
    print(result)

def vote(deployer: Deployer, poll_id: int):
    amount = get_staked_amount(deployer)
    result = deployer.execute_contract(CONTRACT_ADDRESS, {
    "cast_vote": {
        "poll_id": poll_id,
        "vote": "yes",
        "amount": str(amount),
        }
    })
    print("\n\n\n Vote response is")
    print(result)
    return result

def query_poll(client: LCDClient, poll_id: int):
    result = client.wasm.contract_query(CONTRACT_ADDRESS, {
    "poll": { "poll_id": poll_id }
    })
    print(result)
    return result

def execute_poll( poll_id: int):
    result = deployer.execute_contract(CONTRACT_ADDRESS, {
    "execute_poll": {
        "poll_id": poll_id,
        }
    })
    print(result)

def expire_poll(poll_id: int):
    result = deployer.execute_contract(CONTRACT_ADDRESS, {
    "expire_poll": {
        "poll_id": poll_id,
        }
    })
    print(result)

def end_poll( poll_id: int):
    result = deployer.execute_contract(CONTRACT_ADDRESS, {
    "end_poll": {
        "poll_id": poll_id,
        }
    })
    print(result)

def query_ust_vault(client):
    result = client.wasm.contract_query(UST_VAULT, {
    "config": {},
    })
    print(result)   

def create_poll(deployer: Deployer):
    set_slippage_msg =  {
    "set_slippage": {
        "slippage": "0.008"
        }
    }
    msg = base64.b64encode(bytes(json.dumps(set_slippage_msg), 'ascii')).decode()
    poll_execute_msg = {
        "order": 1,
        "contract": UST_VAULT,
        "msg": msg
    }
    create_poll_msg = {
        "create_poll": {
            "title": "Example Poll",
            "description": "Example Poll desc",
            "link": "https://whitewhale.finance",
            "execute_msgs": []
        }
    }
    msg = base64.b64encode(bytes(json.dumps(create_poll_msg), 'ascii')).decode()
    result = deployer.execute_contract(contract_addr=WHALE_TOKEN, execute_msg={
        "send": {
            "contract": CONTRACT_ADDRESS,
            "amount": "100000",
            "msg": msg
        }
    })
    print("\n\n\n Create Poll response is")
    print(result)
    return result
