# Changelog

## [1.0.0] - 2024-03-26
The first stable release of the Public API incorporates minor enhancements.

### Added
* Introduced new field `enabled` in the `braiins.bos.v1.DPSConfiguration` that provides info, if DPS is enabled by default or not.
* Introduced new field `enabled` in the `braiins.bos.v1.TunerConstraints` that provides info, if DPS is enabled by default or not.
* Introduced new field `default_mode` in the `braiins.bos.v1.TunerConstraints` that provides info about the default tuner mode.
* Introduced new field `status` in the `braiins.bos.v1.GetMinerDetailsResponse` that provides info about the current miner status.

## [1.0.0-beta.6] - 2024-03-05
Version **1.0.0-beta.6** contains one new feature Network configuration.

### Added
* New `braiins.bos.v1.NetworkService` with `GetNetworkConfiguration` and `SetNetworkConfiguration` methods

## [1.0.0-beta.5] - 2023-12-14
Version **1.0.0-beta.5** contains one small extension.

### Added
* Extension of the `braiins.bos.v1.Platform` enumeration with new value for Zynq.

## [1.0.0-beta.4] - 2023-11-23
Version **1.0.0-beta.4** contains one new feature and one breaking change.

### Added
* We added option to clean tuner profiles by adding `braiins.bos.v1.PerformanceService::RemoveTunedProfiles`
* Introduced a new field `system_uptime_s` in the `braiins.bos.v1.GetMinerDetailsResponse` that replaces `system_uptime`(marked as deprecated) to keep the best practice that a field name should also describe the unit (when applicable).

### Breaking Changes:
* We reverted removing `braiins.bos.v1.MinerModel` enumeration from the previous release because this change was causing troubles to our users.
  Instead of dropping enumeration, we decided to mark it as deprecated and introduce new field `miner_model` for string representation.

## [1.0.0-beta.3] - 2023-11-02
Version **1.0.0-beta.3** contains a few minor improvements.

### Added
* Extension of the `braiins.bos.v1.GetMinerDetailsResponse` message with `bosminer_uptime_s` field that contains bosminer uptime.

### Breaking Changes:
* We removed `braiins.bos.v1.MinerModel` enumeration and changed type of `model` field in `braiins.bos.v1.MinerIdentity` to `string`. Replacing model enumeration with string eliminates the need to release a new version every time we add support for new model,

## [1.0.0-beta.2] - 2023-08-10
Version **1.0.0-beta.2** extends performance management and Pool Group management possibilities. It also introduces an API to get Mining Status.

#### Enhanced Performance Options
We have extended the alternatives for performance tuning in this release.

##### Hashrate Target Introduction
Formerly, users could only configure power target values. Now, we made it possible to define hashrate targets as well.

##### Dynamic Performance Scaling
This release adds the possibility to configure Dynamic Performance Scaling parameters. Users now can fully optimize their operations to their unique requirements using Public API.

##### Augmented Hashboard Administration
We have extended the hashboards management. Public API now lets users activate or deactivate individual hashboards.

#### Evolved Pool Group Management
Previously only single pool group modifications were possible. We have reworked the Public API to allow addition of new pool groups and removal of existing ones.

#### Introduction of Mining Status
An entirely new API makes its debut, providing concise insights into the mining status of individual devices. The mining status allows users to readily differentiate between paused operations and miners requiring attention or possible repair.

