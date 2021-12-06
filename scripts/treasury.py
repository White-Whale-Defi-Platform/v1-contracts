import base64
import json

import pathlib
import sys
from typing import List
# temp workaround
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract
from terra_sdk.core.auth import StdFee
from terra_sdk.core.bank import MsgSend
from white_whale.deploy import get_deployer
from terra_sdk.core.coins import Coin
from white_whale.contracts.treasury import *
from white_whale.contracts.terraswap_dapp import *

def execute_on_treasury_msg(msgs: any, coins: List[Coin]):
    msg = MsgExecuteContract(
        deployer.wallet.key.acc_address,
        treasury.address,
        {
            "trader_action": {
                "msgs": msgs
                }
        },
        coins,
    )
    return msg


# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"


# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

treasury = TreasuryContract(deployer)
terraswap_dapp = TerraswapDAppContract(deployer)

create = False

if create:
    treasury.create()
    # terraswap_dapp.create()
    treasury.add_trader(terraswap_dapp.address)
    treasury.add_trader(deployer.wallet.key.acc_address)

terraswap_dapp.query_config()

    
treasury.query_holding_amount("uluna")
# treasury.send_asset("uluna", 10000, "terra1khmttxmtsmt0983ggwcufalxkn07l4yj5thu3h")
# treasury.update_vault_assets()
# treasury.query_vault_asset("uluna")
# terraswap_dapp.swap("luna", "luna_ust_pair", int(10000000))
# terraswap_dapp.provide_liquidity("luna_ust_pair", "luna", int(9000000))
treasury.query_holding_value("uluna")
# terraswap_dapp.withdraw_liquidity("luna_ust", 10000)

exit()
