import requests
import pathlib
import sys
# temp workaround
# Uninstall local terrasdk! 
sys.path.append('/workspaces/devcontainer/terra-sdk-python')
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.key.key import Key

from terra_sdk.core import Coins
from terra_sdk.core.auth import StdFee
from white_whale.deploy import Deployer
from terra_sdk.core.wasm import MsgExecuteContract

import pathlib
import sys
import json
import base64
sys.path.append(pathlib.Path(__file__).parent.resolve())

CONTRACT_ADDRESS = "terra1s4v50aqdtlmd4yuehjukak04rxyd5yf2fx5c3x"

def stake(deployer: Deployer, amount: int):
    whale_token_addr = "terra1gwmn9h9cyp76ea2ce0djrhn9dpwmjhy3hepq50"
    staking_msg = {
        "stake_voting_tokens": {}
    }
    msg = base64.b64encode(bytes(json.dumps(staking_msg), 'ascii')).decode()
    result = deployer.execute_contract(whale_token_addr, {
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
    result = deployer.execute_contract(CONTRACT_ADDRESS, {
    "cast_vote": {
        "poll_id": poll_id,
        "vote": "yes",
        "amount": "100000"
        }
    })
    print(result)

def query_poll(client: LCDClient, poll_id: int):
    result = client.wasm.contract_query(CONTRACT_ADDRESS, {
    "poll": { "poll_id": poll_id }
    })
    print(result)

def create_poll(deployer: Deployer, poll_id: int):
    pass

def expire_poll(client: LCDClient, poll_id: int):
    result = deployer.execute_contract(CONTRACT_ADDRESS, {
    "expire_poll": {
        "poll_id": poll_id,
        }
    })
    print(result)

def end_poll(client: LCDClient, poll_id: int):
    result = deployer.execute_contract(CONTRACT_ADDRESS, {
    "end_poll": {
        "poll_id": poll_id,
        }
    })
    print(result)

def create_poll(deployer: Deployer):

    whale_token_addr = "terra1gwmn9h9cyp76ea2ce0djrhn9dpwmjhy3hepq50"
    # UST vault contract address??? 
    # set_slippage_msg =  {
    # "set_slippage": {
    #     "slippage": "0.008"
    #     }
    # }
    # msg = base64.b64encode(bytes(json.dumps(set_slippage_msg), 'ascii')).decode()
    # poll_execute_msg = {
    #     "order": 1,
    #     "contract": stablecoin_vault_address,
    #     "msg": msg
    # }
    # create_poll_msg = {
    #     "create_poll": {
    #         "title": "Set Slippage to 0.008",
    #         "description": "set slippage to 0.008",
    #         "link": "https://whitewhale.finance",
    #         "execute_msgs": [poll_execute_msg]
    #     }
    # }
    # msg = base64.b64encode(bytes(json.dumps(create_poll_msg), 'ascii')).decode()
    msg = "eyJjcmVhdGVfcG9sbCI6IHsidGl0bGUiOiAiU2V0IFNsaXBwYWdlIHRvIDAuMDA4IiwgImRlc2NyaXB0aW9uIjogInNldCBzbGlwcGFnZSB0byAwLjAwOCIsICJsaW5rIjogImh0dHBzOi8vd2hpdGV3aGFsZS5maW5hbmNlIiwgImV4ZWN1dGVfbXNncyI6IFt7Im9yZGVyIjogMSwgImNvbnRyYWN0IjogInRlcnJhMTc1OWplcG5kZmp5Znp6Z2V4YTV2bWtoaGFxN3Z5bjh0cThnZjV6IiwgIm1zZyI6ICJleUp6WlhSZmMyeHBjSEJoWjJVaU9pQjdJbk5zYVhCd1lXZGxJam9nSWpBdU1EQTRJbjE5In1dfX0="
    result = deployer.execute_contract(contract_addr=whale_token_addr, execute_msg={
        "send": {
            "contract": contract_address,
            "amount": "100000",
            "msg": msg
        }
    })
    print(result)

client = LCDClient(url="https://bombay-lcd.terra.dev", chain_id="bombay-10", gas_prices=Coins(requests.get("https://bombay-fcd.terra.dev/v1/txs/gas_prices").json()))
mnemonic = ""
wallet = Wallet(lcd=client, key=MnemonicKey(mnemonic))
std_fee = StdFee(4000000, "1500000uusd")
deployer = Deployer(client=client, wallet=wallet, fee=std_fee)
whale_pair_addr = "terra1tc4dertggfyz9qye4ymptneqxlye2dpgfxfrhf"
whale_token_addr = "terra1gwmn9h9cyp76ea2ce0djrhn9dpwmjhy3hepq50"

#stake(deployer, amount=50000000)
#print(str(get_staked_amount(deployer)))
vote(deployer, 6)
# expire_poll(client, 3)
# end_poll(client, 3)
query_poll(client, 6)
#unstake_all(deployer)
# create_poll(deployer)



# result = client.wasm.contract_query(contract_address, {
# "config": {},
# })

# SEND GGY
# msg = MsgExecuteContract(
#             sender=wallet.key.acc_address,
#             contract=whale_token_addr,
#             execute_msg={
#         "transfer": {
#             "recipient": "terra1f6nthhyvtjalucnzdwwajp7mnhm5tpn5l46sed",
#             "amount": "100000000",
#         }},
#         )
# tx = wallet.create_and_sign_tx(
#             msgs=[msg], fee=std_fee
#         )
# result = client.tx.broadcast(tx)