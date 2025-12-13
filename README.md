# IBKR Trading Bot

A simple trading bot that runs a momentum strategy using Interactive Brokers.

## How It Works

1. At ASX market open, checks if SPY (S&P 500) closed higher than the previous day
2. If SPY was up, buys ASX shares at open
3. Holds until market close, then sells

## Getting Started

Follow these two guides in order:

1. **[AWS Guide](AWS-GUIDE.md)** — Set up a server in the cloud to run the bot
2. **[Setup Guide](SETUP-GUIDE.md)** — Install everything and run the bot

For a smoother experience, paste each guide into ChatGPT and ask it to walk you through step by step. If you get stuck, feel free to reach out

## Requirements

- An [Interactive Brokers](https://www.interactivebrokers.com) account
- An [AWS](https://aws.amazon.com) account (free tier works fine)
