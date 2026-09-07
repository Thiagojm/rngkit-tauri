# Device-setup source record

Verification date: 2026-09-07. Files were hashed and, where applicable,
Authenticode-checked without executing installers.

## Zadig 2.9 (`windows/bitbabbler/zadig-2.9.exe`)

- Upstream page: https://zadig.akeo.ie/
- Downloaded from: https://github.com/pbatard/libwdi/releases/download/v1.5.1/zadig-2.9.exe
- libwdi tag: `v1.5.1` (commit `9b23b82a2dd1cbffc16d46c212f92c6bf8c0c602`)
- File version: 2.9.788
- Size: 5334088 bytes
- SHA-256: `4ecaa95df3da3621486a043aef8b3050b8bafe7c901402871e816229ef82039b`
- Authenticode: valid, signer `CN=Akeo Consulting, O=Akeo Consulting, C=IE`.
  Signing certificate expired 2024-09-28; the file is timestamped and Windows
  reported the signature as verified. The copy from `zadig.akeo.ie/downloads/`
  was not retrieved (404 at verification time); GitHub release v1.5.1 matched
  the advertised 2.9 binary.
- License: Zadig GPL-3.0; libwdi LGPL-3.0-or-later. Texts: `COPYING`,
  `COPYING-LGPL`. Source offer: `SOURCE-OFFER.txt`.
- Redistribution: permitted under those licenses with notices and corresponding
  source.

## TrueRNG Windows CDC/usbser (excluded)

The previously inspected INF/CAT are excluded from the kit: redistribution
permission has not been established. The records below identify the inspected
upstream package, not bundled files. Inclusion remains blocked pending documented
permission. Users may obtain the package directly from the manufacturer source.

- Upstream: ubld.it points to https://github.com/euler357/TrueRNG
  (`Windows_Drivers/TrueRNG-Windows-Driver1.zip`).
- Zip SHA-256: `3e33624df1f3ec113961af3d19e3786b4f357ef29461cc52f1f3b41c24dde3cd`
- Extracted:
  - `TrueRNG.inf` SHA-256 `d26064764b1dace833349a33fb0f2876b39e90a2f4b97e04cbbd504644cf4632`
  - `truerng.cat` SHA-256 `d6a95af7e0b539820388e4b6a84b4c197a57a6b610ba67cbb3f746b36439bd41`
- INF device ID: `USB\VID_04D8&PID_F5FE` only. `DriverVer=02/03/2016,1.0.0.1`.
  CatalogFile `TrueRNG.cat` (on-disk name `truerng.cat`). No TrueRNGpro IDs.
- Authenticode on `truerng.cat`: valid, signer `CN=Chris K Cockrum`. Signing
  certificate expired 2019-01-13; timestamped; Windows reported the signature
  as verified.
- License: the GitHub repository publishes no LICENSE file. The INF names
  ubld.it / Chris K Cockrum and is the manufacturer signed CDC/usbser package
  for this ID. Manufacturer provenance and a valid signature do not establish
  redistribution permission.
- Vendor note in the zip README: the INF is not strictly required on Windows 7
  or later because native USB CDC often already maps the device.

## Linux helper and rules

- Authored for this app: `linux/setup-rng-devices.sh`,
  `linux/60-rngkit-bitbabbler.rules`, `linux/60-rngkit-truerng3.rules`.
- Runtime library need from the pinned `rngkit-core` revision
  `3dc969d983ffa7c981536c46d19afa223f0c490b`: BitBabbler via `rusb`/`libusb-1.0`;
  TrueRNG3 via `serialport`/`libudev`. Not PyUSB.
