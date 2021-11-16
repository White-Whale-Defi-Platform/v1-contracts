import base64
import json

import pathlib
import sys
# temp workaround
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.core.auth import StdFee
from white_whale.deploy import get_deployer
from terra_sdk.core.coins import Coin
from white_whale.contracts.treasury import *
from white_whale.contracts.terraswap_dapp import *

# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"
std_fee = StdFee(10*690000, "1200000uusd")

# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

treasury = TreasuryContract(deployer)
terraswap_dapp = TerraswapDAppContract(deployer)

create = False

if create:
    # treasury.create()
    terraswap_dapp.create()
    treasury.add_trader(terraswap_dapp.address)


terraswap_dapp.query_config()
# terraswap_dapp.swap("ust", "luna_ust_pair", int(100000))
terraswap_dapp.provide_liquidity("luna_ust_pair", "luna", int(100000000))

exit()
