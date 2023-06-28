use std::str::FromStr;

use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coin, Addr};

use crate::admin::set_route;
use crate::contract::instantiate;
use crate::helpers::Scaled;
use injective_cosmwasm::{OwnedDepsExt, TEST_MARKET_ID_1, TEST_MARKET_ID_2};
use injective_math::FPDecimal;

use crate::msg::{FeeRecipient, InstantiateMsg};
use crate::queries::{estimate_swap_result, SwapQuantity};
use crate::state::get_all_swap_routes;
use crate::testing::test_utils::{
    human_to_dec, mock_deps_eth_inj, mock_realistic_deps_eth_inj, round_usd_like_fee, Decimals,
    MultiplierQueryBehavior, TEST_USER_ADDR,
};
use crate::types::{FPCoin, SwapRoute};

#[test]
fn test_calculate_swap_price_source_quantity() {
    let mut deps = mock_deps_eth_inj(MultiplierQueryBehavior::Success);
    let admin = &Addr::unchecked(TEST_USER_ADDR);

    instantiate(
        deps.as_mut_deps(),
        mock_env(),
        mock_info(admin.as_ref(), &[coin(1_000u128, "usdt")]),
        InstantiateMsg {
            fee_recipient: FeeRecipient::Address(admin.to_owned()),
            admin: admin.to_owned(),
        },
    )
    .unwrap();
    set_route(
        deps.as_mut_deps(),
        &Addr::unchecked(TEST_USER_ADDR),
        "eth".to_string(),
        "inj".to_string(),
        vec![TEST_MARKET_ID_1.into(), TEST_MARKET_ID_2.into()],
    )
    .unwrap();

    let actual_swap_result = estimate_swap_result(
        deps.as_ref(),
        &mock_env(),
        "eth".to_string(),
        "inj".to_string(),
        SwapQuantity::InputQuantity(FPDecimal::from_str("12").unwrap()),
    )
    .unwrap();

    assert_eq!(
        actual_swap_result.result_quantity,
        FPDecimal::must_from_str("2879.74"),
        "Wrong amount of swap execution estimate received"
    ); // value rounded to min tick

    assert_eq!(
        actual_swap_result.expected_fees.len(),
        2,
        "Wrong number of fee entries received"
    );

    // values from the spreadsheet
    let expected_fee_1 = FPCoin {
        amount: FPDecimal::must_from_str("9368.749003"),
        denom: "usdt".to_string(),
    };

    // values from the spreadsheet
    let expected_fee_2 = FPCoin {
        amount: FPDecimal::must_from_str("9444"),
        denom: "usdt".to_string(),
    };

    assert_eq!(
        round_usd_like_fee(
            &actual_swap_result.expected_fees[0],
            FPDecimal::must_from_str("0.000001")
        ),
        expected_fee_2,
        "Wrong amount of first fee received"
    );

    assert_eq!(
        round_usd_like_fee(
            &actual_swap_result.expected_fees[1],
            FPDecimal::must_from_str("0.000001")
        ),
        expected_fee_1,
        "Wrong amount of second fee received"
    );
}

#[test]
fn test_calculate_swap_price_self_relaying_source_quantity() {
    let mut deps = mock_deps_eth_inj(MultiplierQueryBehavior::Success);
    let admin = &Addr::unchecked(TEST_USER_ADDR);

    instantiate(
        deps.as_mut_deps(),
        mock_env(),
        mock_info(admin.as_ref(), &[coin(1_000u128, "usdt")]),
        InstantiateMsg {
            fee_recipient: FeeRecipient::SwapContract,
            admin: admin.to_owned(),
        },
    )
    .unwrap();

    set_route(
        deps.as_mut_deps(),
        &Addr::unchecked(TEST_USER_ADDR),
        "eth".to_string(),
        "inj".to_string(),
        vec![TEST_MARKET_ID_1.into(), TEST_MARKET_ID_2.into()],
    )
    .unwrap();

    let actual_swap_result = estimate_swap_result(
        deps.as_ref(),
        &mock_env(),
        "eth".to_string(),
        "inj".to_string(),
        SwapQuantity::InputQuantity(FPDecimal::from_str("12").unwrap()),
    )
    .unwrap();

    assert_eq!(
        actual_swap_result.result_quantity,
        FPDecimal::must_from_str("2888.78"),
        "Wrong amount of swap execution estimate received"
    ); // value rounded to min tick

    assert_eq!(
        actual_swap_result.expected_fees.len(),
        2,
        "Wrong number of fee entries received"
    );

    // values from the spreadsheet
    let expected_fee_1 = FPCoin {
        amount: FPDecimal::must_from_str("5666.4"),
        denom: "usdt".to_string(),
    };

    // values from the spreadsheet
    let expected_fee_2 = FPCoin {
        amount: FPDecimal::must_from_str("5639.2664"),
        denom: "usdt".to_string(),
    };

    assert_eq!(
        round_usd_like_fee(
            &actual_swap_result.expected_fees[0],
            FPDecimal::must_from_str("0.000001")
        ),
        expected_fee_1,
        "Wrong amount of fee received"
    );

    assert_eq!(
        round_usd_like_fee(
            &actual_swap_result.expected_fees[1],
            FPDecimal::must_from_str("0.000001")
        ),
        expected_fee_2,
        "Wrong amount of fee received"
    )
}

