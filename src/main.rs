#![warn(unused_crate_dependencies)]

mod constants;
mod helpers;
#[cfg(test)]
mod tests;
mod types;

use std::time::Duration as StdDuration;

use chrono::{Duration, Utc};
use chrono_tz::Tz;
use dialoguer::{Input, Select};
use eyre::{Result, eyre};
use ibapi::{market_data::historical::Bar, prelude::*};
use tokio::time::sleep;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use crate::{
    constants::ASX_TZ,
    helpers::{asx, await_order_filled, get_balance, get_client, get_position, spy},
    types::ParsedTradingHours,
};

async fn place_buy_order(client: &Client, contract: &Contract, amount: f64) -> Result<(f64, f64)> {
    if !amount.is_sign_positive() {
        return Err(eyre!("cannot buy negative AUD worth of shares"));
    }

    // Ensure we have sufficient balance for the trade
    let aud_balance = get_balance(client, "CashBalance", "AUD").await?;
    if aud_balance < amount {
        return Err(eyre!(
            "account balance {aud_balance} less than buy amount {amount}"
        ));
    }

    let historical_data = client
        .historical_data(
            contract,
            None,
            2.days(),
            HistoricalBarSize::Day,
            Some(HistoricalWhatToShow::Trades),
            TradingHours::Regular,
        )
        .await
        .map_err(|e| eyre!("failed to fetch historical data: {e}"))?;

    // Bars are ordered oldest first, so last() gives the most recent close
    let last_close = historical_data
        .bars
        .last()
        .ok_or(eyre!("no candles returned"))?
        .close;

    let max_shares = (amount / last_close).floor();

    let order = order_builder::market_order(Action::Buy, max_shares);
    let order_id = client.next_order_id();

    let sub = client
        .place_order(order_id, contract, &order)
        .await
        .map_err(|e| eyre!("failed to place order: {e}"))?;

    await_order_filled(sub).await?;

    let aud_balance_after = get_balance(client, "CashBalance", "AUD").await?;
    let change = aud_balance - aud_balance_after;

    if !change.is_sign_positive() {
        return Err(eyre!("aud spent is negative: {change}"));
    }

    Ok((change, max_shares))
}

