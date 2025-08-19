#!/bin/sh

if [ -f "/run/secrets/db-password" ]; then
    export DB_PASSWORD=$(cat /run/secrets/db-password)
fi

if [ -f "/run/secrets/hadith-bot-db-password" ]; then
    export DB_PASSWORD=$(cat /run/secrets/hadith-bot-db-password)
fi

if [ -f "/run/secrets/telegram-bot-token" ]; then
    export TELOXIDE_TOKEN=$(cat /run/secrets/telegram-bot-token)
fi

if [ -f "/run/secrets/hadith-bot-telegram-token" ]; then
    export TELOXIDE_TOKEN=$(cat /run/secrets/hadith-bot-telegram-token)
fi

exec "$@"