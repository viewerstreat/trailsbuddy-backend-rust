use axum::Router;
use hyper::StatusCode;
use mongodb::bson::doc;
use std::sync::Arc;

use trailsbuddy_backend_rust::{
    app::build_app_routes,
    constants::{COLL_WALLET_TRANSACTIONS, DB_NAME},
    database::AppDatabase,
    models::{
        AddBalEndReq, AddBalInitReq, AddBalInitRes, GenericResponse, GetBalResponse, Money,
        WalletTransaction, WalletTransactionStatus, WalltetTransactionType,
    },
    utils::{get_random_num, parse_object_id},
};

use crate::helper::{
    build_get_request, build_post_request, create_user_and_get_token, generate_uniq_phone,
    get_database, oneshot_request, WALLET_BALANCE_PREFIX,
};

mod helper;

const GET_BAL_PATH: &str = "/api/v1/wallet/getBalance";
const ADD_BAL_INIT: &str = "/api/v1/wallet/addBalanceInit";
const ADD_BAL_END: &str = "/api/v1/wallet/addBalanceEnd";

async fn get_bal_unauthorized(app: Router) {
    let request = build_get_request(GET_BAL_PATH, None);
    let response: GenericResponse =
        oneshot_request(app, request, Some(StatusCode::UNAUTHORIZED)).await;
    assert_eq!(response.success, false);
    assert_eq!(response.message, "Missing token".to_owned());
}

async fn test_add_balance_flow(app: Router, db: Arc<AppDatabase>) {
    let (user_id, token) = create_new_user(app.clone(), db.clone()).await;
    check_balance_new_user(app.clone(), token.as_str()).await;
    let add_bal_amount = get_random_num(10u64, 100);
    let add_bal_res =
        add_balance_init_transaction(app.clone(), db.clone(), user_id, &token, add_bal_amount)
            .await;
    add_balance_end_successful(
        app.clone(),
        db.clone(),
        add_bal_res,
        user_id,
        &token,
        add_bal_amount,
    )
    .await;
}

async fn create_new_user(app: Router, db: Arc<AppDatabase>) -> (u32, String) {
    let phone = generate_uniq_phone(WALLET_BALANCE_PREFIX);
    let name = "add_balance_flow";
    let res = create_user_and_get_token(app, db, &phone, name, false).await;
    (res.data.id, res.token)
}

async fn check_balance_new_user(app: Router, token: &str) {
    let expected = Money::default();
    check_user_balance(app, token, expected).await;
}

async fn check_user_balance(app: Router, token: &str, expected: Money) {
    let request = build_get_request(GET_BAL_PATH, Some(token));
    let response: GetBalResponse = oneshot_request(app, request, Some(StatusCode::OK)).await;
    assert_eq!(response.success, true);
    assert_eq!(response.balance.real(), expected.real());
    assert_eq!(response.balance.bonus(), expected.bonus());
    assert_eq!(response.balance.withdrawable(), expected.withdrawable());
}

async fn add_balance_init_transaction(
    app: Router,
    db: Arc<AppDatabase>,
    user_id: u32,
    token: &str,
    amount: u64,
) -> AddBalInitRes {
    let add_bal_init_req = AddBalInitReq { amount };
    let add_bal_init_req = serde_json::to_string(&add_bal_init_req).unwrap();
    let request = build_post_request(ADD_BAL_INIT, &add_bal_init_req, Some(token));
    let response: AddBalInitRes = oneshot_request(app.clone(), request, Some(StatusCode::OK)).await;
    assert_eq!(response.success, true);
    assert_eq!(response.transaction_id.is_empty(), false);
    assert_eq!(response.app_upi_id.is_empty(), false);
    check_wallet_transaction_add_bal_init(db, user_id, amount).await;
    response
}

async fn check_wallet_transaction_add_bal_init(db: Arc<AppDatabase>, user_id: u32, amount: u64) {
    let filter = doc! {
        "userId": user_id,
        "transactionType": WalltetTransactionType::AddBalance.to_bson().unwrap(),
        "status": WalletTransactionStatus::Pending.to_bson().unwrap()
    };
    let transaction = db
        .find_one::<WalletTransaction>(DB_NAME, COLL_WALLET_TRANSACTIONS, Some(filter), None)
        .await
        .unwrap();
    let transaction = transaction.unwrap();
    assert_eq!(transaction.amount().real(), amount);
    assert_eq!(transaction.balance_before().real(), 0);
    assert_eq!(transaction.balance_before().bonus(), 0);
    assert_eq!(transaction.balance_after().is_none(), true);
}

async fn add_balance_end_successful(
    app: Router,
    db: Arc<AppDatabase>,
    add_bal_res: AddBalInitRes,
    user_id: u32,
    token: &str,
    amount: u64,
) {
    let tracking_id = "TEST_ADD_BALANCE_FLOW";
    let transaction_id = add_bal_res.transaction_id;
    let add_bal_end_req = AddBalEndReq {
        is_successful: true,
        amount,
        transaction_id: transaction_id.clone(),
        tracking_id: Some(tracking_id.to_owned()),
        error_reason: None,
    };
    let add_bal_end_req = serde_json::to_string(&add_bal_end_req).unwrap();
    let request = build_post_request(ADD_BAL_END, &add_bal_end_req, Some(&token));
    let response: GenericResponse =
        oneshot_request(app.clone(), request, Some(StatusCode::OK)).await;
    assert_eq!(response.success, true);
    assert_eq!(response.message, "Updated successfully".to_owned());
    let expected = Money::new(amount, 0);
    check_user_balance(app.clone(), token, expected).await;
    check_wallet_transaction_add_bal_end(db, user_id, amount, &transaction_id, &tracking_id).await;
}

async fn check_wallet_transaction_add_bal_end(
    db: Arc<AppDatabase>,
    user_id: u32,
    amount: u64,
    transaction_id: &str,
    tracking_id: &str,
) {
    let transaction_id = parse_object_id(transaction_id, "").unwrap();
    let filter = doc! {
        "_id": transaction_id,
        "userId": user_id,
        "transactionType": WalltetTransactionType::AddBalance.to_bson().unwrap(),
        "status": WalletTransactionStatus::Completed.to_bson().unwrap(),
        "trackingId": &tracking_id
    };
    let transaction = db
        .find_one::<WalletTransaction>(DB_NAME, COLL_WALLET_TRANSACTIONS, Some(filter), None)
        .await
        .unwrap();
    let transaction = transaction.unwrap();
    assert_eq!(transaction.amount().real(), amount);
    assert_eq!(transaction.balance_before().real(), 0);
    assert_eq!(transaction.balance_before().bonus(), 0);
    let balance_after = transaction.balance_after().unwrap();
    assert_eq!(balance_after.real(), amount);
    assert_eq!(balance_after.bonus(), 0);
    assert_eq!(balance_after.withdrawable(), 0);
}

#[tokio::test]
async fn test_wallet_balance() {
    let db_client = get_database().await;
    let db_client = Arc::new(db_client);
    let app = build_app_routes(db_client.clone());

    tokio::join!(
        get_bal_unauthorized(app.clone()),
        test_add_balance_flow(app.clone(), db_client.clone()),
    );
}