async fn place_sell_order(client: &Client, contract: &Contract, amount: f64) -> Result<f64> {
    if !amount.is_sign_positive() {
        return Err(eyre!("cannot buy a negative amount of shares ({amount})"));
    }

    let asx_balance = get_position(client, contract.contract_id).await?;
    if amount > asx_balance {
        return Err(eyre!(
            "asx shares {asx_balance} less than sell amount {amount}"
        ));
    }
    let aud_balance_before = get_balance(client, "CashBalance", "AUD").await?;

    let order = order_builder::market_order(Action::Sell, amount);
    let order_id = client.next_order_id();

    let sub = client
        .place_order(order_id, contract, &order)
        .await
        .map_err(|e| eyre!("failed to place order: {e}"))?;

    await_order_filled(sub).await?;

    let aud_balance_after = get_balance(client, "CashBalance", "AUD").await?;
    let change = aud_balance_after - aud_balance_before;

    if !change.is_sign_positive() {
        return Err(eyre!("aud received is negative: {change}"));
    }

    Ok(change)
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("ibkr_bot=info,ibapi=error"))
        .try_init();

    let spy = spy();
    let asx = asx();

    // Define asx timezone
    let asx_tz: Tz = ASX_TZ.parse().unwrap();

    let trading_mode = Select::new()
        .with_prompt("Select trading mode")
        .items(&["Simulated (paper trading)", "Real (live trading)"])
        .default(0)
        .interact()?;

    let port: u16 = match trading_mode {
        0 => 4002, // Paper trading
        1 => 4001, // Live trading
        _ => unreachable!(),
    };

    let starting_balance: u64 = Input::new()
        .with_prompt("Enter starting balance (AUD)")
        .interact()?;

    let mut aud_balance = starting_balance as f64;

    // Ensure we have enough AUD for the starting balance
    {
        let client = get_client(port).await?;
        sleep(StdDuration::from_secs(5)).await;
        let real_balance = get_balance(&client, "CashBalance", "AUD").await?;
        if real_balance < aud_balance {
            return Err(eyre!(
                "account balance {real_balance} less than starting balance {starting_balance}"
            ));
        } else {
            tracing::info!(
                "starting bot with balance {aud_balance}. ibkr account balance is {real_balance}"
            );
        }
        drop(client);
    }

    loop {
        sleep(std::time::Duration::from_secs(5)).await;

        // Step 1: Sleep until order placement time
        let next_close_utc = {
            // Cancel any open orders
            let client = get_client(port).await?;
            sleep(StdDuration::from_secs(5)).await;
            match client.global_cancel().await {
                Ok(_) => info!("cancelled open orders"),
                Err(e) => {
                    warn!("failed to cancel open orders: {e}");
                    continue;
                }
            }

            // Fetch ASX details
            let asx_details = match client.contract_details(&asx).await {
                Ok(d) => match d.first() {
                    Some(details) => details.clone(),
                    None => {
                        warn!("ASX details empty");
                        continue;
                    }
                },
                Err(e) => {
                    warn!("failed to fetch ASX details: {e}");
                    continue;
                }
            };

            // Fetch next session
            let asx_th = ParsedTradingHours::parse(asx_tz, asx_details.trading_hours.clone())?;
            let next_session = match asx_th.next_session() {
                Some(session) => session,
                None => {
                    warn!("ASX: no next open returned");
                    continue;
                }
            };
            let next_open_utc = next_session.open_local.to_utc();
            tracing::info!(
                "ASX next open is at {} Australian / {next_open_utc} UTC",
                next_session.open_local
            );

            // Calculate sleep
            let order_placement_time = next_open_utc + Duration::minutes(1);
            let utc_duration = order_placement_time.signed_duration_since(Utc::now());
            let sleep_duration = if utc_duration.num_seconds() > 0 {
                utc_duration.to_std()?
            } else {
                warn!("next open has already passed");
                continue;
            };

            // Sleep until order placement time
            drop(client);
            tracing::info!("Waiting {} hours for open", utc_duration.num_hours());
            sleep(sleep_duration).await;

            // Return next close
            next_session.close_local.to_utc()
        };

        // Step 2: Place order if requirements are met and wait for sell order placement
        let asx_balance = {
            // Get SPY price change
            let client = get_client(port).await?;
            sleep(std::time::Duration::from_secs(5)).await;
            let spy_change = match client
                .historical_data(
                    &spy,
                    None,
                    2.days(),
                    HistoricalBarSize::Day,
                    Some(HistoricalWhatToShow::Trades),
                    TradingHours::Regular,
                )
                .await
            {
                Ok(hd) => {
                    let bars: [Bar; 2] = match hd.bars.try_into() {
                        Ok(bars) => bars,
                        Err(e) => {
                            warn!("SPY returned wrong number of bars: {e:?}");
                            continue;
                        }
                    };
                    // Bars are ordered oldest first: bars[0] is previous day, bars[1] is most recent
                    bars[1].close - bars[0].close
                }
                Err(e) => {
                    warn!("failed to fetch SPY historical price: {e}");
                    continue;
                }
            };

            // Evaluate order placement and place buy order
            let asx_balance = match spy_change.is_sign_positive() {
                true => match place_buy_order(&client, &asx, aud_balance).await {
                    Ok((aud, shares)) => {
                        info!("buy order filled. {aud} AUD for {shares} shares");
                        aud_balance -= aud;
                        shares
                    }
                    Err(e) => {
                        warn!("buy order failed: {e}");
                        continue;
                    }
                },
                false => {
                    info!("SPY was not up yesterday ({spy_change} USD). Skipping order placement");
                    continue;
                }
            };

            // After this point the order has been placed. If there are issues we exit with an error.

            let sell_time = next_close_utc - Duration::minutes(1);
            let utc_duration = sell_time.signed_duration_since(Utc::now());
            let sleep_duration = if utc_duration.num_seconds() > 0 {
                utc_duration.to_std()?
            } else {
                return Err(eyre!("next close has already passed"));
            };

            // Sleep until order placement time
            drop(client);
            tracing::info!("Waiting {} hours for close", utc_duration.num_hours());
            sleep(sleep_duration).await;

            // Return the amount of shares we own
            asx_balance
        };

        // Step 3: Place sell order
        let client = get_client(port).await?;
        sleep(std::time::Duration::from_secs(5)).await;
        match place_sell_order(&client, &asx, asx_balance).await {
            Ok(aud) => {
                info!("sell order filled. {asx_balance} shares for {aud} AUD");
                aud_balance += aud;
            }
            Err(e) => return Err(eyre!("sell order failed: {e}")),
        }
    }
}
