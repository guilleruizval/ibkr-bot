use eyre::{Result, eyre};
use ibapi::{
    Client,
    accounts::types::AccountGroup,
    client::Subscription,
    orders::PlaceOrder,
    prelude::{AccountSummaryResult, Contract, Currency, Exchange, PositionUpdate},
};
use tracing::info;

pub async fn get_client(port: u16) -> Result<Client> {
    let connection_url = format!("127.0.0.1:{port}");
    match Client::connect(&connection_url, 1).await {
        Ok(client) => {
            info!("Connected to IB Gateway on port {port}");
            Ok(client)
        }
        Err(e) => Err(eyre!("Connection to IB Gateway failed: {e}")),
    }
}

pub fn spy() -> Contract {
    Contract {
        contract_id: 756733,
        exchange: Exchange("ARCA".to_string()),
        currency: Currency("USD".to_string()),
        ..Default::default()
    }
}

// ETF to buy in prod
// pub fn asx() -> Contract {
//     Contract {
//         contract_id: 60009472,
//         exchange: Exchange("SMART".to_string()),
//         currency: Currency("AUD".to_string()),
//         ..Default::default()
//     }
// }

// NOTE: This fetches BHP for now
pub fn asx() -> Contract {
    Contract {
        contract_id: 4036812,
        exchange: Exchange("SMART".to_string()),
        currency: Currency("AUD".to_string()),
        ..Default::default()
    }
}

pub async fn get_balance(client: &Client, tag: &str, currency: &str) -> Result<f64> {
    let mut sub = client
        .account_summary(
            &AccountGroup("All".to_string()),
            &[&format!("$LEDGER:{currency}")],
        )
        .await?;

    while let Some(res) = sub.next().await {
        match res {
            Ok(msg) => match msg {
                AccountSummaryResult::End => return Err(eyre!("No balance returned")),
                AccountSummaryResult::Summary(summary) => {
                    if summary.tag == tag && summary.currency == currency {
                        let value: f64 = summary.value.parse()?;
                        return Ok(value);
                    }
                }
            },
            Err(e) => return Err(e.into()),
        }
    }

    Err(eyre!("Balance stream returned None"))
}

pub async fn get_position(client: &Client, contract_id: i32) -> Result<f64> {
    let mut sub = client.positions().await?;

    while let Some(res) = sub.next().await {
        match res {
            Ok(msg) => match msg {
                PositionUpdate::PositionEnd => return Err(eyre!("No position returned")),
                PositionUpdate::Position(pos) => {
                    if pos.contract.contract_id == contract_id {
                        return Ok(pos.position);
                    }
                }
            },
            Err(e) => return Err(e.into()),
        }
    }

    Err(eyre!("Position stream returned None"))
}

pub async fn await_order_filled(mut sub: Subscription<PlaceOrder>) -> Result<()> {
    while let Some(res) = sub.next().await {
        match res {
            Ok(PlaceOrder::OrderStatus(status)) => match status.status.as_str() {
                "Inactive" => tracing::info!("Order Inactive"),
                "PreSubmitted" => tracing::info!("Order Pre-Submitted"),
                "Submitted" => tracing::info!("Order Submitted"),
                "Filled" => {
                    tracing::info!("Order Filled");
                    return Ok(());
                }
                "ApiCancelled" | "Cancelled" => {
                    return Err(eyre!("Order Cancelled: {status:#?}"));
                }
                other => tracing::info!("unknown order status: {other}"),
            },
            Ok(_) => {}
            Err(e) => return Err(eyre!("error submitting order: {e}")),
        }
    }
    Err(eyre!("order status subscription ended"))
}
