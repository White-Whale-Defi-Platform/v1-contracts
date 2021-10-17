from pytest_bdd import scenario, given, when, then


import requests
import pathlib

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.core import Coins
from terra_sdk.core.auth import StdFee
from white_whale.deploy import Deployer
from terra_sdk.core.wasm import MsgExecuteContract

import pathlib
import sys
import json
import base64
sys.path.append(pathlib.Path(__file__).parent.resolve())

CONTRACT_ADDRESS = "terra1672xmlle8a0adkcklkeqvkr7x57gywg2kpt02u"
UST_VAULT= "terra14uqjlrg5efah459xkstxavf3wr7ku8s0j5h328"
WHALE_TOKEN= "terra1al4gd6wudfalazsvrjzz4fs8srasqcn9vyvqp9"
WHALE_PAIR = "terra1tc4dertggfyz9qye4ymptneqxlye2dpgfxfrhf"
LP_TOKEN = "terra1pl98xje34559ama3f7xfm5szkz68qxewgvcgdv"

from white_whale.deploy import Deployer

client = LCDClient(url="https://bombay-lcd.terra.dev", chain_id="bombay-12", gas_prices=Coins(requests.get("https://bombay-fcd.terra.dev/v1/txs/gas_prices").json()))
mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
wallet = Wallet(lcd=client, key=MnemonicKey(mnemonic))
std_fee = StdFee(4000000, "1500000uusd")

@scenario('stablecoinvault.feature', 'Depositing and Withdrawing')
def test_stablecoin_in_out():
    pass 


@given("I'm a UST token holder", target_fixture="deployer")
def verify_has_ust():
    deployer = Deployer(client=client, wallet=wallet, fee=std_fee)

    return deployer


@when("I attempt to deposit some UST into the vault")
@when("I submit the deposit tx")
def submit_deposit(deployer):
    res = deposit_to_ust_vault()
    print(res)


@then("the deposit should be there")
def verify_deposit_exists(deployer, poll_creation_response):
    res = get_deposited_amount()
    print(res)


@then("I should be able to withdraw some of it")
def attempt_to_withdraw(deployer, poll_creation_response):
    # Attempt to withdraw half of it
    withdraw_some_from_vault(deployer, get_deposited_amount()/2)
    

# Helper functions for the steps above
# TODO: Decide if this should go in SDK or just moved to a lib folder

def deposit_to_ust_vault(deployer: Deployer, amount: int):
    result = deployer.execute_contract(UST_VAULT, execute_msg={
    "provide_liquidity": {
        "asset": {
            "info": {
                "native_token": { "denom": "uusd" }
            },
            "amount": str(amount)
        }
    }})
    print(result)
    return result

def get_deposited_amount(deployer: Deployer, address):
    result = client.wasm.contract_query(UST_VAULT, {
        "balance": {
            "address": deployer.wallet.key.acc_address
        }
    })
    return int(result["balance"])

def withdraw_some_from_vault(deployer: Deployer, amount):
    msg = base64.b64encode(bytes(json.dumps({"withdraw_liquidity": {}}), 'ascii')).decode()
    result = deployer.execute_contract(contract_addr=LP_TOKEN, execute_msg={
        "send": {
            "contract": UST_VAULT,
            "amount": str(amount),
            "msg": msg
        }
    })
    print(result)

def withdraw_all_from_vault(deployer: Deployer, amount):
    msg = base64.b64encode(bytes(json.dumps({"withdraw_liquidity": {}}), 'ascii')).decode()
    result = deployer.execute_contract(contract_addr=LP_TOKEN, execute_msg={
        "send": {
            "contract": UST_VAULT,
            "amount": str(get_deposited_amount()),
            "msg": msg
        }
    })
    print(result)