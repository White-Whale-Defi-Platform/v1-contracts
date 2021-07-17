from datetime import timedelta

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())
from pool_arb_bot import Arbbot, SmartContractMessages, MILLION
from poolconfig import TERRASWAP_UST_CONFIG
from loop import execute_loop


def main():
    client = LCDClient(url="https://tequila-lcd.terra.dev", chain_id="tequila-0004")
    mnemonic = '<ADD_TEST_ACCOUNT_MNEMONIC>'
    deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))

    bot = Arbbot(client=client, wallet=deployer, config=TERRASWAP_UST_CONFIG, contract_address="terra1ezhgrm50vj3g0635t8s2y7em8n0e9frhpr8q63", get_messages=SmartContractMessages)
    execute_loop(op=bot, sleep_time=timedelta(seconds=3))


if __name__ == "__main__":
    main()
