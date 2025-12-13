#[cfg(test)]
mod tests {
    use crate::helpers::{get_client, spy};
    use ibapi::prelude::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn verify_spy_bar_ordering() {
        // This test verifies our assumption about IBKR bar ordering.
        // Our code assumes bars are ordered OLDEST FIRST (bars[0] is oldest, bars[1] is most recent).
        // If this assumption is violated, our strategy would be inverted (buying when SPY is down).

        // Tests use paper trading port
        let client = get_client(4002).await.expect("failed to connect to IB Gateway");
        sleep(Duration::from_secs(5)).await;

        let spy = spy();

        // Request more days to ensure we get at least 2 completed trading days
        let historical_data = client
            .historical_data(
                &spy,
                None,
                5.days(),
                HistoricalBarSize::Day,
                Some(HistoricalWhatToShow::Trades),
                TradingHours::Regular,
            )
            .await
            .expect("failed to fetch SPY historical data");

        println!("Number of bars returned: {}", historical_data.bars.len());
        println!();

        for (i, bar) in historical_data.bars.iter().enumerate() {
            println!("Bar {}: date={}, close={}", i, bar.date, bar.close);
        }

        assert!(
            historical_data.bars.len() >= 2,
            "expected at least 2 bars, got {}",
            historical_data.bars.len()
        );

        let first_bar = &historical_data.bars[0];
        let last_bar = &historical_data.bars[historical_data.bars.len() - 1];

        // Assert that bars are ordered oldest first
        assert!(
            first_bar.date < last_bar.date,
            "Expected bars to be ordered OLDEST FIRST, but bars[0] ({}) is not older than bars[last] ({}). \
            This would invert our trading strategy!",
            first_bar.date,
            last_bar.date
        );

        println!();
        println!("Confirmed: bars are ordered oldest first (bars[0] is oldest)");
    }
}
