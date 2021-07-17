from datetime import datetime
from attr import dataclass

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.core.auth.data.tx import StdFee
from terra_sdk.core.coin import Coin
from terra_sdk.core.market import MsgSwap
from terra_sdk.core.wasm import MsgExecuteContract

from query import get_market_swap_rate, get_terraswap_rate, get_tobin_tax

MILLION = 1000000

COMMISSION=0.003
LUNA_DENOM="uluna"


@dataclass
class TerraswapConfig:
    contract_address: str = "",
    denom: str = ""


TERRASWAP_UST_CONFIG = TerraswapConfig(contract_address='terra156v8s539wtz0sjpn8y8a8lfg8fhmwa7fy22aff', denom='uusd')
TERRASWAP_KRT_CONFIG = TerraswapConfig(contract_address='terra1rfzwcdhhu502xws6r5pxw4hx8c6vms772d6vyu', denom='ukrw')


class Arbbot:
    def __init__(self, client: LCDClient, wallet: Wallet, config: TerraswapConfig, trade_amount: int = MILLION) -> None:
        self.denom: str = config.denom
        self.pool_address = config.contract_address
        self.client: LCDClient = client
        self.wallet: Wallet = wallet
        self.market_min_spread: float = float(self.client.market.parameters()['min_spread'])
        self.tobin_tax = get_tobin_tax(client=self.client, denom=self.denom)
        self.counter = 0
        self.trade_amount = trade_amount
        self.fee: str = "40000" + self.denom
        
    def get_profit_margin(self):
        return 0.002

    def substract_fees(self, amount):
        return amount- Coin.from_str(self.fee).amount

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

        terraswap_msg = {
            "swap": {
                "offer_asset": {
                "info": {
                        "native_token": { "denom": self.denom }
                    },
                    "amount": str(offer_amount)
                },
                "belief_price": str(terra_luna_to_stable/terraswap_stable_to_luna),
                "max_spread": "1000" 
            }
        }

        msgs=[
            MsgExecuteContract(
                self.wallet.key.acc_address,
                self.pool_address,
                terraswap_msg,
                [Coin.from_str(str(offer_amount) + self.denom)]
            ),
            MsgSwap(
                self.wallet.key.acc_address,
                Coin(LUNA_DENOM, int(terraswap_stable_to_luna)),
                self.denom
            ),
        ]
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

        terraswap_msg = {
            "swap": {
                "offer_asset": {
                "info": {
                        "native_token": { "denom": LUNA_DENOM }
                    },
                    "amount": str(int(terra_stable_to_luna))
                },
                "belief_price": str(terraswap_luna_to_stable/terra_stable_to_luna),
                "max_spread": "1000" 
            }
        }
        msgs=[
            MsgSwap(
                self.wallet.key.acc_address,
                Coin(self.denom, offer_amount),
                LUNA_DENOM
            ),
            MsgExecuteContract(
                self.wallet.key.acc_address,
                self.pool_address,
                terraswap_msg,
                [Coin.from_str(str(int(terra_stable_to_luna)) + LUNA_DENOM)]
            ),
        ]
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
