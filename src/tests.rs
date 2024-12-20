use crate::grpc;
use acts::Config;
use acts_channel::{model::ModelInfo, ActsChannel, ActsOptions, Vars};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn grpc_start() {
    let options = Config::default();
    let port = 10081;

    tokio::spawn(async move {
        let addr = format!("127.0.0.1:{port}").parse().unwrap();
        grpc::start(addr, &options).await.unwrap();
    });

    let client = ActsChannel::connect(&format!("http://127.0.0.1:{port}")).await;
    assert!(client.is_ok());
}

#[tokio::test]
async fn grpc_action_ok() {
    let options = Config::default();
    let port = 10082;

    tokio::spawn(async move {
        let addr = format!("127.0.0.1:{port}").parse().unwrap();
        grpc::start(addr, &options).await.unwrap();
    });

    let mut client = ActsChannel::connect(&format!("http://127.0.0.1:{port}"))
        .await
        .unwrap();

    let ret = client.send::<ModelInfo>("models", Vars::new()).await;
    assert!(ret.is_ok());
}

#[tokio::test]
async fn grpc_action_err() {
    let options = Config::default();
    let port = 10083;

    tokio::spawn(async move {
        let addr = format!("127.0.0.1:{port}").parse().unwrap();
        grpc::start(addr, &options).await.unwrap();
    });

    let mut client = ActsChannel::connect(&format!("http://127.0.0.1:{port}"))
        .await
        .unwrap();

    let ret = client.send::<()>("complete", Vars::new()).await;
    assert!(ret.is_err());
}

#[tokio::test]
async fn grpc_message_all() {
    let messages = Arc::new(Mutex::new(Vec::new()));
    let options = Config::default();
    let port = 10084;

    tokio::spawn(async move {
        let addr = format!("127.0.0.1:{port}").parse().unwrap();
        grpc::start(addr, &options).await.unwrap();
    });

    let mut client = ActsChannel::connect(&format!("http://127.0.0.1:{port}"))
        .await
        .unwrap();

    let m = messages.clone();
    client
        .subscribe(
            "my_client_1",
            move |msg| {
                m.lock().unwrap().push(msg.clone());
            },
            &ActsOptions::default(),
        )
        .await;

    let model = r#"
    id: m1
    name: test
    "#;
    client.deploy(model, None).await.unwrap();
    client.start("m1", Vars::new()).await.unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert_eq!(messages.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn grpc_message_filter() {
    let messages = Arc::new(Mutex::new(Vec::new()));
    let options = Config::default();
    let port = 10085;

    tokio::spawn(async move {
        let addr = format!("127.0.0.1:{port}").parse().unwrap();
        grpc::start(addr, &options).await.unwrap();
    });

    let mut client = ActsChannel::connect(&format!("http://127.0.0.1:{port}"))
        .await
        .unwrap();

    let m = messages.clone();
    client
        .subscribe(
            "my_client_2",
            move |msg| {
                m.lock().unwrap().push(msg.clone());
            },
            &ActsOptions {
                state: Some("created".to_string()),
                ..Default::default()
            },
        )
        .await;

    let model = r#"
    id: m2
    name: test
    "#;
    client.deploy(model, None).await.unwrap();
    client.start("m2", Vars::new()).await.unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    assert_eq!(messages.lock().unwrap().len(), 1);
}
