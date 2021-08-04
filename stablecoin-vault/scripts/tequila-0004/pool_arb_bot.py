from datetime import datetime, timedelta, timezone
from enum import Enum, auto

from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.core.auth.data.tx import StdFee
from terra_sdk.core.coin import Coin
from terra_sdk.core.market import MsgSwap
from terra_sdk.core.wasm import MsgExecuteContract

from query import get_market_swap_rate, get_terraswap_rate, NativeToken
from poolconfig import PoolConfig
from sender import Sender

MILLION = 1000000

LUNA_DENOM = "uluna"


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


class Balances:
    def __init__(self, client: LCDClient, aust_contract: str) -> None:
        self._client: LCDClient = client
        self._aust_contract: str = aust_contract
        self._anchor_contract: str = 'terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal'  # tequila-0004
    def uaust(self, contract_address: str) -> int:
        return int(self._client.wasm.contract_query(self._aust_contract, {
            "balance": {"address": contract_address}
        })["balance"])

    def uusd(self, contract_address: str) -> int:
        try:
            return int(self._client.bank.balance(contract_address)["uusd"].amount)
        except KeyError:
            return 0

    def uluna(self, contract_address: str) -> int:
        try:
            return int(self._client.bank.balance(contract_address)["uluna"].amount)
        except KeyError:
            return 0

    def aust_ust_exchange_rate(self) -> float:
        return float(self._client.wasm.contract_query(self._anchor_contract, {"epoch_state": {}})["exchange_rate"])


class AnchorModel:
    def __init__(self, sign_and_send: Sender, balances: Balances, max_deposit_ratio: float = 0.9) -> None:
        self.sign_and_send: Sender = sign_and_send
        self.balances: Balances = balances
        self.aust_contract: str = 'terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl'  # tequila-0004
        self.max_deposit_ratio: float = max_deposit_ratio
        assert(0 < max_deposit_ratio < 1)
        self.fee: str = "85000uusd"
        self.min_withdrawal: int = 50*MILLION
        self.deposit_profit_margin_ratio: float = 0.5

    def deposit(self, sender: str, contract: str):
        ust_balance = self.balances.uusd(contract)
        print(f'uusd={ust_balance}')
        aust_balance = self.balances.uaust(contract)
        print(f'aust={aust_balance}')
        exchange_rate = self.balances.aust_ust_exchange_rate()
        print(f'rate={exchange_rate}')
        aust_value_in_ust = aust_balance * exchange_rate
        print(f'overall: {ust_balance + aust_value_in_ust}')
        if ust_balance < 1.5*(1-self.max_deposit_ratio)*(ust_balance + aust_value_in_ust):
            print('low ust balance -> skipping deposit')
            return
        ust_balance += aust_value_in_ust
        print(f'uusd total={ust_balance}')

        deposit_amount = int(
            ust_balance*self.max_deposit_ratio - aust_value_in_ust) - MILLION
        print(f'deposit {deposit_amount}')
        if deposit_amount <= 0:
            print('insufficient funds for anchor deposit')
            return

        msg = anchor_deposit_msg(
            sender=sender, contract=contract, amount=deposit_amount)
        return self.sign_and_send(msgs=[msg])


class ProfitabilityCheck:
    def __init__(self, profit_margin: float) -> None:
        self.profit_margin: float = profit_margin
        self.offer: int = 0
        self.received: int = 0
        self.tax_rate: float = 0.0
        self.__profit_ratio: float = 0.0
    
    @property
    def profit_ratio(self):
        return self.__profit_ratio

    def update(self, offer: int, received: int, tax_rate: float):
        self.offer = offer
        self.received = received
        self.tax_rate = tax_rate
    
    def __call__(self, fee: StdFee) -> bool:
        denom = fee.amount.denoms()[0]
        self.__profit_ratio = (self.received - fee.amount[denom].amount - self.offer*self.tax_rate)/self.offer
        return self.__profit_ratio > 1 + self.profit_margin

