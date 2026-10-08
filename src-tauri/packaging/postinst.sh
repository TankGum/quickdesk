#!/bin/sh
# Apply the uinput access rule without a reboot.
udevadm control --reload-rules 2>/dev/null || true
udevadm trigger --action=change --sysname-match=uinput 2>/dev/null || true
udevadm settle 2>/dev/null || true
exit 0
