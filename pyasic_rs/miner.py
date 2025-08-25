import asic_rs

from .data import MinerData


class Miner:
    def __init__(self, inner: asic_rs.Miner | None = None):
        self.inner = inner

    async def get_data(self) -> MinerData:
        data = await self.inner.get_data()
        return MinerData.model_validate(data)
