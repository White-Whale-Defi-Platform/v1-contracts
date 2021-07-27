from datetime import datetime
from enum import Enum, auto

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.core.auth.data.tx import StdFee
from terra_sdk.core.coin import Coin
from terra_sdk.core.market import MsgSwap
from terra_sdk.core.wasm import MsgExecuteContract

from query import get_market_swap_rate, get_terraswap_rate, NativeToken
from poolconfig import PoolConfig

MILLION = 1000000

LUNA_DENOM="uluna"

class ArbResult(Enum):
    Success = auto(),
    CloseToOpportunity = auto(),
    NoOpportunity = auto()

def anchor_deposit_msg(sender: str, contract: str, amount: int):
    msg = {
        "anchor_deposit": {
            "amount": {
                "denom": "uusd",
                "amount": str(amount)
            }
        }
    }
    return MsgExecuteContract(
        sender=sender,
        contract=contract,
        execute_msg=msg
    )

def anchor_withdrawal_msg(sender: str, contract: str, amount: int):
    msg = {
        "anchor_withdraw": {
            "amount": str(amount)
        }
    }
    return MsgExecuteContract(
        sender=sender,
        contract=contract,
        execute_msg=msg
    )


class Sender:
    def __init__(self, client: LCDClient, wallet: Wallet) -> None:
        self.client: LCDClient = client
        self.wallet: Wallet = wallet
        self.gas_safety_factor: float = 1.1

    def __call__(self, msgs, fee):
        tx = self.wallet.create_and_sign_tx(msgs=msgs)
        estimated_fee = self.client.tx.estimate_fee(tx)
        tx = self.wallet.create_and_sign_tx(msgs=msgs, fee=StdFee(estimated_fee.gas*self.gas_safety_factor, fee))
        result = self.client.tx.broadcast(tx)
        print('result')
        print(result)
        return result


class Balances:
    def __init__(self, client: LCDClient, aust_contract: str) -> None:
        self._client: LCDClient = client
        self._aust_contract: str = aust_contract

    def uaust(self, contract_address: str):
        return int(self._client.wasm.contract_query(self._aust_contract, {
            "balance": {"address": contract_address}
        })["balance"])

    def uusd(self, contract_address: str):
        return int(self._client.bank.balance(contract_address)["uusd"].amount)



class AnchorModel:
    def __init__(self, sign_and_send: Sender, balances: Balances, max_deposit_ratio: float = 0.9) -> None:
        self.sign_and_send: Sender = sign_and_send
        self.balances: Balances = balances
        self.anchor_contract: str
        self.anchor_contract = 'terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal' # tequila-0004
        self.aust_contract: str = 'terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl' # tequila-0004
        self.max_deposit_ratio: float = max_deposit_ratio
        assert(0 < max_deposit_ratio < 1)
        self.fee: str = "85000uusd"
        self.min_withdrawal: int = 50*MILLION
        self.deposit_profit_margin_ratio: float = 0.5

    def aust_ust_exchange_rate(self) -> float:
        return float(self.sign_and_send.client.wasm.contract_query(self.anchor_contract, {"state": {}})["prev_exchange_rate"])

    def deposit(self, sender: str, contract: str):
        ust_balance = self.balances.uusd(contract)
        print(f'uusd={ust_balance}')
        aust_balance = self.balances.uaust(contract)
        print(f'aust={aust_balance}')
        aust_value_in_ust = aust_balance * self.aust_ust_exchange_rate()
        if ust_balance < 1.5*(1-self.max_deposit_ratio)*(ust_balance + aust_value_in_ust):
            print('low ust balance -> skipping deposit')
            return
        ust_balance += aust_balance * aust_value_in_ust
        print(f'uusd total={ust_balance}')

        deposit_amount = int(ust_balance*self.max_deposit_ratio - aust_value_in_ust) - MILLION
        print(f'deposit {deposit_amount}')
        if deposit_amount <= 0:
            print('insufficient funds for anchor deposit')
            return

        msg = anchor_deposit_msg(sender=sender, contract=contract, amount=deposit_amount)
        return self.sign_and_send(msgs=[msg], fee=self.fee)

    def withdraw(self, sender: str, contract: str):
        aust_balance = self.balances.uaust(contract)
        print(f'withdraw {aust_balance}')
        if aust_balance < self.min_withdrawal:
            print('insufficient funds for withdrawal')
            return

        msg = anchor_withdrawal_msg(sender=sender, contract=contract, amount=aust_balance)
        return self.sign_and_send(msgs=[msg], fee=self.fee)


