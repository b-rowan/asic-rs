from typing import Self

from pyasic_rs.asic_rs import MinerFactory as _rs_MinerFactory
from .miner import Miner


class MinerFactory:
    def __init__(self, *, inner: _rs_MinerFactory = _rs_MinerFactory()):
        self.__inner = inner

    @classmethod
    def from_subnet(cls, subnet: str) -> Self:
        return cls(inner=_rs_MinerFactory.from_subnet(subnet))

    @classmethod
    def from_octets(cls, octet_1: int | str, octet_2: int | str, octet_3: int | str, octet_4: int | str) -> Self:
        return cls(inner=_rs_MinerFactory.from_octets(str(octet_1), str(octet_2), str(octet_3), str(octet_4)))

    async def get_miner(self, ip: str) -> Miner | None:
        base = await self.__inner.get_miner(ip)
        if base is not None:
            return Miner(inner=base)
        return None

    async def scan(self) -> list[Miner]:
        bases = await self.__inner.scan()
        return [Miner(inner=m) for m in filter(lambda x: x is not None, bases)]
