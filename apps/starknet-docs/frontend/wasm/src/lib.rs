use wasm_bindgen::prelude::*;

use ekubo_sdk::chain::starknet::Starknet;
use ekubo_sdk::quoting::pools::concentrated::{ConcentratedPool, TickSpacing};
use ekubo_sdk::quoting::types::{Pool, PoolConfig, PoolKey, QuoteParams, Tick, TokenAmount};
use ruint::aliases::U256;
use starknet_types_core::felt::Felt;

#[derive(serde::Deserialize)]
struct TickInput {
    index: i32,
    liquidity_delta: String,
}

#[derive(serde::Serialize)]
struct QuoteResult {
    output: String,
    fees: String,
    ticks_crossed: u32,
    error: Option<String>,
}

fn parse_u128_hex(s: &str) -> u128 {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u128::from_str_radix(s, 16).unwrap_or(0)
}

fn parse_u256_hex(low: &str, high: &str) -> U256 {
    let lo = parse_u128_hex(low);
    let hi = parse_u128_hex(high);
    U256::from(lo) | (U256::from(hi) << 128)
}

fn parse_i128_signed(s: &str) -> i128 {
    if let Some(rest) = s.strip_prefix('-') {
        -(parse_u128_hex(rest) as i128)
    } else {
        parse_u128_hex(s) as i128
    }
}

#[wasm_bindgen]
pub fn compute_quote(
    sqrt_ratio_low: &str,
    sqrt_ratio_high: &str,
    liquidity: &str,
    current_tick: i32,
    fee: &str,
    tick_spacing: u32,
    ticks_json: &str,
    amount_raw: &str,
    is_token1: bool,
    min_tick_searched: i32,
    max_tick_searched: i32,
) -> String {
    let sqrt_ratio = parse_u256_hex(sqrt_ratio_low, sqrt_ratio_high);
    let liquidity = parse_u128_hex(liquidity);
    let fee_val = parse_u128_hex(fee);

    let tick_inputs: Vec<TickInput> = match serde_json::from_str(ticks_json) {
        Ok(v) => v,
        Err(e) => {
            return serde_json::to_string(&QuoteResult {
                output: "0".into(),
                fees: "0".into(),
                ticks_crossed: 0,
                error: Some(format!("bad ticks json: {e}")),
            })
            .unwrap();
        }
    };

    let ticks: Vec<Tick> = tick_inputs
        .iter()
        .map(|t| Tick {
            index: t.index,
            liquidity_delta: parse_i128_signed(&t.liquidity_delta),
        })
        .collect();

    // Dummy addresses — the pool doesn't use them for quoting, only for token identity checks
    let token0 = Felt::ZERO;
    let token1 = Felt::ONE;

    let key = PoolKey {
        token0,
        token1,
        config: PoolConfig {
            extension: Felt::ZERO,
            fee: fee_val,
            pool_type_config: TickSpacing(tick_spacing),
        },
    };

    let pool = match ConcentratedPool::<Starknet>::from_partial_data(
        key,
        sqrt_ratio,
        ticks,
        min_tick_searched,
        max_tick_searched,
        liquidity,
        current_tick,
    ) {
        Ok(p) => p,
        Err(e) => {
            return serde_json::to_string(&QuoteResult {
                output: "0".into(),
                fees: "0".into(),
                ticks_crossed: 0,
                error: Some(format!("pool construction error: {e}")),
            })
            .unwrap();
        }
    };

    let amount: i128 = match amount_raw.parse::<i128>() {
        Ok(a) => a,
        Err(e) => {
            return serde_json::to_string(&QuoteResult {
                output: "0".into(),
                fees: "0".into(),
                ticks_crossed: 0,
                error: Some(format!("bad amount: {e}")),
            })
            .unwrap();
        }
    };

    let token = if is_token1 { token1 } else { token0 };

    let result = pool.quote(QuoteParams {
        token_amount: TokenAmount { token, amount },
        sqrt_ratio_limit: None,
        override_state: None,
        meta: (),
    });

    match result {
        Ok(quote) => serde_json::to_string(&QuoteResult {
            output: quote.calculated_amount.to_string(),
            fees: quote.fees_paid.to_string(),
            ticks_crossed: quote.execution_resources.initialized_ticks_crossed,
            error: None,
        })
        .unwrap(),
        Err(e) => serde_json::to_string(&QuoteResult {
            output: "0".into(),
            fees: "0".into(),
            ticks_crossed: 0,
            error: Some(format!("quote error: {e}")),
        })
        .unwrap(),
    }
}
