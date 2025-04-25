#![cfg(test)]
extern crate std;

use crate::{token, SingleOfferClient};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, Symbol,
};

fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(e, &sac.address()),
        token::StellarAssetClient::new(e, &sac.address()),
    )
}

fn create_single_offer_contract<'a>(
    e: &Env,
    sell_token: &Address,
    buy_token: &Address,
    sell_price: u32,
    buy_price: u32,
) -> SingleOfferClient<'a> {
    let offer = SingleOfferClient::new(e, &e.register(crate::SingleOffer, ()));
    offer.create(sell_token, buy_token, &sell_price, &buy_price);
    offer
}

#[test]
fn test() {
    let e = Env::default();
    e.mock_all_auths();

    let token_admin = Address::generate(&e);
    let seller = Address::generate(&e);
    let buyer = Address::generate(&e);

    let sell_token = create_token_contract(&e, &token_admin);
    let sell_token_client = sell_token.0;
    let sell_token_admin_client = sell_token.1;

    let buy_token = create_token_contract(&e, &token_admin);
    let buy_token_client = buy_token.0;
    let buy_token_admin_client = buy_token.1;

    let offer = create_single_offer_contract(
        &e,
        &sell_token_client.address,
        &buy_token_client.address,
        1,
        2,
    );

    sell_token_admin_client.mint(&seller, &1000);
    buy_token_admin_client.mint(&buyer, &1000);
    sell_token_client.transfer(&seller, &offer.address, &100);

    assert!(offer.try_trade(&buyer, &20_i128, &11_i128).is_err());
    offer.trade(&buyer, &20_i128, &10_i128);

    assert_eq!(sell_token_client.balance(&seller), 900);
    assert_eq!(sell_token_client.balance(&buyer), 10);
    assert_eq!(sell_token_client.balance(&offer.address), 90);
    assert_eq!(buy_token_client.balance(&seller), 20);
    assert_eq!(buy_token_client.balance(&buyer), 980);
    assert_eq!(buy_token_client.balance(&offer.address), 0);

    offer.withdraw(&sell_token_client.address, &70);

    assert_eq!(sell_token_client.balance(&seller), 970);
    assert_eq!(sell_token_client.balance(&offer.address), 20);

    offer.updt_price(&1, &1);

    offer.trade(&buyer, &10_i128, &9_i128);
    assert_eq!(sell_token_client.balance(&seller), 970);
    assert_eq!(sell_token_client.balance(&buyer), 20);
    assert_eq!(sell_token_client.balance(&offer.address), 10);
    assert_eq!(buy_token_client.balance(&seller), 30);
    assert_eq!(buy_token_client.balance(&buyer), 970);
    assert_eq!(buy_token_client.balance(&offer.address), 0);
}
