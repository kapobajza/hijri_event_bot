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

1. Clone the repository:
   ```bash
   git clone
   ```
2. Navigate to the project directory:
   ```bash
   cd hijri_event_bot
   ```
3. [Create a Telegram bot](https://core.telegram.org/bots/tutorial) and get your bot token.
4. Make sure you have a PostgreSQL database set up. The default connection string is `postgres://postgres:postgres@localhost:5433/hijri_event_bot`. But you can provide your own values to override it, by setting the env variables. for example:
   ```bash
    export DB_USER="postgres"
    export DB_PASSWORD="postgres"
    export DB_HOST="localhost"
    export DB_PORT="5433"
    export DB_NAME="hijri_event_bot"
   ```
5. Set the `TELEGRAM_BOT_TOKEN` environment variable with your bot token:
   ```bash
   export TELEGRAM_BOT_TOKEN="your_bot_token_here"
   ```
6. Run the bot:
   ```bash
   cargo run
   ```
7. Interact with the bot on Telegram by sending commands like `/help` or `/date`.