### Added
* Introduction of a new `braiins.bos.v1.PerformanceService::SetDefaultHashrateTarget()` method that allows the user to set the currently configured hashrate target value to default one,
* Introduction of a new `braiins.bos.v1.PerformanceService::SetHashrateTarget()` method that allows the user to set the currently configured hashrate target value to a specific one,
* Introduction of a new `braiins.bos.v1.PerformanceService::IncrementHashrateTarget()` method that allows the user to increment the currently configured hashrate target value,
* Introduction of a new `braiins.bos.v1.PerformanceService::DecrementHashrateTarget()` method that allows the user to decrement currently configured hashrate target value,
* Introduction of a new `braiins.bos.v1.MinerService::EnableHashboards()` method that allows user to enable hashboards,
* Introduction of a new `braiins.bos.v1.MinerService::DisableHashboards()` method that allows user to enable hashboards,
* Introduction of a new `braiins.bos.v1.PerformanceService::SetDPS()` method that allows user to configure dynamic performance scaling,
* Introduction of a new `braiins.bos.v1.PerformanceService::SetPerformanceMode()` method that allows the user to set performance mode to one of the modes: manual or tuner,
* Introduction of a new `braiins.bos.v1.PerformanceService::GetActivePerformanceMode()` method that allows the user to read the current performance mode,
* Introduction of a new `braiins.bos.v1.PerformanceService::ListTargetProfiles()` method that allows the user to read the currently used tuner profile,
* Introduction of a new `braiins.bos.v1.MinerService::GetMinerStatus()` method streaming actual miner status.
* Introduction of a new `braiins.bos.v1.PoolService::RemovePoolGroup()` method that allows to remove pools group,
* Introduction of a new `braiins.bos.v1.PoolService::CreatePoolGroup()`,
* Extension of the `braiins.bos.v1.UpdatePoolGroupResponse` message with `group` field that contains updated pool group configuration,
* Extension of the `braiins.bos.v1.TunerConstraints` message with new field `hashrate_target` that contains constraints for hashrate target value,
* Extension of the `braiins.bos.v1.GetMinerStatsResponse` message with new field `power_stats` that contains miner power stats like consumption or efficiency,
* Extension of the `braiins.bos.v1.GetTunerStateResponse` message with new fields:
  * `current_power_target` that contains the current power target,
  * `current_hashrate_target` that contains the current hashrate target,
  * `current_profile` that contains info about the currently used tuner profile,
* Extension of the `braiins.bos.v1.Hashboard` with the new field `enabled` that represents a flag if HB is enabled
* Extension of the `braiins.bos.v1.GetMinerConfigurationResponse` with new fields:
  * `dps` that contain Dynamic Performance Scaling configuration,
  * `hashboard_config` holds information regarding the presently configured hashboard frequency, voltage settings, or the enabled/disabled status of HB,
* Extension of the `braiins.bos.v1.GetConstraintsResponse` message with new fields:
  * `dps_constraints` that contains constraints for Dynamic Performance Scaling configuration,
  * `hashboards_constraints` that contains constraints for hashboards config,
* Extension of the `braiins.bos.v1.Platform` enumeration with new values for CVITEK and Braiins control boards.

### Breaking Changes:
* The service `braiins.v1.bos.TunerService` has been updated to `braiins.v1.bos.PerformanceService` to encompass methods relevant to this service.
* Furthermore, the method `braiins.v1.bos.PerformanceService::SetAbsolutePowerTarget()` has been modified to `braiins.v1.bos.PerformanceService::SetPowerTarget()`, removing the potentially confusing term "Absolute".
* In alignment with the adjusted method name, the request message previously labeled as `braiins.bos.v1.SetAbsolutePowerTargetRequest` has been changed to `braiins.bos.v1.SetPowerTargetRequest`.
* Standardization of modification requests has been enacted, with each request containing a designated `save_action` field located at index 1.
* In anticipation of potential hashboard identifiers incorporating non-numeric characters in the future, adjustments have been made to the hashboards identifier fields, shifting from `uint32` to `string` type.

### Migration Guide:
In order to transition to version **1.0.0-beta.2**, the integration requires implementation of the subsequent adjustments:
* Ensure incorporation of the updated moniker `PerformanceService`. The same applies to usage of the `SetAbsolutePowerTarget` method.
* Verify the seamless functionality of your modification request with the modified index placements within the `save_action` fields.
* Guarantee accurate parsing of the novel string-based hashboard identifiers.

