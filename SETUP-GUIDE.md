# How to Download and Run the Trading Bot

This guide continues from the AWS Guide. Make sure your server is running first!

---

## Step 1: Connect to Your Server

Open Terminal and connect:

```bash
ssh -i ~/.ssh/my-key.pem ubuntu@YOUR-IP-ADDRESS
```

---

## Step 2: Install Rust

Rust is the programming language the bot is written in.

1. Run this command:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. When it asks what to do, type `1` and press Enter

3. When it's done, run this to activate Rust:

```bash
source ~/.cargo/env
```

---

## Step 3: Install Some Extra Tools

Run these two commands:

```bash
sudo apt update
```

```bash
sudo apt install -y build-essential pkg-config libssl-dev
```

---

## Step 4: Install Docker

Docker lets us run IB Gateway (the thing that connects to Interactive Brokers).

```bash
sudo apt install -y docker.io docker-compose-v2
```

Then let your user run Docker:

```bash
sudo usermod -aG docker $USER
```

**Important:** Log out and back in for this to take effect:

```bash
exit
```

Then reconnect:

```bash
ssh -i ~/.ssh/my-key.pem ubuntu@YOUR-IP-ADDRESS
```

---

## Step 5: Download IB Gateway

```bash
git clone https://github.com/YOUR-USERNAME/ib-gateway-docker.git
```

Go into the folder:

```bash
cd ib-gateway-docker
```

---

## Step 6: Set Up Your IBKR Login

Open the settings file:

```bash
nano .env
```

Find these lines and update them:

```
TWS_USERID=myTwsAccountName
TWS_PASSWORD=myTwsPassword
```

Change these to your actual Interactive Brokers username and password.

Also find this line:

```
TRADING_MODE=paper
```

- Leave it as `paper` to practice with fake money (recommended to start!)
- Change it to `live` when you're ready to trade with real money

To save: press `Ctrl+X`, then `Y`, then `Enter`

---

## Step 7: Start IB Gateway

```bash
docker compose up -d
```

This starts IB Gateway in the background. It will automatically log in to your IBKR account.

To check if it's running:

```bash
docker compose ps
```

You should see it says "running".

---

## Step 8: Download the Bot

Go back to your home folder:

```bash
cd ~
```

Download the bot:

```bash
git clone https://github.com/YOUR-USERNAME/ibkr-bot.git
```

Go into the folder:

```bash
cd ibkr-bot
```

---

## Step 9: Install tmux

tmux lets the bot keep running even after you disconnect from the server.

```bash
sudo apt install -y tmux
```

---

## Step 10: Start a tmux Session

```bash
tmux new -s bot
```

Your terminal will look a bit different — that's normal! You're now inside tmux.

---

## Step 11: Run the Bot

```bash
cargo run --release
```

The first time, this takes a few minutes to build everything. Just wait for it to finish.

It will then ask you a couple questions:
1. Which port to use (pick paper trading to test)
2. How much money to trade with

---

## You're Running!

The bot will now wait for the next trading day and execute the strategy.

---

## Disconnecting Without Stopping the Bot

Press `Ctrl+B` then `D` to leave the bot running in the background.

You can now close your terminal or disconnect from SSH — the bot keeps running!

---

## Checking on the Bot Later

1. Connect to your server again:

```bash
ssh -i ~/.ssh/my-key.pem ubuntu@YOUR-IP-ADDRESS
```

2. Go back into the tmux session:

```bash
tmux attach -t bot
```

---

## Updating to a New Version

When there's an update:

```bash
cd ~/ibkr-bot
git pull
cargo run --release
```

---

## If Something Goes Wrong

**Build fails?**
- Make sure you ran the Step 3 commands

**Bot says "connection refused"?**
- IB Gateway needs to be running — check with `cd ~/ib-gateway-docker && docker compose ps`

**Forgot what folder you're in?**
- Run `cd ~/ibkr-bot` to get back to the bot folder
