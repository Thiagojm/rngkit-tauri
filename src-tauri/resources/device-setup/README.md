# RngKit device setup kit

Offline support files for BitBabbler (`0403:7840`) and TrueRNG3 (`04d8:f5fe`).
RngKit never launches these tools, scripts, or installers. Open the files
yourself. Permission or driver setup cannot certify hardware I/O.

Linux hardware collection is not yet accepted for this app.

## Windows / BitBabbler

1. Plug in only the BitBabbler.
2. Run `windows/bitbabbler/zadig-2.9.exe`.
3. Enable Options → List All Devices and select USB ID `0403:7840`.
4. Bind **WinUSB** to that device only. Do not replace an arbitrary FTDI driver.
5. Unplug and reconnect, then use Refresh sources in Collect.

## Windows / TrueRNG3

On Windows 7 or later the device often uses the built-in CDC / `usbser`
driver. If Windows does not bind it:

1. Plug in the TrueRNG3 (`04d8:f5fe` only; not TrueRNGpro).
2. Right-click `windows/truerng3/TrueRNG.inf` and install it. Keep the paired
   `truerng.cat` in the same folder.
3. Do not bind WinUSB to this device.
4. Unplug and reconnect, then use Refresh sources in Collect.

## Ubuntu or Debian

Install `libusb-1.0-0` for BitBabbler if it is missing. TrueRNG3 uses udev
serial access; RngKit sets serial parameters.

From `linux/`:

```text
./setup-rng-devices.sh --check --device bitbabbler --user YOUR_USER
sudo ./setup-rng-devices.sh --device bitbabbler --user YOUR_USER
```

Replace `bitbabbler` with `truerng3` or `both` as needed. `--help` and
`--check` do not require root. Apply does.

Then log out and back in, reconnect the selected device, Refresh sources, and
run a short Start/Stop collection.

Rollback only what the helper reported: remove installed
`/etc/udev/rules.d/60-rngkit-*.rules` files, drop the user from group `rngkit`
if the helper added that membership, and delete group `rngkit` only if the
helper created it and you want it gone. Leave pre-existing groups, members, and
administrator rules in place.

## Notices

Third-party licenses, hashes, and source offers are in `SOURCES.md` and
`THIRD-PARTY-NOTICES.md`.