---

## [1.0.0-beta.1] - 2023-06-29
The latest release, version **1.0.0-beta.1**, enhances the authentication functionalities while also introducing two new features: Device Location and Support Archive.

#### Enhanced Authentication
A novel method is unveiled within this iteration, affording users the ability to institute a new password for their miners.

#### Device Location Functionality
The Locate Device feature debuts, offering users the capability to activate or deactivate LEDs on the device. This serves as a visual aid, allowing the client to precisely identify the targeted device.

#### Introduction of Support Archive
A new API has emerged, facilitating streamlined access to a comprehensive support archive. This archive acts as a valuable repository, housing crucial information for troubleshooting and problem resolution.

### Added
* Introduction of a new `braiins.bos.v1.AuthenticationService::SetPassword()` method that allows the user to set a new password on the miner,
* Introduction of a new `braiins.bos.v1.ActionsService::SetLocateDeviceStatus()` and `braiins.bos.v1.ActionsService::GetLocateDeviceStatus()` methods to turn on/off device location function,
* Introduction of a new `braiins.bos.v1.MinerService::GetSupportArchive()` streaming method that allows to download BOS support archive.
* Extension of the `braiins.bos.v1.GetMinerDetailsResponse` message with `sticker_hashrate` field that contains HR declared by miner vendor,

**Important:** This iteration bears resemblance to **1.0.0-alpha.1**, yet it is grounded in a licenses-integrated branch and encompasses a broader array of features.

---

## [1.0.0-beta] - 2023-05-11
The latest release, version 1.0.0-beta, introduces a significant addition: the all-new BOS+ License feature.

### Added
* Introduction of a new `braiins.bos.v1.LicenseService::GetLicenseState()` streaming method to fetch BOS Licence state.

**Important:** This version remained in a private state for a duration, attributed to the testing phase of the novel licenses approach.

---

## [1.0.0-alpha.1] - 2023-06-26
New version 1.0.0-alpha.1 extends authentication options and introduces new features, **Locate Device** and **Support Archive**.

##### Authentication options
We introduce a new method that allows user to set new password on miner.

##### Locate Device
Locate device is a feature where user can turn on/off a LED on the device and client is able to visually detect, which one is it.

##### Support Archive
We introduce a new API that provides simple way how to download a support archive that contains useful information for problem investigation.

### Added
* Introduction of a new `braiins.bos.v1.AuthenticationService::SetPassword()` method that allows user to set a new password on the miner,
* Introduction of new `braiins.bos.v1.ActionsService::SetLocateDeviceStatus()` and `braiins.bos.v1.ActionsService::GetLocateDeviceStatus()` methods to turn on/off device location function,
* Introduction of a new `braiins.bos.v1.MinerService::GetSupportArchive()` streaming method that allows to download BOS support archive.
* Extension of the `braiins.bos.v1.GetMinerDetailsResponse` message with a `sticker_hashrate` field that contains HR declared by miner vendor.

---

## [1.0.0-alpha] - 2023-05-25
The first release for the new Braiins OS+ Public API, which introduces the first batch of features.

##### Actions
With actions from `ActionService`, user can start/stop/restart/pause/resume mining (BOS+). Reboot of the whole miner is supported as well.

##### Configuration
With `ConfigurationService` methods, user can read current miner configuration and configuration constraints.

##### Cooling
With `CoolingService`methods, user can read current state of fans or current temperatures, or configure immersion mode.

##### Miner
With `MinerService` methods, user can read miner HW details, overall miner or hashboards statistics.

##### Pools
With `PoolService` methods, user can read all currently used pool groups, as well as update a specific pool group.

##### Tuner
With `TunerService` methods, user can read current tuner state and configure tuner power target.

##### Other
Overall to use gRPC API, user must be authenticated. For this purpose, `AuthenticationService` with `Login` method is present.

The method in `VersionService` provides information about a particular version of the public Braiins OS+ API.