#[test]
fn test_calculate_estimate_when_selling_both_quantity_directions_simple() {
    let mut deps = mock_realistic_deps_eth_inj(MultiplierQueryBehavior::Success);
    let admin = &Addr::unchecked(TEST_USER_ADDR);

    instantiate(
        deps.as_mut_deps(),
        mock_env(),
        mock_info(admin.as_ref(), &[coin(1_000u128, "usdt")]),
        InstantiateMsg {
            fee_recipient: FeeRecipient::Address(admin.to_owned()),
            admin: admin.to_owned(),
        },
    )
    .unwrap();
    set_route(
        deps.as_mut_deps(),
        &Addr::unchecked(TEST_USER_ADDR),
        "eth".to_string(),
        "usdt".to_string(),
        vec![TEST_MARKET_ID_1.into()],
    )
    .unwrap();

    let eth_input_amount = human_to_dec("4.08", Decimals::Eighteen);

    let input_swap_estimate = estimate_swap_result(
        deps.as_ref(),
        &mock_env(),
        "eth".to_string(),
        "usdt".to_string(),
        SwapQuantity::InputQuantity(eth_input_amount),
    )
    .unwrap();

    let expected_usdt_result_quantity = human_to_dec("8115.53875488", Decimals::Six);

    assert_eq!(
        input_swap_estimate.result_quantity, expected_usdt_result_quantity,
        "Wrong amount of swap execution estimate received when using source quantity"
    ); // value rounded to min tick

    assert_eq!(
        input_swap_estimate.expected_fees.len(),
        1,
        "Wrong number of fee entries received"
    );

    let expected_usdt_fee_amount = human_to_dec("32.59252512", Decimals::Six);

    // values from the spreadsheet
    let expected_fee_2 = FPCoin {
        amount: expected_usdt_fee_amount,
        denom: "usdt".to_string(),
    };

    assert_eq!(
        round_usd_like_fee(
            &input_swap_estimate.expected_fees[0],
            FPDecimal::must_from_str("0.000001")
        ),
        expected_fee_2,
        "Wrong amount of first fee received"
    );

    let output_swap_estimate = estimate_swap_result(
        deps.as_ref(),
        &mock_env(),
        "eth".to_string(),
        "usdt".to_string(),
        SwapQuantity::OutputQuantity(expected_usdt_result_quantity),
    )
    .unwrap();

    let diff = (output_swap_estimate.result_quantity - eth_input_amount).abs();
    println!("eth diff: {}", diff.scaled(-18));

    assert_eq!(
        output_swap_estimate.result_quantity, eth_input_amount,
        "Wrong amount of swap execution estimate received when using target quantity"
    ); // value rounded to min tick

    assert_eq!(
        output_swap_estimate.expected_fees.len(),
        1,
        "Wrong number of fee entries received"
    );

    assert_eq!(
        round_usd_like_fee(
            &output_swap_estimate.expected_fees[0],
            FPDecimal::must_from_str("0.000001")
        ),
        expected_fee_2,
        "Wrong amount of first fee received"
    );
}