class Arbbot:
    def __init__(self, client: LCDClient, wallet: Wallet, config: PoolConfig, msg_sender, sender: Sender, trade_amount: int = 95*MILLION, contract_address=None) -> None:
        self.denom: str = config.token.denom
        self.pool_address: str = config.contract_address
        self.contract_address: str = contract_address if contract_address else wallet.key.acc_address
        self.client: LCDClient = client
        self.wallet: Wallet = wallet
        self._sign_and_send: Sender = sender
        self.counter: int = 0
        self.trade_amount = trade_amount
        self.fee: StdFee = StdFee(gas=80000, amount="80000" + self.denom)
        self.msg_sender = msg_sender
        self.profitability_check: ProfitabilityCheck = ProfitabilityCheck(profit_margin=0.0015)
        self.withdraw_profit_margin_ratio: float = 3.0
        self.deposit_profit_margin_ratio: float = 0.5
        self.aust_contract: str = 'terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl'  # tequila-0004
        self._balances: Balances = Balances(
            client=self.client, aust_contract=self.aust_contract)
        self.last_gas_update: datetime = datetime.now(tz=timezone.utc)
        self.gas_update_period: timedelta = timedelta(minutes=10)

    def get_profit_margin(self) -> float:
        return self.profitability_check.profit_margin

    def substract_fees(self, amount) -> int:
        return amount - self.fee.amount[self.denom].amount

    def get_min_offer_amount(self):
        return int(min(self.trade_amount, self._balances.uusd(self.contract_address))*0.99) - MILLION

    def get_max_offer_amount(self):
        uust_amount = self._balances.uusd(self.contract_address)
        uaust_amount = self._balances.uaust(self.contract_address)
        uaust_uust_rate = self._balances.aust_ust_exchange_rate()
        sum = uust_amount + uaust_amount * uaust_uust_rate
        return int(sum*0.99) - MILLION

    def try_arb_above(self) -> ArbResult:
        offer_amount = self.get_min_offer_amount()
        terraswap_stable_to_luna = get_terraswap_rate(client=self.client, offer=NativeToken(
            denom=self.denom), amount=offer_amount, pool_address=self.pool_address)
        terra_luna_to_stable = get_market_swap_rate(client=self.client, offer=Coin(
            denom=LUNA_DENOM, amount=int(terraswap_stable_to_luna)), ask_denom=self.denom)
        print(f'{offer_amount} uusd -> {terraswap_stable_to_luna} uluna -> {terra_luna_to_stable} uusd -> {terra_luna_to_stable/offer_amount}')
        self.profitability_check.update(offer=offer_amount, received=terra_luna_to_stable, tax_rate=self.client.treasury.tax_rate())
        print(self.fee.amount[self.denom].amount)
        print(f"tx cost: {self.fee.amount[self.denom].amount/offer_amount}")
        is_profitable = self.profitability_check(self.fee)
        print(f"simulated profit: {(self.profitability_check.profit_ratio - 1)*100}%")
        if not is_profitable:
            print("No arb opportunity above peg")
            print(f'overall arb opportunities = {self.counter}')
            if self.profitability_check.profit_ratio < 1 + self.get_profit_margin()*self.deposit_profit_margin_ratio:
                return ArbResult.NoOpportunity
            return ArbResult.CloseToOpportunity
        else:
            self.counter = self.counter + 1
            print(" >>> Found arb opportunity above peg")

        uaust_withdraw_amount = 0
        if self.profitability_check.profit_ratio > 1 + self.withdraw_profit_margin_ratio*self.get_profit_margin():
            uaust_withdraw_amount = self._balances.uaust(self.contract_address)
            offer_amount = self.get_max_offer_amount()

        print(f'offer {offer_amount}')
        msgs = self.msg_sender.above_peg(
            offer_amount=offer_amount, luna_to_stable=terra_luna_to_stable, stable_to_luna=terraswap_stable_to_luna, uaust_withdraw_amount=uaust_withdraw_amount)
        self.sign_and_send(msgs=msgs)
        return ArbResult.Success

    def try_arb_below(self) -> None:
        offer_amount = self.get_min_offer_amount()
        terra_stable_to_luna = get_market_swap_rate(client=self.client, offer=Coin(
            denom=self.denom, amount=offer_amount), ask_denom=LUNA_DENOM)
        terraswap_luna_to_stable = get_terraswap_rate(client=self.client, offer=NativeToken(
            denom=LUNA_DENOM), amount=int(terra_stable_to_luna), pool_address=self.pool_address)
        print(f'{offer_amount} uusd -> {terra_stable_to_luna} uluna -> {terraswap_luna_to_stable} uusd -> {terraswap_luna_to_stable/offer_amount}')
        self.profitability_check.update(offer=offer_amount, received=terraswap_luna_to_stable, tax_rate=self.client.treasury.tax_rate())
        is_profitable = self.profitability_check(self.fee)
        print(f"simulated profit: {(self.profitability_check.profit_ratio - 1)*100}%")
        if not is_profitable:
            print("No arb opportunity below peg")
            print(f'overall arb opportunities = {self.counter}')
            if self.profitability_check.profit_ratio < 1 + self.get_profit_margin()*self.deposit_profit_margin_ratio:
                return ArbResult.NoOpportunity
            return ArbResult.CloseToOpportunity
        else:
            self.counter = self.counter + 1
            print(" >>> Found arb opportunity below peg")

        uaust_withdraw_amount = 0
        if self.profitability_check.profit_ratio > 1 + self.withdraw_profit_margin_ratio*self.get_profit_margin():
            uaust_withdraw_amount = self._balances.uaust(self.contract_address)
            offer_amount = self.get_max_offer_amount()

        print(f'offer {offer_amount}')
        msgs = self.msg_sender.below_peg(
            offer_amount=offer_amount, luna_to_stable=terraswap_luna_to_stable, stable_to_luna=terra_stable_to_luna, uaust_withdraw_amount=uaust_withdraw_amount)
        self.sign_and_send(msgs=msgs)
        return ArbResult.Success

    def sign_and_send(self, msgs):
        self._sign_and_send(msgs=msgs, accept_fee=self.profitability_check)

    def __call__(self) -> None:
        print("===")
        print(f'time: {datetime.now()}')
        above_result = self.try_arb_above()
        below_result = self.try_arb_below()
        if above_result == ArbResult.NoOpportunity and below_result == ArbResult.NoOpportunity:
            self.msg_sender.deposit_to_anchor()

        now = datetime.now(tz=timezone.utc)
        if now - self.last_gas_update > self.gas_update_period:
            self._sign_and_send.update_gas_prices()


