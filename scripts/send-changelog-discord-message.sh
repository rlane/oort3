#!/bin/bash -eu
. scratch/secrets.sh
awk -vN=2 'n<N;/###/{++n}' CHANGELOG.md | head -n-1 | sed -e 's/^###/Released version/' |
  ./scripts/send-discord-message.py $DISCORD_CHANGELOG_WEBHOOK
