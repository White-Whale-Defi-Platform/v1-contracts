import base64
import json

import pathlib
import sys
# temp workaround
sys.path.append('/workspaces/devcontainer/terra-sdk-python')
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.core.auth import StdFee
from white_whale.deploy import get_deployer
from terra_sdk.core.coins import Coin
from white_whale.contracts.stable_vault_2 import *
from white_whale.contracts.stable_arb import *
from white_whale.contracts.community import *

# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"
std_fee = StdFee(6900000, "3500000uusd")

deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=std_fee)

profit_check = ProfitCheckContract(deployer)
vault = StableVaultContract_2(deployer)
ust_arb = StableArbContract(deployer)
community_fund = CommunityContract(deployer)
create = False

if create:
    # profit_check.create()
    # vault.create()
    # profit_check.set_vault(vault.address)
    # ust_arb.create()
    vault.add_to_whitelist(ust_arb.address)

# ust_arb.call_arb(1)
# print(vault.address)
# profit_check.get_vault()
# vault.query_vault_value()
# deployer.send_funds(ust_arb.address, [Coin("uusd", 10000000)])
vault.provide_liquidity(100_000_000_000)

# community_fund.simulate_deposit(1_000_000)

# vault.withdraw_all()
# vault.query_vault_value()


# lp_balance = vault.query_lp_balance()
# print(f'lp {lp_balance}')
# while True:
#     # vault.provide_liquidity(2_000_000)
#     lp_balance = vault.query_lp_balance()
#     vault.withdraw_liquidity(lp_balance/2)
# lp_balance = vault.query_lp_balance()
# print(f'lp {lp_balance}')

exit()

sc_addr = deployer.get_address_dict()
print(sc_addr)
vault = sc_addr["stablecoin_vault"]
lp_token_address = sc_addr["liquidity_token"]

result = deployer.client.wasm.contract_query(lp_token_address, {
    "balance": {
        "address": deployer.wallet.key.acc_address
    }
})
lp_balance = int(result["balance"])
