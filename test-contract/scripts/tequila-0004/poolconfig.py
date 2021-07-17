from attr import dataclass


@dataclass
class PoolConfig:
    contract_address: str = "",
    denom: str = ""


TERRASWAP_UST_CONFIG = PoolConfig(contract_address='terra156v8s539wtz0sjpn8y8a8lfg8fhmwa7fy22aff', denom='uusd')
TERRASWAP_KRT_CONFIG = PoolConfig(contract_address='terra1rfzwcdhhu502xws6r5pxw4hx8c6vms772d6vyu', denom='ukrw')