#[test]
fn test_calculate_estimate_when_buying_both_quantity_directions_simple() {
    let mut deps = mock_realistic_deps_eth_inj(MultiplierQueryBehavior::Success);
    let admin = &Addr::unchecked(TEST_USER_ADDR);

    instantiate(
        deps.as_mut_deps(),
        mock_env(),
        mock_info(admin.as_ref(), &[coin(1_000u128, "usdt")]),
        InstantiateMsg {
            fee_recipient: FeeRecipient::Address(admin.to_owned()),
            admin: admin.to_owned(),
        },
    )
    .unwrap();
    set_route(
        deps.as_mut_deps(),
        &Addr::unchecked(TEST_USER_ADDR),
        "eth".to_string(),
        "usdt".to_string(),
        vec![TEST_MARKET_ID_1.into()],
    )
    .unwrap();

    let usdt_input_amount = human_to_dec("8000", Decimals::Six);

    let input_swap_estimate = estimate_swap_result(
        deps.as_ref(),
        &mock_env(),
        "usdt".to_string(),
        "eth".to_string(),
        SwapQuantity::InputQuantity(usdt_input_amount),
    )
    .unwrap();

    let expected_eth_result_quantity = human_to_dec("3.988", Decimals::Eighteen);

    assert_eq!(
        input_swap_estimate.result_quantity, expected_eth_result_quantity,
        "Wrong amount of swap execution estimate received when using source quantity"
    ); // value rounded to min tick

    assert_eq!(
        input_swap_estimate.expected_fees.len(),
        1,
        "Wrong number of fee entries received"
    );

    let expected_usdt_fee_amount = human_to_dec("31.872509960159", Decimals::Six);

    // values from the spreadsheet
    let expected_fee_2 = FPCoin {
        amount: expected_usdt_fee_amount,
        denom: "usdt".to_string(),
    };

    assert_eq!(
        round_usd_like_fee(
            &input_swap_estimate.expected_fees[0],
            FPDecimal::must_from_str("0.000001")
        ),
        expected_fee_2,
        "Wrong amount of first fee received"
    );

    let output_swap_estimate = estimate_swap_result(
        deps.as_ref(),
        &mock_env(),
        "usdt".to_string(),
        "eth".to_string(),
        SwapQuantity::OutputQuantity(expected_eth_result_quantity),
    )
    .unwrap();

    let diff = (output_swap_estimate.result_quantity - usdt_input_amount).abs();
    println!("usdt diff: {}", diff.scaled(-6));

    assert_eq!(
        output_swap_estimate.result_quantity, usdt_input_amount,
        "Wrong amount of swap execution estimate received when using target quantity"
    ); // value rounded to min tick

    assert_eq!(
        output_swap_estimate.expected_fees.len(),
        1,
        "Wrong number of fee entries received"
    );

    assert_eq!(
        round_usd_like_fee(
            &output_swap_estimate.expected_fees[0],
            FPDecimal::must_from_str("0.000001")
        ),
        expected_fee_2,
        "Wrong amount of first fee received"
    );
}

#[test]
fn test_calculate_swap_price_target_quantity() {
    let mut deps = mock_deps_eth_inj(MultiplierQueryBehavior::Success);
    let admin = &Addr::unchecked(TEST_USER_ADDR);

    instantiate(
        deps.as_mut_deps(),
        mock_env(),
        mock_info(admin.as_ref(), &[coin(1_000u128, "usdt")]),
        InstantiateMsg {
            fee_recipient: FeeRecipient::Address(admin.to_owned()),
            admin: admin.to_owned(),
        },
    )
    .unwrap();
    set_route(
        deps.as_mut_deps(),
        &Addr::unchecked(TEST_USER_ADDR),
        "eth".to_string(),
        "inj".to_string(),
        vec![TEST_MARKET_ID_1.into(), TEST_MARKET_ID_2.into()],
    )
    .unwrap();

    let actual_swap_result = estimate_swap_result(
        deps.as_ref(),
        &mock_env(),
        "eth".to_string(),
        "inj".to_string(),
        SwapQuantity::OutputQuantity(FPDecimal::from_str("2879.743675898814381036").unwrap()),
    )
    .unwrap();

    assert_eq!(
        actual_swap_result.result_quantity,
        FPDecimal::must_from_str("12"),
        "Wrong amount of swap execution estimate received"
    ); // value rounded to min tick

    assert_eq!(
        actual_swap_result.expected_fees.len(),
        2,
        "Wrong number of fee entries received"
    );

    // values from the spreadsheet
    let expected_fee_1 = FPCoin {
        amount: FPDecimal::must_from_str("9368.749003"),
        denom: "usdt".to_string(),
    };

    // values from the spreadsheet
    let expected_fee_2 = FPCoin {
        amount: FPDecimal::must_from_str("9444"),
        denom: "usdt".to_string(),
    };

    assert_eq!(
        round_usd_like_fee(
            &actual_swap_result.expected_fees[0],
            FPDecimal::must_from_str("0.000001")
        ),
        expected_fee_2,
        "Wrong amount of first fee received"
    );

    assert_eq!(
        round_usd_like_fee(
            &actual_swap_result.expected_fees[1],
            FPDecimal::must_from_str("0.000001")
        ),
        expected_fee_1,
        "Wrong amount of second fee received"
    );
}

