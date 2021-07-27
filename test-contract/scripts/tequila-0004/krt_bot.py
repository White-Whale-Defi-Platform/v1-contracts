from datetime import timedelta

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())
from pool_arb_bot import Arbbot, MILLION, BotMessages
from poolconfig import TERRASWAP_KRT_CONFIG
from loop import execute_loop


def main():
    client = LCDClient(url="https://tequila-lcd.terra.dev", chain_id="tequila-0004")
    mnemonic = "<ADD_TEST_ACCOUNT_MNEMONIC>"
    deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))

    config = TERRASWAP_KRT_CONFIG
    print(config.contract_address)
    bot = Arbbot(client=client, wallet=deployer, config=config, get_messages=BotMessages)
    bot.trade_amount = 200000*MILLION
    bot.fee = "400000000" + config.denom
    execute_loop(op=bot, sleep_time=timedelta(seconds=3))


if __name__ == "__main__":
    main()