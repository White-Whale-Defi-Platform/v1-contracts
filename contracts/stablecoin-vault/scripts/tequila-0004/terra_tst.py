import terra_sdk
from terra_sdk.client.lcd import LCDClient
from terra_sdk.core.coins import Coins
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgExecuteContract
from terra_sdk.core.market import MsgSwap
from terra_sdk.core import Coin

def main():
	#terra = LCDClient(chain_id="columbus-4", url="https://lcd.terra.dev")
	#address = 'terra1h37fmc7uuw36lutpljycghvje886s687k6k20l'
	terra = LCDClient(chain_id="tequila-0004", url="https://tequila-lcd.terra.dev")
	address = 'terra1lzquc5em3qrz6e2uyp9se60un4e7wnpf5yvz97'
	contract_address = 'terra1z3sf42ywpuhxdh78rr5vyqxpaxa0dx657x5trs' # ust -> luna on tequila-0004
	terraswap_address = 'terra156v8s539wtz0sjpn8y8a8lfg8fhmwa7fy22aff' # ust-luna terraswap pool on tequila-0004

	mnemonic = 'main jar girl opinion train type cycle blood marble kitchen april champion amount engine crumble tunnel model vicious system student hood fee curious traffic'
	key = MnemonicKey(mnemonic)
	wallet = terra_sdk.client.lcd.wallet.Wallet(terra, key)

	bank = terra_sdk.client.lcd.api.bank.BankAPI(terra)
	balance = bank.balance(address)
	print(f'account balance: {balance}')
	print(terra.oracle.parameters())

	min_receive = "130665"
	execute_msg = {
	  "assert_limit_order": {
	    "offer_coin": {
	      "denom": "uusd",
	      "amount": "1000000"
	    },
	    "ask_denom": "uluna",
	    "minimum_receive": min_receive
	  }
	}

	execute_terraswap_msg = {
	  "swap": {
	    "offer_asset": {
		"info": {
		  "native_token": { "denom": "uluna" }
		},
		"amount": min_receive
	    },
	    "belief_price": "6899197",
	    "max_spread": "10000000",
	    "to": address
	  }
	}

	tx = wallet.create_and_sign_tx(
	    msgs=[
			# MsgExecuteContract(
			# 	address,
			# 	contract_address,
			# 	execute_msg
			# ),
			MsgSwap(
				address,
				Coin('uusd', 1000000),
				'uluna'
			),
			MsgExecuteContract(
				address,
				terraswap_address,
				execute_terraswap_msg,
				[Coin.from_str(min_receive + "uluna")]
			)
		],
	    memo="test transaction!",
	    fee=StdFee(400000, "120000uusd")
	)
	estimated_fee = terra.tx.estimate_fee(tx)
	tx = wallet.create_and_sign_tx(
	    msgs=[
			# MsgExecuteContract(
			# 	address,
			# 	contract_address,
			# 	execute_msg
			# ),
			MsgSwap(
				address,
				Coin('uusd', 1000000),
				'uluna'
			),
			MsgExecuteContract(
				address,
				terraswap_address,
				execute_terraswap_msg,
				[Coin.from_str(min_receive + "uluna")]
			)
		],
	    memo="test transaction!",
	    fee=StdFee(estimated_fee.gas, "40000uusd")
	)
#	tx.fee.gas = estimated_fee.gas
	# tx.fee.amount = Coins([Coin(denom="uusd", amount=tx.fee.gas)])
	print(f"estimated fee: {estimated_fee}")
	print('===')
	result = terra.tx.broadcast(tx)
	print(result)
        

if __name__ == "__main__":
    main()
