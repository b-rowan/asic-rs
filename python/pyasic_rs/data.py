from datetime import timedelta
from ipaddress import IPv4Address
from typing import Annotated

from pydantic import (
    BaseModel,
    ConfigDict,
    BeforeValidator,
    TypeAdapter,
    field_serializer,
    model_serializer,
)
from pyasic_rs.asic_rs import HashRate, HashRateUnit  # noqa: F401 — re-exported for callers

_hashrate_adapter: TypeAdapter[HashRate] = TypeAdapter(HashRate)


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


class PoolGroupData(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    name: str
    quota: int
    pools: list[PoolData]


class TuningTargetPower(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    watts: float

    @model_serializer
    def serialize_tuning_target(self):
        return {"type": "power", "value": self.watts}


class TuningTargetHashRate(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    hashrate: HashRate

    @model_serializer
    def serialize_tuning_target(self):
        return {
            "type": "hashrate",
            "value": _hashrate_adapter.dump_python(self.hashrate, mode="json"),
        }


class TuningTargetMiningMode(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    mode: Annotated[str, BeforeValidator(str)]

    @model_serializer
    def serialize_tuning_target(self):
        return {"type": "mode", "value": self.mode}


def _parse_tuning_target(v):
    if isinstance(v, (TuningTargetPower, TuningTargetHashRate, TuningTargetMiningMode)):
        return v

    if isinstance(v, dict):
        if "watts" in v:
            return TuningTargetPower.model_validate(v)
        if "hashrate" in v:
            return TuningTargetHashRate.model_validate(v)
        if "mode" in v:
            return TuningTargetMiningMode.model_validate(v)

        target_type = v.get("type")
        value = v.get("value")
        if target_type == "power" and value is not None:
            return TuningTargetPower(watts=float(value))
        if target_type == "hashrate" and value is not None:
            return TuningTargetHashRate(hashrate=_hashrate_adapter.validate_python(value))
        if target_type == "mode" and value is not None:
            return TuningTargetMiningMode(mode=str(value))

        return v

    variant = type(v).__name__
    if variant == "Power" and hasattr(v, "_0"):
        return TuningTargetPower(watts=float(v._0))
    if variant == "HashRate" and hasattr(v, "_0"):
        return TuningTargetHashRate(hashrate=v._0)
    if variant == "MiningMode" and hasattr(v, "_0"):
        return TuningTargetMiningMode(mode=str(v._0))
    return v


TuningTarget = Annotated[
    TuningTargetPower | TuningTargetHashRate | TuningTargetMiningMode,
    BeforeValidator(_parse_tuning_target),
]


class MinerMessage(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    timestamp: int
    code: int
    message: str
    severity: Annotated[str, BeforeValidator(str)]


class MinerControlBoard(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    known: bool
    name: str

    @model_serializer(mode="plain")
    def serialize_model(self) -> str:
        return self.name

    def __repr__(self):
        if self.known:
            return self.name
        return f"Unknown: {self.name}"


class MinerData(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    schema_version: str
    timestamp: int
    ip: IPv4Address
    mac: str | None
    device_info: DeviceInfo
    control_board_version: MinerControlBoard | None
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
    wattage: float | None
    tuning_target: TuningTarget | None
    efficiency: float | None
    light_flashing: bool | None
    messages: list[MinerMessage]
    uptime: timedelta | None
    is_mining: bool
    pools: list[PoolGroupData]

    @field_serializer("uptime")
    def serialize_uptime(self, uptime: timedelta, _info) -> float | None:
        if uptime is not None:
            return uptime.total_seconds()
        return None
