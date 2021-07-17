from datetime import datetime

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.core.auth.data.tx import StdFee
from terra_sdk.core.coin import Coin
from terra_sdk.core.market import MsgSwap
from terra_sdk.core.wasm import MsgExecuteContract

from query import get_market_swap_rate, get_terraswap_rate, get_tobin_tax
from poolconfig import PoolConfig

MILLION = 1000000

COMMISSION=0.003
LUNA_DENOM="uluna"


class Arbbot:
    def __init__(self, client: LCDClient, wallet: Wallet, config: PoolConfig, get_messages, trade_amount: int = 100*MILLION, contract_address = None) -> None:
        self.denom: str = config.denom
        self.pool_address = config.contract_address
        self.contract_address = contract_address if contract_address else self.pool_address
        self.client: LCDClient = client
        self.wallet: Wallet = wallet
        self.market_min_spread: float = float(self.client.market.parameters()['min_spread'])
        self.tobin_tax = get_tobin_tax(client=self.client, denom=self.denom)
        self.counter = 0
        self.trade_amount = trade_amount
        self.fee: str = "45000" + self.denom
        self._get_messages = get_messages
        
    def get_profit_margin(self):
        return 0.002

    def substract_fees(self, amount):
        return amount - Coin.from_str(self.fee).amount

    def get_messages(self, offer_amount, luna_to_stable, stable_to_luna):
        return self._get_messages.above_peg(self, offer_amount=offer_amount, luna_to_stable=luna_to_stable, stable_to_luna=stable_to_luna)

    def try_arb_above(self) -> None:
        offer_amount = self.trade_amount
        terraswap_stable_to_luna = get_terraswap_rate(client=self.client, offer=Coin(denom=self.denom, amount=offer_amount), pool_address=self.pool_address)
        terra_luna_to_stable = get_market_swap_rate(client=self.client, offer=Coin(denom=LUNA_DENOM, amount=int(terraswap_stable_to_luna)), ask_denom=self.denom)
        print(f"tx cost: {Coin.from_str(self.fee).amount/offer_amount}")
        profit_ratio = self.substract_fees(terra_luna_to_stable)/offer_amount
        print(f"simulated profit: {(profit_ratio - 1)*100}%")
        if profit_ratio < 1 + self.get_profit_margin():
            print("No arb opportunity above peg")
            print(f'overall arb opportunities = {self.counter}')
            return
        else:
            self.counter = self.counter + 1
            print(" >>> Found arb opportunity above peg")

        msgs = self._get_messages.above_peg(self, offer_amount=offer_amount, luna_to_stable=terra_luna_to_stable, stable_to_luna=terraswap_stable_to_luna)
        self.sign_and_send(msgs=msgs)
        

    def try_arb_below(self) -> None:
        offer_amount = self.trade_amount
        terra_stable_to_luna = get_market_swap_rate(client=self.client, offer=Coin(denom=self.denom, amount=offer_amount), ask_denom=LUNA_DENOM)
        terraswap_luna_to_stable = get_terraswap_rate(client=self.client, offer=Coin(denom=LUNA_DENOM, amount=int(terra_stable_to_luna)), pool_address=self.pool_address)
        profit_ratio = self.substract_fees(terraswap_luna_to_stable)/offer_amount
        print(f"simulated profit: {(profit_ratio - 1)*100}%")
        if profit_ratio < 1 + self.get_profit_margin():
            print("No arb opportunity below peg")
            print(f'overall arb opportunities = {self.counter}')
            return
        else:
            self.counter = self.counter + 1
            print(" >>> Found arb opportunity below peg")


        msgs = self._get_messages.below_peg(self, offer_amount=offer_amount, luna_to_stable=terraswap_luna_to_stable, stable_to_luna=terra_stable_to_luna)
        self.sign_and_send(msgs=msgs)

    def sign_and_send(self, msgs):
        tx = self.wallet.create_and_sign_tx(msgs=msgs)
        estimated_fee = self.client.tx.estimate_fee(tx)
        tx = self.wallet.create_and_sign_tx(msgs=msgs, fee=StdFee(estimated_fee.gas*1.1, self.fee))
        result = self.client.tx.broadcast(tx)
        print(result)
        return result

    def __call__(self) -> None:
        print("===")
        print(f'time: {datetime.now()}')
        self.try_arb_above()
        self.try_arb_below()


class BotMessages:
    @staticmethod
    def above_peg(bot: Arbbot, offer_amount: int, luna_to_stable: int, stable_to_luna: int):
        terraswap_msg = {
            "swap": {
                "offer_asset": {
                "info": {
                        "native_token": { "denom": bot.denom }
                    },
                    "amount": str(offer_amount)
                },
                "belief_price": str(float(luna_to_stable)/stable_to_luna),
                "max_spread": "1000" 
            }
        }

        return [
            MsgExecuteContract(
                bot.wallet.key.acc_address,
                bot.contract_address,
                terraswap_msg,
                [Coin.from_str(str(offer_amount) + bot.denom)]
            ),
            MsgSwap(
                bot.wallet.key.acc_address,
                Coin(LUNA_DENOM, stable_to_luna),
                bot.denom
            ),
        ]

    @staticmethod
    def below_peg(bot: Arbbot, offer_amount: int, luna_to_stable: int, stable_to_luna: int):
        terraswap_msg = {
            "swap": {
                "offer_asset": {
                "info": {
                        "native_token": { "denom": LUNA_DENOM }
                    },
                    "amount": str(stable_to_luna)
                },
                "belief_price": str(int(float(luna_to_stable)/stable_to_luna*MILLION)),
                "max_spread": "1000" 
            }
        }
        return [
            MsgSwap(
                bot.wallet.key.acc_address,
                Coin(bot.denom, offer_amount),
                LUNA_DENOM
            ),
            MsgExecuteContract(
                bot.wallet.key.acc_address,
                bot.contract_address,
                terraswap_msg,
                [Coin.from_str(str(stable_to_luna) + LUNA_DENOM)]
            ),
        ]


class SmartContractMessages:
    @staticmethod
    def _get_msg(direction: str, bot: Arbbot, offer_amount: int, luna_to_stable: int, stable_to_luna: int):
        msg = {
            direction: {
                "amount": {
                    "denom": bot.denom,
                    "amount": str(offer_amount)
                },
                "luna_price": {
                    "denom": bot.denom,
                    "amount": str(int(float(luna_to_stable)/stable_to_luna*MILLION))
                }
            }
        }
        return [
            MsgExecuteContract(
                bot.wallet.key.acc_address,
                bot.contract_address,
                msg
            ),
        ]

    @staticmethod
    def above_peg(bot: Arbbot, offer_amount: int, luna_to_stable: int, stable_to_luna: int):
        return SmartContractMessages._get_msg("above_peg", bot, offer_amount, luna_to_stable, stable_to_luna)

    @staticmethod
    def below_peg(bot: Arbbot, offer_amount: int, luna_to_stable: int, stable_to_luna: int):
        return SmartContractMessages._get_msg("below_peg", bot, offer_amount, luna_to_stable, stable_to_luna)
