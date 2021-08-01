from datetime import timedelta

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())
#from bluna_pool_arb_bot import Arbbot
from bluna_swap_bot import Arbbot
from poolconfig import TERRASWAP_BLUNA_CONFIG as CONFIG, TERRASWAP_BLUNA_BOND_CONFIG as BOND_CONFIG
from loop import execute_loop


def main():
    client = LCDClient(url="https://tequila-lcd.terra.dev", chain_id="tequila-0004")
    mnemonic = 'main jar girl opinion train type cycle blood marble kitchen april champion amount engine crumble tunnel model vicious system student hood fee curious traffic'
    deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))

    # print(client.wasm.contract_query(CONFIG.contract_address, {"pool": {}}))

    bot = Arbbot(client=client, wallet=deployer, config=CONFIG, bond_contract=BOND_CONFIG.contract_address)
    execute_loop(op=bot, sleep_time=timedelta(seconds=3))


if __name__ == "__main__":
    main()