class Arbbot:
    def __init__(self, client: LCDClient, wallet: Wallet, config: PoolConfig, get_messages, trade_amount: int = 100*MILLION, contract_address = None) -> None:
        self.denom: str = config.token.denom
        self.pool_address: str = config.contract_address
        self.contract_address: str = contract_address if contract_address else self.pool_address
        self.client: LCDClient = client
        self.wallet: Wallet = wallet
        self._sign_and_send: Sender = Sender(client=client, wallet=wallet)
        self.counter: int = 0
        self.trade_amount = trade_amount
        self.fee: str = "47000" + self.denom
        self._get_messages = get_messages
        self.profit_margin: float = 0.002
        self.aust_contract: str = 'terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl' # tequila-0004
        self._balances: Balances = Balances(client=self.client, aust_contract=self.aust_contract)
        self.anchor: AnchorModel = AnchorModel(sign_and_send=self._sign_and_send, balances=self._balances)
        
    def get_profit_margin(self):
        return self.profit_margin

    def substract_fees(self, amount):
        return amount - Coin.from_str(self.fee).amount

    def get_messages(self, offer_amount, luna_to_stable, stable_to_luna):
        return self._get_messages.above_peg(self, offer_amount=offer_amount, luna_to_stable=luna_to_stable, stable_to_luna=stable_to_luna)

    def try_arb_above(self) -> ArbResult:
        offer_amount = self.trade_amount
        # offer_amount = min(self.trade_amount, self._balances.uusd(self.contract_address))
        terraswap_stable_to_luna = get_terraswap_rate(client=self.client, offer=NativeToken(denom=self.denom), amount=offer_amount, pool_address=self.pool_address)
        terra_luna_to_stable = get_market_swap_rate(client=self.client, offer=Coin(denom=LUNA_DENOM, amount=int(terraswap_stable_to_luna)), ask_denom=self.denom)
        print(f"tx cost: {Coin.from_str(self.fee).amount/offer_amount}")
        profit_ratio = self.substract_fees(terra_luna_to_stable)/offer_amount
        print(f"simulated profit: {(profit_ratio - 1)*100}%")
        if profit_ratio < 1 + self.get_profit_margin():
            print("No arb opportunity above peg")
            print(f'overall arb opportunities = {self.counter}')
            if profit_ratio < 1 + self.get_profit_margin()*self.anchor.deposit_profit_margin_ratio:
                return ArbResult.NoOpportunity
            return ArbResult.CloseToOpportunity
        else:
            self.counter = self.counter + 1
            print(" >>> Found arb opportunity above peg")

        if profit_ratio > 1 + 3*profit_ratio:
            self.withdraw_from_anchor()
            offer_amount = self._balances.uusd(self.contract_address)

        msgs = self._get_messages.above_peg(self, offer_amount=offer_amount, luna_to_stable=terra_luna_to_stable, stable_to_luna=terraswap_stable_to_luna)
        print('send msg to contract')
        self.sign_and_send(msgs=msgs)
        return ArbResult.Success

    def try_arb_below(self) -> None:
        offer_amount = self.trade_amount
        # offer_amount = min(self.trade_amount, self._balances.uusd(self.contract_address))
        terra_stable_to_luna = get_market_swap_rate(client=self.client, offer=Coin(denom=self.denom, amount=offer_amount), ask_denom=LUNA_DENOM)
        terraswap_luna_to_stable = get_terraswap_rate(client=self.client, offer=NativeToken(denom=LUNA_DENOM), amount=int(terra_stable_to_luna), pool_address=self.pool_address)
        profit_ratio = self.substract_fees(terraswap_luna_to_stable)/offer_amount
        print(f"simulated profit: {(profit_ratio - 1)*100}%")
        if profit_ratio < 1 + self.get_profit_margin():
            print("No arb opportunity below peg")
            print(f'overall arb opportunities = {self.counter}')
            if profit_ratio < 1 + self.get_profit_margin()*self.anchor.deposit_profit_margin_ratio:
                return ArbResult.NoOpportunity
            return ArbResult.CloseToOpportunity
        else:
            self.counter = self.counter + 1
            print(" >>> Found arb opportunity below peg")

        if profit_ratio > 1 + 3*profit_ratio:
            self.withdraw_from_anchor()
            offer_amount = self._balances.uusd(self.contract_address)
        msgs = self._get_messages.below_peg(self, offer_amount=offer_amount, luna_to_stable=terraswap_luna_to_stable, stable_to_luna=terra_stable_to_luna)
        self.sign_and_send(msgs=msgs)
        return ArbResult.Success

    def sign_and_send(self, msgs):
        self._sign_and_send(msgs=msgs, fee=self.fee)

    def __call__(self) -> None:
        print("===")
        print(f'time: {datetime.now()}')
        above_result = self.try_arb_above()
        below_result = self.try_arb_below()
        if above_result == ArbResult.NoOpportunity and below_result == ArbResult.NoOpportunity:
            self.deposit_to_anchor()

    def deposit_to_anchor(self):
        self.anchor.deposit(sender=self.wallet.key.acc_address, contract=self.contract_address)

    def withdraw_from_anchor(self):
        self.anchor.withdraw(sender=self.wallet.key.acc_address, contract=self.contract_address)


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
                "max_spread": "0.1" 
            }
        }

        return [
            MsgExecuteContract(
                bot.wallet.key.acc_address,
                bot.pool_address,
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
                "max_spread": "0.1" 
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
                bot.pool_address,
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
