# IBKR Bot

A simple trading bot that executes a momentum strategy using Interactive Brokers.

## Strategy

The strategy is straightforward:
1. At ASX market open, check if SPY (S&P 500 ETF) closed higher than it closed the previous day
2. If SPY was up, buy ASX shares at open
3. Hold until ASX market close, then sell

The hypothesis is that US market momentum carries over to the Australian market the following day.

## Goals

- **Simplicity**: The strategy is simple, and the code should reflect that
- **Robustness**: The bot should keep running for a long time unless a real error requires human intervention

## Architecture

### Files

- **main.rs**: Core trading loop with three phases per day
- **helpers.rs**: IBKR client utilities - connection management, contract definitions, balance/position queries, order status handling
- **types.rs**: `ParsedTradingHours` struct that parses IBKR's trading hours format to determine next open/close times
- **constants.rs**: ASX timezone constant (`Australia/NSW`)
- **tests.rs**: Integration tests that verify assumptions about IBKR API behavior (requires IB Gateway running)

### Main Loop Flow

1. **Sleep until order placement time**: Fetch ASX contract details, parse trading hours, calculate next open, sleep until open + 1 minute
2. **Evaluate and place buy order**: Fetch SPY's last two daily closes, check if most recent > previous. If yes, place market buy order for ASX shares
3. **Sleep until close**: Wait until close - 1 minute
4. **Place sell order**: Sell all shares acquired in step 2

## Key Design Decisions

### Fresh IBKR connections each time

IB Gateway requires re-authentication every 24 hours. The `ibapi` library doesn't handle this gracefully, so instead of managing reconnection logic, we create a fresh client connection for each operation. The 5-second sleep after connection allows the client to stabilize. Authentication happens outside of trading hours, so this doesn't interfere with order placement.

### Hardcoded 10AM-4PM ASX trading hours

IBKR's contract details return "trading hours" and "liquid hours", but not the regular session hours we actually care about. The times in the IBKR response aren't reliable for our purposes, so we hardcode the actual ASX regular trading hours (10:00 AM - 4:00 PM Sydney time).

### Error handling philosophy

The bot has two distinct error handling modes:

- **Before owning shares**: Errors log a warning and `continue` to the next iteration. The bot will retry on the next trading session. This handles transient issues like network problems or API hiccups.

- **After buying shares**: Errors return `Err` and crash the bot. If we're holding a position and something goes wrong with selling, we want human intervention rather than silent failure. The position must be managed.

### User-specified starting balance

On startup, the user enters their desired trading balance. This allows running the strategy with a subset of account funds rather than the entire AUD balance. The bot tracks this balance in memory and only trades with these allocated funds.

If the bot crashes and restarts, the user re-enters their starting balance. The assumption is that you're running this with dedicated/isolated funds.

### Market orders with timing offsets

ASX doesn't support MOO (Market On Open) or MOC (Market On Close) order types. We approximate this by:
- Buying 1 minute after market open - liquidity is established by then
- Selling 1 minute before market close - ensures the order fills before the session ends

Market orders are used because the strategy specifically wants the open/close prices, not limit fills.

### BHP as test proxy for VAS

The intended production instrument is VAS (Vanguard Australian Shares ETF, contract ID 300). However, due to account restrictions preventing purchase of Australian ETFs during testing, BHP (contract ID 4036812) is used as a test proxy since it trades on the same exchange with similar liquidity characteristics.

### Single client ID

The client ID in `get_client()` is hardcoded to 1. This means only one instance of the bot can connect to IB Gateway at a time. This is intentional - all operations are sequential and there's no need for multiple connections.

## IBKR Configuration

- **Gateway address**: `127.0.0.1` with port selected at startup:
  - Port 4002: Simulated/paper trading
  - Port 4001: Real/live trading
- **Client ID**: 1
- **Contract IDs**:
  - SPY: 756733 (on ARCA exchange, USD)
  - BHP: 4036812 (on SMART exchange, AUD) - test instrument
  - VAS: 300 (on SMART exchange, AUD) - production instrument

## IBKR API Assumptions

### Bar ordering is oldest-first

When fetching historical data, IBKR returns bars ordered **oldest first** - `bars[0]` is the oldest, `bars[last]` is the most recent. This is verified by the `verify_spy_bar_ordering` test.

This matters because:
- When calculating SPY change: `bars[1].close - bars[0].close` (most recent minus previous)
- When getting the last close price: use `.last()` not `.first()`

Getting this wrong would invert the entire trading strategy.

### Historical data during trading hours returns fewer bars

When requesting historical daily bars during trading hours, IBKR only returns **completed** days. The current (incomplete) trading day is not included.

For example, requesting `2.days()` during ASX trading hours will only return 1 bar (yesterday's completed session). This is why we request 2 days when we only need 1 - it ensures we get at least one completed bar regardless of when the request is made.

## Testing

Tests require IB Gateway to be running. Run with:

```bash
cargo test -- --nocapture
```

The `--nocapture` flag shows printed output which is useful for seeing actual values returned by IBKR.

Current tests:
- `verify_spy_bar_ordering`: Confirms bars are ordered oldest-first. This is a critical assumption - if IBKR ever changes this behavior, the test will fail and alert us before we trade with inverted logic.

## Dependencies

- `ibapi`: IBKR API client
- `chrono` / `chrono-tz`: Date/time handling with timezone support
- `tokio`: Async runtime
- `eyre`: Error handling
- `tracing`: Logging
- `dialoguer`: CLI prompts for user input
