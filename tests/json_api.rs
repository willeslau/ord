use {super::*, bitcoin::BlockHash};

#[test]
fn get_sat_without_sat_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let response = TestServer::spawn_with_server_args(&rpc_server, &[], &["--enable-json-api"])
    .json_request("/sat/2099999997689999");

  assert_eq!(response.status(), StatusCode::OK);

  let mut sat_json: SatJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  // this is a hack to ignore the timestamp, since it changes for every request
  sat_json.timestamp = 0;

  pretty_assert_eq!(
    sat_json,
    SatJson {
      number: 2099999997689999,
      decimal: "6929999.0".into(),
      degree: "5°209999′1007″0‴".into(),
      name: "a".into(),
      block: 6929999,
      cycle: 5,
      epoch: 32,
      period: 3437,
      offset: 0,
      rarity: Rarity::Uncommon,
      percentile: "100%".into(),
      satpoint: None,
      timestamp: 0,
      inscriptions: vec![],
    }
  )
}

#[test]
fn get_sat_with_inscription_and_sat_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  let (inscription_id, reveal) = inscribe(&rpc_server);

  let response =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"])
      .json_request(format!("/sat/{}", 50 * COIN_VALUE));

  assert_eq!(response.status(), StatusCode::OK);

  let sat_json: SatJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    sat_json,
    SatJson {
      number: 50 * COIN_VALUE,
      decimal: "1.0".into(),
      degree: "0°1′1″0‴".into(),
      name: "nvtcsezkbth".into(),
      block: 1,
      cycle: 0,
      epoch: 0,
      period: 0,
      offset: 0,
      rarity: Rarity::Uncommon,
      percentile: "0.00023809523835714296%".into(),
      satpoint: Some(SatPoint::from_str(&format!("{}:{}:{}", reveal, 0, 0)).unwrap()),
      timestamp: 1,
      inscriptions: vec![inscription_id],
    }
  )
}

#[test]
fn get_sat_with_inscription_on_common_sat_and_more_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  inscribe(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let Inscribe { reveal, .. } = CommandBuilder::new(format!(
    "wallet inscribe --satpoint {}:0:1 --fee-rate 1 --file foo.txt",
    txid
  ))
  .write("foo.txt", "FOO")
  .rpc_server(&rpc_server)
  .run_and_deserialize_output();

  rpc_server.mine_blocks(1);
  let inscription_id = InscriptionId {
    txid: reveal,
    index: 0,
  };

  let response =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"])
      .json_request(format!("/sat/{}", 3 * 50 * COIN_VALUE + 1));

  assert_eq!(response.status(), StatusCode::OK);

  let sat_json: SatJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    sat_json,
    SatJson {
      number: 3 * 50 * COIN_VALUE + 1,
      decimal: "3.1".into(),
      degree: "0°3′3″1‴".into(),
      name: "nvtblvikkiq".into(),
      block: 3,
      cycle: 0,
      epoch: 0,
      period: 0,
      offset: 1,
      rarity: Rarity::Common,
      percentile: "0.000714285715119048%".into(),
      satpoint: Some(SatPoint::from_str(&format!("{}:{}:{}", reveal, 0, 0)).unwrap()),
      timestamp: 3,
      inscriptions: vec![inscription_id],
    }
  )
}

#[test]
fn get_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  let (inscription_id, reveal) = inscribe(&rpc_server);

  let response =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"])
      .json_request(format!("/inscription/{}", inscription_id));

  assert_eq!(response.status(), StatusCode::OK);

  let mut inscription_json: InscriptionJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();
  assert_regex_match!(inscription_json.address.unwrap(), r"bc1p.*");
  inscription_json.address = None;

  pretty_assert_eq!(
    inscription_json,
    InscriptionJson {
      parent: None,
      children: Vec::new(),
      inscription_id,
      inscription_number: 0,
      genesis_height: 2,
      genesis_fee: 138,
      output_value: Some(10000),
      address: None,
      sat: Some(ord::Sat(50 * COIN_VALUE)),
      satpoint: SatPoint::from_str(&format!("{}:{}:{}", reveal, 0, 0)).unwrap(),
      content_type: Some("text/plain;charset=utf-8".to_string()),
      content_length: Some(3),
      timestamp: 2,
      previous: None,
      next: None,
      rune: None,
    }
  )
}

