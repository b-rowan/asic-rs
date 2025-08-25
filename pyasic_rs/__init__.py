import asic_rs

from pyasic_rs.miner import Miner


class MinerFactory:
    def __init__(self, inner: asic_rs.MinerFactory | None = None):
        if inner is None:
            inner = asic_rs.MinerFactory()
        self.inner = inner

    @classmethod
    def with_subnet(cls, subnet: str):
        cls(inner=asic_rs.MinerFactory.with_subnet(subnet))

    async def get_miner(self, ip: str) -> Miner:
        return Miner(await self.inner.get_miner(ip))
