#!/bin/sh
# rpm passes the number of remaining installs: 0 = uninstall, >0 = upgrade.
# Remove the opt-in auto-paste rule QuickDesk may have installed only on uninstall.
if [ "$1" = "0" ] && [ -f /etc/udev/rules.d/70-quickdesk-uinput.rules ]; then
  rm -f /etc/udev/rules.d/70-quickdesk-uinput.rules
  udevadm control --reload-rules 2>/dev/null || true
  setfacl -b /dev/uinput 2>/dev/null || true
fi
exit 0
