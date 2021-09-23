import requests

import pathlib
import sys
# temp workaround
sys.path.append('/workspaces/devcontainer/terra-sdk-python')
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey

from terra_sdk.core import Coins
from terra_sdk.core.auth import StdFee
from white_whale.deploy import Deployer
from white_whale.address.bombay_10.anchor import anchor_money_market, aust

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())

client = LCDClient(url="https://bombay-lcd.terra.dev", chain_id="bombay-10", gas_prices=Coins(requests.get("https://bombay-fcd.terra.dev/v1/txs/gas_prices").json()))
mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
wallet = Wallet(lcd=client, key=MnemonicKey(mnemonic))
std_fee = StdFee(4000000, "1500000uusd")

deployer = Deployer(client=client, wallet=wallet, fee=std_fee)

whale_token_addr = "terra1gxsfv4ruvda37q3ta0kwx42w7qy5l9hf9l30sz"
whale_pair_addr = "terra1tc4dertggfyz9qye4ymptneqxlye2dpgfxfrhf"

###
print("store contract")
code_id = deployer.store_contract(contract_name="buy_and_burn")
print(f"stored {code_id}")
print("instantiate contract")
contract_address = deployer.instantiate_contract(code_id=code_id, init_msg={
    "whale_token_addr": whale_token_addr,
    "whale_pair_addr": whale_pair_addr,
    "anchor_money_market_addr": anchor_money_market,
    "aust_addr": aust,
    "anchor_deposit_threshold": str(int(10)*int(10**6)),
    "anchor_withdraw_threshold": str(int(1)*int(10**4)),
    "anchor_deposit_ratio": "0.5"
})
print(f'instantiated {contract_address}')

# })