#[test]
fn get_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let mut inscriptions = Vec::new();

  // Create 150 inscriptions
  for i in 0..50 {
    rpc_server.mine_blocks(1);
    rpc_server.mine_blocks(1);
    rpc_server.mine_blocks(1);

    let txid = rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[
        (i * 3 + 1, 0, 0, witness.clone()),
        (i * 3 + 2, 0, 0, witness.clone()),
        (i * 3 + 3, 0, 0, witness.clone()),
      ],
      ..Default::default()
    });

    inscriptions.push(InscriptionId { txid, index: 0 });
    inscriptions.push(InscriptionId { txid, index: 1 });
    inscriptions.push(InscriptionId { txid, index: 2 });
  }

  rpc_server.mine_blocks(1);

  let server =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"]);

  let response = server.json_request("/inscriptions");
  assert_eq!(response.status(), StatusCode::OK);
  let inscriptions_json: InscriptionsJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(inscriptions_json.inscriptions.len(), 100);
  assert!(inscriptions_json.more);
  assert_eq!(inscriptions_json.page_index, 0);

  let response = server.json_request("/inscriptions/1");
  assert_eq!(response.status(), StatusCode::OK);
  let inscriptions_json: InscriptionsJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(inscriptions_json.inscriptions.len(), 50);
  assert!(!inscriptions_json.more);
  assert_eq!(inscriptions_json.page_index, 1);
}

#[test]
fn get_inscriptions_in_block() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);
  rpc_server.mine_blocks(10);

  let envelope = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let txid = rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[
      (1, 0, 0, envelope.clone()),
      (2, 0, 0, envelope.clone()),
      (3, 0, 0, envelope.clone()),
    ],
    ..Default::default()
  });

  rpc_server.mine_blocks(1);

  let _ = rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(4, 0, 0, envelope.clone()), (5, 0, 0, envelope.clone())],
    ..Default::default()
  });

  rpc_server.mine_blocks(1);

  let _ = rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(6, 0, 0, envelope.clone())],
    ..Default::default()
  });

  rpc_server.mine_blocks(1);

  let server = TestServer::spawn_with_server_args(
    &rpc_server,
    &["--index-sats", "--first-inscription-height", "0"],
    &["--enable-json-api"],
  );

  // get all inscriptions from block 11
  let response = server.json_request(format!("/inscriptions/block/{}", 11));
  assert_eq!(response.status(), StatusCode::OK);

  let inscriptions_json: InscriptionsJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    inscriptions_json.inscriptions,
    vec![
      InscriptionId { txid, index: 0 },
      InscriptionId { txid, index: 1 },
      InscriptionId { txid, index: 2 },
    ]
  );
}

#[test]
fn get_output() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);
  rpc_server.mine_blocks(3);

  let envelope = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let txid = rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[
      (1, 0, 0, envelope.clone()),
      (2, 0, 0, envelope.clone()),
      (3, 0, 0, envelope.clone()),
    ],
    ..Default::default()
  });
  rpc_server.mine_blocks(1);

  let server =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"]);

  let response = server.json_request(format!("/output/{}:0", txid));
  assert_eq!(response.status(), StatusCode::OK);

  let output_json: OutputJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    output_json,
    OutputJson {
      value: 3 * 50 * COIN_VALUE,
      script_pubkey: "".to_string(),
      address: None,
      transaction: txid.to_string(),
      sat_ranges: Some(vec![
        (5000000000, 10000000000,),
        (10000000000, 15000000000,),
        (15000000000, 20000000000,),
      ],),
      inscriptions: vec![
        InscriptionId { txid, index: 0 },
        InscriptionId { txid, index: 1 },
        InscriptionId { txid, index: 2 },
      ],
      runes: BTreeMap::new(),
    }
  );
}

#[test]
fn json_request_fails_when_not_enabled() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let response =
    TestServer::spawn_with_args(&rpc_server, &[]).json_request("/sat/2099999997689999");

  assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}

#[test]
fn get_block() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  rpc_server.mine_blocks(1);

  let response = TestServer::spawn_with_server_args(&rpc_server, &[], &["--enable-json-api"])
    .json_request("/block/0");

  assert_eq!(response.status(), StatusCode::OK);

  let block_json: BlockJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(
    block_json,
    BlockJson {
      hash: "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
        .parse::<BlockHash>()
        .unwrap(),
      target: "00000000ffff0000000000000000000000000000000000000000000000000000"
        .parse::<BlockHash>()
        .unwrap(),
      best_height: 1,
      height: 0,
      inscriptions: vec![],
    }
  );
}

#[test]
fn get_status() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  inscribe(&rpc_server);

  let response =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"])
      .json_request("/status");

  assert_eq!(response.status(), StatusCode::OK);

  let mut status_json: StatusHtml = serde_json::from_str(&response.text().unwrap()).unwrap();

  let dummy_started = "2012-12-12 12:12:12+00:00"
    .parse::<DateTime<Utc>>()
    .unwrap();

  let dummy_uptime = Duration::from_secs(1);

  status_json.started = dummy_started;
  status_json.uptime = dummy_uptime;

  pretty_assert_eq!(
    status_json,
    StatusHtml {
      blessed_inscriptions: 1,
      cursed_inscriptions: 0,
      chain: Chain::Mainnet,
      height: Some(2),
      inscriptions: 1,
      lost_sats: 0,
      minimum_rune_for_next_block: Rune(99246114928149462),
      rune_index: false,
      runes: 0,
      sat_index: true,
      started: dummy_started,
      transaction_index: false,
      unrecoverably_reorged: false,
      uptime: dummy_uptime,
    }
  );
}