class BotMessages:
    def __init__(self, sender: str, contract: str, denom: str) -> None:
        self.sender: str = sender
        self.contract: str = contract
        self.denom: str = denom

    def above_peg(self, offer_amount: int, luna_to_stable: int, stable_to_luna: int, uaust_withdraw_amount: int):
        terraswap_msg = {
            "swap": {
                "offer_asset": {
                    "info": {
                        "native_token": {"denom": self.denom}
                    },
                    "amount": str(offer_amount)
                },
                "belief_price": str(float(luna_to_stable)/stable_to_luna),
                "max_spread": "0.01"
            }
        }

        return [
            MsgExecuteContract(
                sender=self.sender,
                contract=self.contract,
                execute_msg=terraswap_msg,
                coins=[Coin.from_str(str(offer_amount) + self.denom)]
            ),
            MsgSwap(
                trader=self.sender,
                offer_coin=Coin(LUNA_DENOM, int(0.995*stable_to_luna)),
                ask_denom=self.denom
            ),
        ]

    def below_peg(self, offer_amount: int, luna_to_stable: int, stable_to_luna: int, uaust_withdraw_amount: int):
        luna_offer_amount = int(0.995*stable_to_luna)
        terraswap_msg = {
            "swap": {
                "offer_asset": {
                    "info": {
                        "native_token": {"denom": LUNA_DENOM}
                    },
                    "amount": str(luna_offer_amount)
                },
                "belief_price": str(int(float(luna_to_stable)/stable_to_luna*MILLION)),
                "max_spread": "0.01"
            }
        }
        return [
            MsgSwap(
                trader=self.sender,
                offer_coin=Coin(self.denom, offer_amount),
                ask_denom=LUNA_DENOM
            ),
            MsgExecuteContract(
                sender=self.sender,
                contract=self.contract,
                execute_msg=terraswap_msg,
                coins=[Coin.from_str(str(luna_offer_amount) + LUNA_DENOM)]
            ),
        ]

    def deposit_to_anchor(self):
        print('not implemented')

class SmartContractMessages:
    def __init__(self, sender: str, contract: str, denom: str, anchor: AnchorModel) -> None:
        self.sender: str = sender
        self.contract: str = contract
        self.denom: str = denom
        self.anchor: AnchorModel = anchor

    def _get_msg(self, direction: str, offer_amount: int, uaust_withdraw_amount: int):
        msg = {
            direction: {
                "amount": {
                    "denom": self.denom,
                    "amount": str(offer_amount)
                },
                "uaust_withdraw_amount": str(uaust_withdraw_amount)
            }
        }
        return [
            MsgExecuteContract(
                sender=self.sender,
                contract=self.contract,
                execute_msg=msg
            ),
        ]

    def above_peg(self, offer_amount: int, luna_to_stable: int, stable_to_luna: int, uaust_withdraw_amount: int):
        return self._get_msg("above_peg", offer_amount, uaust_withdraw_amount)

    def below_peg(self, offer_amount: int, luna_to_stable: int, stable_to_luna: int, uaust_withdraw_amount: int):
        return self._get_msg("below_peg", offer_amount, uaust_withdraw_amount)

    def deposit_to_anchor(self):
        print(f'deposit')
        self.anchor.deposit(sender=self.sender, contract=self.contract)


def get_arbbot(client: LCDClient, wallet: Wallet, config: PoolConfig, sender: Sender, trade_amount: int = 95*MILLION, contract_address=None) -> Arbbot:
    if contract_address:
        balances = Balances(client=client, aust_contract='terra1ajt556dpzvjwl0kl5tzku3fc3p3knkg9mkv8jl') # tequila-0004
        anchor = AnchorModel(sign_and_send=sender, balances=balances)
        msg_sender = SmartContractMessages(sender=wallet.key.acc_address, contract=contract_address, denom=config.token.denom, anchor=anchor)
        return Arbbot(client=client, wallet=wallet, config=config, msg_sender=msg_sender, trade_amount=trade_amount, contract_address=contract_address, sender=sender)
    
    msg_sender = BotMessages(sender=wallet.key.acc_address, contract=config.contract_address, denom=config.token.denom)
    return Arbbot(client=client, wallet=wallet, config=config, msg_sender=msg_sender, trade_amount=trade_amount, sender=sender)
