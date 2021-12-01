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


# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"
std_fee = StdFee(10*690000, "1200000uusd")


deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

test_pool = AstroportTestPool(deployer)
astroport_dapp = AstroportDAppContract(deployer)

test_pool.new_token()
test_pool.create()

# astroport_dapp.create()

