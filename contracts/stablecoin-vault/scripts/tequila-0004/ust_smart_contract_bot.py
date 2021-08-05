from datetime import timedelta

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())
from pool_arb_bot import get_arbbot
from config import get_tequila_config as get_config
from loop import execute_loop
from sender import Sender
from util import get_gas_prices


def main():
    client = LCDClient(url="https://tequila-lcd.terra.dev", chain_id="tequila-0004", 
                       gas_prices=get_gas_prices(), gas_adjustment="1.4")
    mnemonic = "main jar girl opinion train type cycle blood marble kitchen april champion amount engine crumble tunnel model vicious system student hood fee curious traffic"
    deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))

    contract_address = "terra17g2xw6rvn9l3f67m6nma7nnc9qqqs0w5za9ucf"
    bot = get_arbbot(client=client, wallet=deployer, config=get_config("UST"), contract_address=contract_address, sender=Sender(client=client, wallet=deployer, get_gas_prices=get_gas_prices))
    execute_loop(op=bot, sleep_time=timedelta(seconds=1))


if __name__ == "__main__":
    main()
