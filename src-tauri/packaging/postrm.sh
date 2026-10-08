#!/bin/sh
# Remove the opt-in auto-paste rule QuickDesk may have installed (Settings → Auto-paste).
case "$1" in
  remove|purge)
    if [ -f /etc/udev/rules.d/70-quickdesk-uinput.rules ]; then
      rm -f /etc/udev/rules.d/70-quickdesk-uinput.rules
      udevadm control --reload-rules 2>/dev/null || true
      setfacl -b /dev/uinput 2>/dev/null || true
    fi
    ;;
esac
exit 0
