from datetime import timedelta

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())
from pool_arb_bot import get_arbbot
from poolconfig import TERRASWAP_UST_CONFIG_COL_4 as CONFIG
from loop import execute_loop
from sender import Sender
from util import get_gas_prices_col_4


def main():
    client = LCDClient(url="https://lcd.terra.dev", chain_id="columbus-4")
    mnemonic = 'earn gesture bullet busy width stick farm mercy armed baby found distance tomorrow describe despair settle congress toward anchor shiver tongue cover virtual wave'
    deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))
    balance = client.bank.balance(address=deployer.key.acc_address)
    print(balance)

    bot = get_arbbot(client=client, wallet=deployer, config=CONFIG, sender=Sender(client=client, wallet=deployer, get_gas_prices=get_gas_prices_col_4))#, trade_amount=100000*MILLION)
    bot.fee = "40000" + bot.denom
    bot.profit_margin = 0.002
    execute_loop(op=bot, sleep_time=timedelta(seconds=1))


if __name__ == "__main__":
    main()
