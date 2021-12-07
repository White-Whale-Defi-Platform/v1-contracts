from terra_sdk.core.bank import MsgSend
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract
import base64
import json

import pathlib
import sys
import datetime
from typing import List
# temp workaround
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from white_whale.contracts.terraswap_dapp import *
from white_whale.contracts.lbp import *
from white_whale.contracts.treasury import *
from terra_sdk.core.coins import Coin
from white_whale.deploy import get_deployer

def wait():
    s = input("Press enter to continue")
    if s == 'x':
        exit()

# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
# mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"
mnemonic = "pill hole shiver wage infant danger salt dismiss steak weather shell bright grass company violin large pride vessel physical rain number rookie best three"

# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

#####################
#   DEPLOY PARAMETERS
#####################

MILLION = 1_000_000

YEAR = 2021
MONTH = 12
DAY = 12
# 5 AM UTC
HOUR = 5
START_TIME = datetime.datetime(year=YEAR, month=MONTH, day=DAY, hour=HOUR)
# Start time in linux language
BLOCK_START_TIME = (START_TIME - datetime.datetime(1970, 1, 1)).total_seconds()
# 3 day duration (=72h)
DURATION = datetime.timedelta(days=3)

END_TIME = START_TIME + DURATION

NOW = datetime.datetime.utcnow()
print(f'LBP starts in {START_TIME - NOW} from now.')

print(f'Starts on {START_TIME}')
print(f'Ends on {END_TIME}')

BLOCK_END_TIME = BLOCK_START_TIME + DURATION.total_seconds()

print(BLOCK_END_TIME - BLOCK_START_TIME)
print((END_TIME - START_TIME).total_seconds())

MULTISIG_ADDRESS = "terra1xhhedykp6knc0ygtggwek3w0et3dmwe6apzeuz"
print(f'Wallet address {deployer.wallet.key.acc_address}')

wait()

treasury = TreasuryContract(deployer)
terraswap_dapp = TerraswapDAppContract(deployer)
lbp = LiquidityBootstrappingPool(deployer)
# lbp.unregister()

# STEP 1
# Create Treasury
create = False
if create:

    # Uploads and instantiates the treasury
    #treasury.create()
    # Used vars: 
    #   - Whale token
    #   - admin = sender
    #   - 

    print(f'treasury address is: {treasury.get("treasury")}')
    wait()
    
    # STEP 2
    # Uploads and instantiates the LBP
    # lbp.create(True, BLOCK_START_TIME, BLOCK_END_TIME)
    #   - Check begin/end time
    #   - Check weights
    #   - Check whale token
    #   - Check owner (treasury)
    #   - 
    print(f'lbp address is: {lbp.get("lbp_pair")}')

    wait()

    # STEP 3
    # Uploads and instantiates the LBP
    # terraswap_dapp.create()
    #   - Check treasury addr
    #   - Check token/pair addressbook
    #   - Check trader (multisig)
    #   - 
    print(f'terraswap dapp address is: {terraswap_dapp.get("terraswap_dapp")}')
    treasury.add_dapp(terraswap_dapp.get("terraswap_dapp"))
    wait()

print("treasury config:")
treasury.query_config()
print(f'terraswap dapp address is: {terraswap_dapp.get("terraswap_dapp")}')
wait()

print("lbp pool:")
lbp.query_pool()
wait()

print("lbp config:")
lbp.query_pair()
print(f'whale address is: {lbp.get("whale_token")}')
print(f'LBP lp token is: {lbp.get("lbp_lp_token")}')
wait()

print("terraswap config:")
terraswap_dapp.query_config()
print(f'treasury address is: {treasury.get("treasury")}')
print(f'multisig address is: {treasury.get("multisig")}')
print(MULTISIG_ADDRESS)
wait()


# Transfer the Whale and UST tokens to the treasury
wait()
terraswap_dapp.detailed_provide_liquidity("lbp_pair", [("whale", str(int(100*MILLION))), ("ust", str(int(1 * MILLION)))], None, False)

wait()
terraswap_dapp.withdraw_liquidity("lbp", 0, False)

print("WARNING, YOU ARE ABOUT TO CHANGE THE CONTRACT ADMINS")
wait()
# CHANGE ADMIN ON ALL CONTRACTS
treasury.set_admin(MULTISIG_ADDRESS)
terraswap_dapp.set_admin(MULTISIG_ADDRESS)

# exit()

# exit()
# print(deployer.wallet.key.acc_address)
# treasury.update_vault_assets()
# terraswap_dapp.auto_update_address_book()
# terraswap_dapp.update_address_book([("lbp","terra1lmjfpe5n74a5l7ljnh8qcuzg9mwd3fpp9lkzzf"), ("lbp_pair", "terra144yc27hvq3w0pcychtcu6jsfff7cu54tdj6jja")],[])
# terraswap_dapp.query_address_book("ust")
# terraswap_dapp.update_address_book([("whale","terra1mqeqfh4t746pmrx3u9tuqtqcmjlld6w7a6dxvt")],[])
# terraswap_dapp.query_address_book("lbp_pair")

# terraswap_dapp.update_address_book([],["lbp","lbp_pair"])
# exit()
# treasury.query_holding_amount("uluna")
# treasury.send_asset("uluna", 10000, "terra1khmttxmtsmt0983ggwcufalxkn07l4yj5thu3h")
# treasury.query_vault_asset("uluna")
# terraswap_dapp.swap("ust", "lbp_pair", int(100000))
# terraswap_dapp.provide_liquidity("lbp_pair", "whale", int(9000000))
# treasury.query_holding_value("uluna")

# LBP token id
# terraswap_dapp.withdraw_liquidity("lbp", 315511529)

exit()