#[test]
fn get_all_queries_returns_empty_array_if_no_routes_are_set() {
    let mut deps = mock_deps_eth_inj(MultiplierQueryBehavior::Success);
    let admin = &Addr::unchecked(TEST_USER_ADDR);

    instantiate(
        deps.as_mut_deps(),
        mock_env(),
        mock_info(admin.as_ref(), &[coin(1_000u128, "usdt")]),
        InstantiateMsg {
            fee_recipient: FeeRecipient::SwapContract,
            admin: admin.to_owned(),
        },
    )
    .unwrap();

    let all_routes_result = get_all_swap_routes(deps.as_ref().storage);

    assert!(all_routes_result.is_ok(), "Error getting all routes");
    assert!(
        all_routes_result.unwrap().is_empty(),
        "Routes should be empty"
    );
}

#[test]
fn get_all_queries_returns_expected_array_if_routes_are_set() {
    let mut deps = mock_deps_eth_inj(MultiplierQueryBehavior::Success);
    let admin = &Addr::unchecked(TEST_USER_ADDR);

    instantiate(
        deps.as_mut_deps(),
        mock_env(),
        mock_info(admin.as_ref(), &[coin(1_000u128, "usdt")]),
        InstantiateMsg {
            fee_recipient: FeeRecipient::SwapContract,
            admin: admin.to_owned(),
        },
    )
    .unwrap();

    set_route(
        deps.as_mut_deps(),
        &Addr::unchecked(TEST_USER_ADDR),
        "eth".to_string(),
        "inj".to_string(),
        vec![TEST_MARKET_ID_1.into(), TEST_MARKET_ID_2.into()],
    )
    .unwrap();

    set_route(
        deps.as_mut_deps(),
        &Addr::unchecked(TEST_USER_ADDR),
        "eth".to_string(),
        "usdt".to_string(),
        vec![TEST_MARKET_ID_1.into()],
    )
    .unwrap();

    set_route(
        deps.as_mut_deps(),
        &Addr::unchecked(TEST_USER_ADDR),
        "usdt".to_string(),
        "inj".to_string(),
        vec![TEST_MARKET_ID_2.into()],
    )
    .unwrap();

    let all_routes_result = get_all_swap_routes(deps.as_ref().storage);
    assert!(all_routes_result.is_ok(), "Error getting all routes");

    let eth_inj_route = SwapRoute {
        source_denom: "eth".to_string(),
        target_denom: "inj".to_string(),
        steps: vec![TEST_MARKET_ID_1.into(), TEST_MARKET_ID_2.into()],
    };

    let eth_usdt_route = SwapRoute {
        source_denom: "eth".to_string(),
        target_denom: "usdt".to_string(),
        steps: vec![TEST_MARKET_ID_1.into()],
    };

    let usdt_inj_route = SwapRoute {
        source_denom: "usdt".to_string(),
        target_denom: "inj".to_string(),
        steps: vec![TEST_MARKET_ID_2.into()],
    };

    let all_routes = all_routes_result.unwrap();
    assert_eq!(
        all_routes,
        vec![eth_inj_route, eth_usdt_route, usdt_inj_route],
        "Incorrect routes returned"
    );
}
