from terra_sdk.core.bank import MsgSend
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract
import base64
import json

import pathlib
import sys
from typing import List
# temp workaround
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from white_whale.contracts.terraswap_dapp import *
from white_whale.contracts.treasury import *
from terra_sdk.core.coins import Coin
from white_whale.deploy import get_deployer

# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
# mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"
mnemonic = "pill hole shiver wage infant danger salt dismiss steak weather shell bright grass company violin large pride vessel physical rain number rookie best three"
# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

treasury = TreasuryContract(deployer)
terraswap_dapp = TerraswapDAppContract(deployer)


# STEP 1
# Create Treasury
create = False
if create:
    # treasury.create()

    print("PRESS ENTER AFTER YOU DEPLOYED THE LBP")
    # input()
    # STEP 3
    # Create terraswap dApp
    terraswap_dapp.create(True)
    print(terraswap_dapp.address)
    treasury.add_dapp(terraswap_dapp.address)



# exit()

# terraswap_dapp.auto_update_address_book()
# exit()
# print(deployer.wallet.key.acc_address)
# treasury.update_vault_assets()
terraswap_dapp.query_config()
# terraswap_dapp.auto_update_address_book()
# exit()
# terraswap_dapp.update_address_book([("lbp","terra1lmjfpe5n74a5l7ljnh8qcuzg9mwd3fpp9lkzzf"), ("lbp_pair", "terra144yc27hvq3w0pcychtcu6jsfff7cu54tdj6jja")],[])
# terraswap_dapp.query_address_book("ust")
# terraswap_dapp.update_address_book([("whale","terra1mqeqfh4t746pmrx3u9tuqtqcmjlld6w7a6dxvt")],[])
# terraswap_dapp.query_address_book("lbp_pair")

# terraswap_dapp.set_trader(MULTISIG_ADDRESS)
exit()
# CHANGE ADMIN ON ALL CONTRACTS
treasury.set_admin(MULTISIG_ADDRESS)
terraswap_dapp.set_admin(MULTISIG_ADDRESS)


# terraswap_dapp.detailed_provide_liquidity("lbp_pair", [("whale", str(int(1000000000))), ("ust", str(int(40000000)))], None)
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
