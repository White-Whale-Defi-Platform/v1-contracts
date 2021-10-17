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

CONTRACT_ADDRESS = "terra13zkn22u6fs550xdmx2s2mydesd0ym4jk3r4um3"
UST_VAULT= "terra1tyym9mkpjq55qr5rkza4mm4hjqcpdp39vewauk"
WHALE_TOKEN= "terra1k8pgyyxde6y893kjfqhtw7q0uttn68th60d6gh"
WHALE_PAIR = "terra1xq3v5rp0w84ugqesv9m5q3xdx04akacsdlk5z7"

client = LCDClient(url="https://bombay-lcd.terra.dev", chain_id="bombay-12", gas_prices=Coins(requests.get("https://bombay-fcd.terra.dev/v1/txs/gas_prices").json()))
mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
wallet = Wallet(lcd=client, key=MnemonicKey(mnemonic))
std_fee = StdFee(4000000, "1500000uusd")

@scenario('governance.feature', 'Creating a Poll and Voting')
def test_governance_voting():
    pass 


@given("I'm a Whale token holder", target_fixture="deployer")
def verify_has_whale_or_provide_some_whale():
    deployer = Deployer(client=client, wallet=wallet, fee=std_fee)

    return deployer


@given("I have staked whale", target_fixture="stake_response")
def verify_has_whale_or_try_to_allocate(deployer):
    staked_amount = get_staked_amount(deployer)
    print(staked_amount)
    if not staked_amount:
        stake(deployer, 50000000)


@when("I attempt to create a simple text proposal", target_fixture="proposal_request")
def create_poll():
    return {
        "create_poll": {
            "title": "An example Text proposal",
            "description": "The first automated text proposal",
            "link": "https://whitewhale.finance",
            "execute_msgs": []
        }
    }


@when("I submit the proposal", target_fixture="poll_creation_response")
def publish_poll(deployer):
    # res = create_poll(deployer)
    # assert res
    # print(res)
    return {}


@then("the Poll should be created")
def verify_poll_exists(deployer, poll_creation_response):
    """verify_poll_exists

    At time of writing structure looks like this 

    {'id': 1, 
    'creator': 'terra1gxsfv4ruvda37q3ta0kwx42w7qy5l9hf9l30sz', 
    'status': 'in_progress', 
    'end_height': 6092877, 
    'title': 'Set Slippage to 0.008', 
    'description': 'set slippage to 0.008', 
    'link': 'https://whitewhale.finance', 
    'deposit_amount': '100000', 
    'execute_data': [{'order': 1, 'contract': 'terra1tyym9mkpjq55qr5rkza4mm4hjqcpdp39vewauk', 
    'msg': 'eyJzZXRfc2xpcHBhZ2UiOiB7InNsaXBwYWdlIjogIjAuMDA4In19'}], 
    'yes_votes': '0', 
    'no_votes': '0', '
    staked_amount': None, 
    'total_balance_at_end_poll': None}

    """
    res = query_poll(client, 3)
    print(res)
    print(poll_creation_response)

    assert res['id'] == 3
    assert res['creator'] == "terra1gxsfv4ruvda37q3ta0kwx42w7qy5l9hf9l30sz"
    assert res['status'] == "in_progress"
    # Assuming a newly created poll their should be no votes
    assert res['yes_votes'] == '0'
    assert res['no_votes'] == '0'


@then("I should be able to vote on it")
def attempt_to_vote(deployer, poll_creation_response):
    # poll_res = vote(deployer, 4)
    # print(poll_res)
    pass
    
# Helper functions for the steps above
# TODO: Decide if this should go in SDK or just moved to a lib folder

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