from datetime import timedelta

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())
from pool_arb_bot import get_arbbot
from poolconfig import TERRASWAP_UST_CONFIG
from loop import execute_loop
from util import get_gas_prices


def main():
    client = LCDClient(url="https://tequila-lcd.terra.dev", chain_id="tequila-0004", 
                       gas_prices=get_gas_prices(), gas_adjustment="1.1")
    mnemonic = '<ADD_TEST_ACCOUNT_MNEMONIC>'
    deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))

    contract_address = "terra1ky5hkd0jwn8t76n85wkzmywa7qzrgrh50g4s5x"
    # contract_address = "terra1ezhgrm50vj3g0635t8s2y7em8n0e9frhpr8q63"
    bot = get_arbbot(client=client, wallet=deployer, config=TERRASWAP_UST_CONFIG, contract_address=contract_address, get_gas_prices=get_gas_prices)
    execute_loop(op=bot, sleep_time=timedelta(seconds=1))


if __name__ == "__main__":
    main()
