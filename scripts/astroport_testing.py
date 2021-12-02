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
<<<<<<< HEAD

whaletoken = test_pool.addresses["test_whale_token"]
pair_token = test_pool.addresses["astro_lp_token"] 
pool_addr = test_pool.addresses["astro_pair"]

# test_pool.create()
astroport_dapp.create()



# print(astroport_dapp.auto_update_address_book())

# treasury.query_balance(whaletoken)
# treasury.query_balance(pair_token)

# print("withdraw liquidity")
astroport_dapp.provide_liquidity("twhale_ust","twhale",1000)


# astroport provide liquidity
# astroport_dapp.provide_liquidity(pool_addr, whaletoken, 20000)
=======

# test_pool.create()
# astroport_dapp.create()


# test_pool.provide_astro_liquidity()

# Send white whale tokens to treasury?



# test_pool.send_lp_token_to_treasury(1000000)

# asset_to_add =  { 
#             "asset": {
#                 "info": {
#                     "token": { "contract_addr": "terra1s8mvwktsz2rqjrw5xtgta3764nszpzxcuczj55" }
#                 },
#                 "amount": str(int(0))
#             },
#             "value_reference": {
#                 "liquidity": {
#                     "pool_address": "terra1x034d0sxw0vspkh7axlljchv0c4yw2qk9dh23r"
#                 }
#             }
#         }
# treasury.update_vault_assets([asset_to_add],[])

treasury.query_lp_balance()

# astroport provide liquidity
astroport_dapp.provide_liquidity("terra1x034d0sxw0vspkh7axlljchv0c4yw2qk9dh23r", "terra1mjjmnqkt7e4kfumsp5zkxhdah75xfev0aasf7d", 20000)
>>>>>>> astroport dapp
