from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.core.auth.data.tx import StdFee
from terra_sdk.core.coins import Coins


def always_accept_fee(_fee: StdFee) -> bool:
    return True


class Sender:
    def __init__(self, client: LCDClient, wallet: Wallet, get_gas_prices) -> None:
        self.client: LCDClient = client
        self.wallet: Wallet = wallet
        self.get_gas_prices = get_gas_prices
        self.gas_denom: str = 'uusd'
        self.update_gas_prices()

    def update_gas_prices(self):
        self.client.gas_prices = self.get_gas_prices()

    def estimate_fee(self, tx) -> StdFee:
        estimated_fee = self.client.tx.estimate_fee(tx, fee_denoms=[self.gas_denom])
        print(f'estimated: {estimated_fee}')
        return estimated_fee

    def __call__(self, msgs, accept_fee=always_accept_fee):
        print(f'send (denom: {self.gas_denom})')
        tx = self.wallet.create_and_sign_tx(msgs=msgs, fee_denoms=[self.gas_denom])
        fee = self.estimate_fee(tx)
        if not accept_fee(fee):
            print(f'reject due to fee {fee}')
            return
        print('broadcast')
        result = self.client.tx.broadcast(tx)
        print('result')
        print(result)
        return result