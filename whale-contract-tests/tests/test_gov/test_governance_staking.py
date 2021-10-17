import requests
import json
import base64

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.core import Coins
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgExecuteContract

from pytest_bdd import scenario, given, when, then
from white_whale.deploy import Deployer
from tests.test_gov.helpers import get_staked_amount, stake, unstake_all

CONTRACT_ADDRESS = "terra13zkn22u6fs550xdmx2s2mydesd0ym4jk3r4um3"
UST_VAULT= "terra1tyym9mkpjq55qr5rkza4mm4hjqcpdp39vewauk"
WHALE_TOKEN= "terra1k8pgyyxde6y893kjfqhtw7q0uttn68th60d6gh"
WHALE_PAIR = "terra1xq3v5rp0w84ugqesv9m5q3xdx04akacsdlk5z7"

client = LCDClient(url="https://bombay-lcd.terra.dev", chain_id="bombay-12", gas_prices=Coins(requests.get("https://bombay-fcd.terra.dev/v1/txs/gas_prices").json()))
mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
wallet = Wallet(lcd=client, key=MnemonicKey(mnemonic))
std_fee = StdFee(4000000, "1500000uusd")

@scenario('governance.feature', 'Staking Whale Tokens')
def test_governance_voting():
    pass 


@given("I'm a Whale token holder", target_fixture="deployer")
def verify_has_whale_or_provide_some_whale():
    deployer = Deployer(client=client, wallet=wallet, fee=std_fee)

    return deployer


@given("I have some available whale", target_fixture="stake_response")
def verify_has_whale_or_try_to_allocate(deployer):
    staked_amount = get_staked_amount(deployer)
    print(staked_amount)
    if not staked_amount:
        stake(deployer, 50000000)


@when("I attempt to stake some whale tokens to be able to vote on polls")
@when("I submit the staking tx ")
def create_poll():
    return {
        "create_poll": {
            "title": "An example Text proposal",
            "description": "The first automated text proposal",
            "link": "https://whitewhale.finance",
            "execute_msgs": []
        }
    }


@then("I should be able to stake", target_fixture="staker_response")
def attempt_to_stake(deployer):
    return {}


@then("I should be able to unstake the same amount")
def attempt_to_unstake(deployer, poll_creation_response):
    pass

 