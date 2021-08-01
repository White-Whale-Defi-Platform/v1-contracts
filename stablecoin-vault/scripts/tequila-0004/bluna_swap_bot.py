import base64
from datetime import datetime
import json

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.core.auth.data.tx import StdFee
from terra_sdk.core.coin import Coin
from terra_sdk.core.coins import Coins
from terra_sdk.core.wasm import MsgExecuteContract

from query import get_terraswap_rate, get_tobin_tax, NativeToken, Token
from poolconfig import PoolConfig

MILLION = 1000000

COMMISSION=0.003
LUNA_DENOM="uluna"


class Arbbot:
    def __init__(self, client: LCDClient, wallet: Wallet, config: PoolConfig, bond_contract: str, contract_address = None) -> None:
        self.token_contract_address: str = config.token.contract_addr
        self.denom="ubluna"
        self.pool_address = config.contract_address
        self.contract_address = contract_address if contract_address else self.pool_address
        self.client: LCDClient = client
        self.wallet: Wallet = wallet
        self.market_min_spread: float = float(self.client.market.parameters()['min_spread'])
        self.tobin_tax = get_tobin_tax(client=self.client, denom=self.denom)
        self.counter = 0
        self.fee: str = "80000" + LUNA_DENOM
        self.profit_margin = 0.025
        self.bond_contract: str = bond_contract # bluna hub / tequila-0004
        
    def get_profit_margin(self):
        return self.profit_margin

    def substract_fees(self, amount):
        return amount - Coin.from_str(self.fee).amount

    def get_messages(self, offer_amount, luna_to_stable, stable_to_luna):
        return self._get_messages.above_peg(self, offer_amount=offer_amount, luna_to_stable=luna_to_stable, stable_to_luna=stable_to_luna)

    def try_bluna_to_luna_swap(self) -> None:
        result = self.client.wasm.contract_query(self.token_contract_address, {
            "balance": {"address": self.wallet.key.acc_address}
        })
        offer_amount = int(result['balance'])
        if offer_amount == 0:
            print('no bluna funds')
            return
        terraswap_bluna_to_luna = get_terraswap_rate(client=self.client, offer=Token(contract_addr=self.token_contract_address), amount=offer_amount, pool_address=self.pool_address)

        profit_ratio = self.substract_fees(terraswap_bluna_to_luna)/offer_amount

        print(f"offer amount: {offer_amount}")
        print(f"swap rate: {terraswap_bluna_to_luna/offer_amount}")
        print(f"simulated profit: {(profit_ratio-1)*100}%")
        if profit_ratio < 0.995:
            print("No  swap back bluna to luna")
            print(f'overall swap opportunities = {self.counter}')
            return
        else:
            self.counter = self.counter + 1
            print(" >>> Found arb opportunity from bluna to luna")
            
        msg = base64.b64encode(bytes(json.dumps({"swap": {"belief_price": str(terraswap_bluna_to_luna/offer_amount), "max_spread": "0.01"}}), 'ascii')).decode()
        terraswap_msg = {
            "send": {
                "amount": str(offer_amount),
                "contract": self.pool_address,
                "msg": msg
            }
        }

        msgs = [
            MsgExecuteContract(
                sender = self.wallet.key.acc_address,
                contract = self.token_contract_address,
                execute_msg=terraswap_msg
            )
        ]
        self.sign_and_send(msgs=msgs)

    def try_luna_to_bluna_swap(self) -> None:
        result = self.client.bank.balance(self.wallet.key.acc_address)
        offer_amount = int(int(result['uluna'].amount)*0.99)
        if offer_amount == 0:
            print('no luna funds')
            return
        terraswap_luna_to_bluna = get_terraswap_rate(client=self.client, offer=NativeToken(denom=LUNA_DENOM), amount=offer_amount, pool_address=self.pool_address)
        profit_ratio = self.substract_fees(terraswap_luna_to_bluna)/offer_amount

        print(f"offer amount: {offer_amount}")
        print(f"swap rate: {terraswap_luna_to_bluna/offer_amount}")
        print(f"simulated profit: {(profit_ratio-1)*100}%")
        if profit_ratio < 1 + self.get_profit_margin():
            print("No arb opportunity from luna to bluna")
            print(f'overall arb opportunities = {self.counter}')
            return
        else:
            self.counter = self.counter + 1
            print(" >>> Found arb opportunity from luna to bluna")

        terraswap_msg = {
            "swap": {
                "offer_asset": {
                    "info": {
                        "native_token": { "denom": LUNA_DENOM }
                    },
                    "amount": str(offer_amount)
                }
            }
        }
        
        msgs = [
            MsgExecuteContract(
                sender = self.wallet.key.acc_address,
                contract = self.pool_address,
                execute_msg=terraswap_msg,
                coins=Coins(str(offer_amount) + LUNA_DENOM)
            )
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
        self.try_bluna_to_luna_swap()
        self.try_luna_to_bluna_swap()
