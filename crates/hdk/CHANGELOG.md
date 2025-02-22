---
unreleasable: false
default_unreleasable: true
---
# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/). This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/holochain/holochain/compare/hdk-v0.0.100...HEAD)

## 0.0.104

## 0.0.103

### Changed

- hdk: `sys_time` returns `Timestamp` instead of `Duration`

### Added

- hdk: Added `accept_countersigning_preflight_request`

- hdk: Added `session_times_from_millis`

- hdk: Now supports creating and updating countersigned entries

- hdk: Now supports deserializing countersigned entries in app entry `try_from`

- hdk: implements multi-call for:
  
  - `remote_call`
  - `call`
  - `get`
  - `get_details`
  - `get_links`
  - `get_link_details`
  
  We strictly only needed `remote_call` for countersigning, but feedback from the community was that having to sequentially loop over these common HDK functions is a pain point, so we enabled all of them to be async over a vector of inputs.

## 0.0.102

### Changed

- hdk: fixed wrong order of recipient and sender in `x_25519_x_salsa20_poly1305_decrypt`

## 0.0.101

### Changed

- Added `HdkT` trait to support mocking the host and native rust unit tests

### Added

- Added `sign_ephemeral` and `sign_ephemeral_raw`

## [0.0.100](https://github.com/holochain/holochain/compare/hdk-v0.0.100-alpha1..hdk-v0.0.100)

### Changed

- hdk: fixup the autogenerated hdk documentation.

## 0.0.100-alpha.1

### Added

- holochain 0.0.100 (RSM) compatibility
- Extensive doc comments
