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
std_fee = StdFee(10*690000, "1200000uusd")

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
# terraswap_dapp.set_treasury_addr()
# treasury.update_vault_assets([],[])
# SendMsg = MsgSend(
#         treasury.address,
#         deployer.wallet.key.acc_address,
#         "1000000uluna" # send 1 luna
#     ).to_data()
# print(SendMsg)
# SendMsg.type = "bank"
# SendMsg = SendMsg.to_json()
# print(SendMsg)
# msg = execute_on_treasury_msg(msgs=[SendMsg], coins=[])

# tx = deployer.wallet.create_and_sign_tx(
#     msgs=[msg],
#     memo="test transaction!",
#     fee=StdFee(200000, "120000uluna")
# )

# result = deployer.client.tx.broadcast(tx)
# print(result)
# exit()
treasury.query_vault_asset("uluna")
treasury.query_config()
treasury.query_holding_amount("uluna")
# treasury.send_native_token("uluna", 10_000, "terra1vjjs2cekma9s0axev989e68lkdx7mgjl0pnaak")
treasury.query_holding_amount("uluna")
# treasury.query_holding_value("terra1srf30cs8ax73y59gm64lkztnx0zexl8fpv3kx2")
# treasury.query_lp_balance()
# treasury.query_holding_value("uusd")
# terraswap_dapp.swap("luna", "luna_ust_pair", int(10000000))
# terraswap_dapp.provide_liquidity("luna_ust_pair", "ust", int(100_000_000))
# terraswap_dapp.withdraw_liquidity("luna_ust", 485111)

exit()
