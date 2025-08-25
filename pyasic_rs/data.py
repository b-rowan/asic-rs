from datetime import timedelta
from ipaddress import IPv4Address
from typing import Annotated

from pydantic import BaseModel, ConfigDict, BeforeValidator, field_serializer


class MinerHardware(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    chips: int | None
    fans: int | None
    boards: int | None

class DeviceInfo(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    make: Annotated[str, BeforeValidator(str)]
    model: Annotated[str, BeforeValidator(str)]
    hardware: MinerHardware
    firmware: Annotated[str, BeforeValidator(str)]
    algo: Annotated[str, BeforeValidator(str)]

class HashRate(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    value: float
    unit: Annotated[int, BeforeValidator(int)]
    algo: str

    def __float__(self):
        return self.value

class ChipData(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    position: int
    hashrate: HashRate | None
    temperature: float | None
    voltage: float | None
    frequency: float | None
    tuned: bool | None
    working: bool | None


class BoardData(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    position: int
    hashrate: HashRate | None
    expected_hashrate: HashRate | None
    board_temperature: float | None
    intake_temperature: float | None
    outlet_temperature: float | None
    expected_chips: int | None
    working_chips: int | None
    serial_number: str | None
    chips: list[ChipData]
    voltage: float | None
    frequency: float | None
    tuned: bool | None
    active: bool | None


class FanData(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    position: int
    rpm: float | None

class PoolData(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    position: int | None
    url: Annotated[str, BeforeValidator(str)] | None
    accepted_shares: int | None
    rejected_shares: int | None
    active: bool | None
    alive: bool | None
    user: str | None


class MinerMessage(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    timestamp:  int
    code: int
    message: str
    severity: Annotated[str, BeforeValidator(str)]

class MinerData(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    schema_version: str
    timestamp: int
    ip: IPv4Address
    mac: str
    device_info: DeviceInfo
    serial_number: str | None
    hostname: str | None
    api_version: str | None
    firmware_version: str | None
    expected_hashboards: int | None
    hashboards: list[BoardData]
    hashrate: HashRate | None
    expected_hashrate: HashRate | None
    expected_chips: int | None
    total_chips: int | None
    expected_fans: int | None
    fans: list[FanData]
    psu_fans: list[FanData]
    average_temperature: float | None
    fluid_temperature: float | None
    wattage: int | None
    wattage_limit: int | None
    efficiency: float | None
    light_flashing: bool | None
    messages: list[MinerMessage]
    uptime: timedelta | None
    is_mining: bool
    pools: list[PoolData]

    @field_serializer("uptime")
    def serialize_uptime(self, uptime: timedelta, _info) -> float:
        return uptime.total_seconds()
