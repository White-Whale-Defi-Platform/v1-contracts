from datetime import timedelta

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey
import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())

#from bluna_pool_arb_bot import Arbbot
from bluna_swap_bot import Arbbot
from poolconfig import TERRASWAP_BLUNA_CONFIG_COL_4 as CONFIG, TERRASWAP_BLUNA_BOND_CONFIG_COL_4 as BOND_CONFIG
from loop import execute_loop


def main():
    client = LCDClient(url="https://lcd.terra.dev", chain_id="columbus-4")
    mnemonic = 'earn gesture bullet busy width stick farm mercy armed baby found distance tomorrow describe despair settle congress toward anchor shiver tongue cover virtual wave'
    deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))

    # print(client.wasm.contract_query(CONFIG.contract_address, {"pool": {}}))

    bot = Arbbot(client=client, wallet=deployer, config=CONFIG, bond_contract=BOND_CONFIG.contract_address)    
    bot.fee = "5000uluna"
    execute_loop(op=bot, sleep_time=timedelta(seconds=3))


if __name__ == "__main__":
    main()
