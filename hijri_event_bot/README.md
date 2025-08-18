# Hijri even bot

This is a Telegram bot which provides notifications based on the lunar calendar, specifically for the Hijri calendar. It sends updates and notifications about important islamic dates and events.

## Features

- Provides the current Hijri date.
- Sends notifications for the 12th day of each lunar month, which is significant for fasting.
- Currently supports the Bosnian language, but can be extended to other languages.
- Uses a scheduler to manage notifications and events.
- Uses a PostgreSQL database for storing user data and event information.
- Uses the `teloxide` library for Telegram bot interactions.
- Uses `sqlx` for database interactions.
- Uses `chrono` for date and time handling.

## Running the Bot

> [!NOTE]
> Make sure that you have Docker and Docker Compose installed on your machine.

1. Clone the repository:
   ```bash
   git clone
   ```
2. Navigate to the project directory:
   ```bash
   cd hijri_event_bot
   ```
3. [Create a Telegram bot](https://core.telegram.org/bots/tutorial) and get your bot token.
4. Run the app:
   ```bash
   TELEGRAM_BOT_TOKEN=the_token_you_got_from_step_3 docker compose -f compose.yml -f compose.local.yml up
   ```
5. The default connection string is `postgres://postgres:postgres@localhost:5433/hijri_event_bot`. But you can provide your own values to override it, by overriding the env. For example:
   ```bash
    DB_USER=my_user DB_NAME=my_db TELEGRAM_BOT_TOKEN=your_telegram_token docker compose -f compose.yml -f compose.local.yml up
   ```
6. Interact with the bot on Telegram by sending commands like `/help` or `/date`.
