import base64
import json

import pathlib
import sys
import datetime

# temp workaround
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.core.auth import StdFee
from white_whale.deploy import get_deployer
from terra_sdk.core.coins import Coin
from white_whale.contracts.treasury import *
from white_whale.contracts.terraswap_dapp import *
from white_whale.contracts.lbp import *

# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
# mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"
mnemonic = "pill hole shiver wage infant danger salt dismiss steak weather shell bright grass company violin large pride vessel physical rain number rookie best three"

#####################
#   DEPLOY PARAMETERS
#####################

MILLION = 1_000_000

YEAR = 2021
MONTH = 12
DAY = 12
# 5 AM UTC
HOUR = 5
# Start time in linux language
START_TIME = (datetime.datetime(year=YEAR, month=MONTH, day=DAY, hour=HOUR) - datetime.datetime(1970, 1, 1))

NOW = datetime.datetime.utcnow() - datetime.datetime(1970, 1, 1)
print(f'Time from now {START_TIME - NOW}')
input()

# exit()

# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

lbp = LiquidityBootstrappingPool(deployer)

create = False

# lbp.unregister()
if create:
    lbp.create(False)
# lbp.unregister()
# lbp.swap_to_whale(10)
# lbp.create_external_whale_pair()
# lbp.migrate_liquidity()
# WHALE, UST
# lbp.provide_lbp_liquidity(1_000 * MIL,1_0 * MIL )
# print(lbp.addresses["lbp_pair"])
lbp.query_pool()
# print("")
lbp.query_pair()

exit()
