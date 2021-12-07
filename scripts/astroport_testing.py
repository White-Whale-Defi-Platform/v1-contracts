import base64
import json

import pathlib
import sys
from typing import AsyncIterator
# temp workaround
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())



from terra_sdk.core.auth import StdFee
from white_whale.deploy import get_deployer
from terra_sdk.core.coins import Coin
from white_whale.contracts.astroport_pair import *
from white_whale.contracts.astroport_dapp import *
from white_whale.contracts.treasury import *

# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"
std_fee = StdFee(10*690000, "1200000uusd")


deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

test_pool = AstroportTestPool(deployer)
astroport_dapp = AstroportDAppContract(deployer)
treasury = TreasuryContract(deployer)

# test_pool.create()
# astroport_dapp.create()


# test_pool.provide_astro_liquidity()

# # Send white whale tokens to treasury?



# test_pool.send_lp_token_to_treasury(1000000)

# treasury.update_vault_assets()
whaletoken = test_pool.addresses["test_whale_token"]
pair_token = test_pool.addresses["astro_lp_token"] 
pool_addr = test_pool.addresses["astro_pair"]

treasury.query_vault_asset(pool_addr)
# treasury.query_balance(whaletoken)
# treasury.query_balance(pair_token)
astroport_dapp.withdraw_liquidity(pair_token,1000)

# astroport provide liquidity
# astroport_dapp.provide_liquidity(pool_addr, whaletoken, 20000)