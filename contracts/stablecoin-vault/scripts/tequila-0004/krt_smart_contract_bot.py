from datetime import timedelta

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())
from pool_arb_bot import MILLION, get_arbbot
from config import get_tequila_config as get_config
from loop import execute_loop
from sender import Sender
from util import get_gas_prices


def main():
    client = LCDClient(url="https://tequila-lcd.terra.dev", chain_id="tequila-0004", 
                       gas_prices=get_gas_prices(), gas_adjustment="1.1")
    mnemonic = '<ADD_TEST_ACCOUNT_MNEMONIC>'
    deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))

    config = get_config("KRT")
    print(config.contract_address)
    bot = get_arbbot(client=client, wallet=deployer, config=config, sender=Sender(client=client, wallet=deployer, get_gas_prices=get_gas_prices))
    bot.trade_amount = 200000*MILLION
    bot.fee = "400000000" + config.denom
    execute_loop(op=bot, sleep_time=timedelta(seconds=3))


if __name__ == "__main__":
    main()
