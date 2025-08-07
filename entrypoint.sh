#!/bin/sh

if [ -f "/run/secrets/db-password" ]; then
    export DB_PASSWORD=$(cat /run/secrets/db-password)
fi

if [ -f "/run/secrets/telegram-bot-token" ]; then
    export TELOXIDE_TOKEN=$(cat /run/secrets/telegram-bot-token)
fi

exec "$@"