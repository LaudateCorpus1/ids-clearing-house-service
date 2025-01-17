// Some tests require a clean database for each test run. The tests are not able to clean
// the database after them if they added processes while running.


use core_lib::errors::*;
use core_lib::util;
use ch_lib::model::ids::message::{DOC_TYPE, IdsMessage};
use ch_lib::model::ids::request::ClearingHouseMessage;
use crate::ch_api_client::ClearingHouseApiClient;
use core_lib::api::ApiClient;
use crate::{TOKEN, delete_test_doc_type_from_keyring, insert_test_doc_type_into_keyring, CH_API, EXPECTED_SENDER_AGENT, EXPECTED_ISSUER_CONNECTOR, OTHER_TOKEN};
use ch_lib::model::ids::MessageType;
use core_lib::model::new_uuid;
use ch_lib::model::ids::InfoModelId::SimpleId;
use ch_lib::model::{OwnerList, Receipt, DataTransaction};

///Testcase: Check correctness of IDS response when querying existing document
#[test]
fn check_ids_message_when_querying_existing_document() -> Result<()> {
    // configure client_api
    let ch_api = ClearingHouseApiClient::new(CH_API);

    // prepare test data
    let dt_id = DOC_TYPE.to_string();
    let pid = String::from("check_ids_message_when_querying_existing_document");

    let ch_message: ClearingHouseMessage = serde_json::from_str(&util::read_file("tests/integration/json/query_message.json")?)?;
    let query_message = ch_message.header;

    // clean up doc type (in case of previous test failure)
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;
    insert_test_doc_type_into_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    let json_data = util::read_file("tests/integration/json/log_message.json")?;
    let existing_message = ch_api.log_message(&TOKEN.to_string(), &pid, json_data)?;
    let existing_doc: Receipt = serde_json::from_str(existing_message.payload.as_ref().unwrap())?;
    let existing_doc_id = DataTransaction::from(existing_doc).document_id;

    // run the test
    let json_data = util::read_file("tests/integration/json/query_message.json")?;
    let result = ch_api.query_with_pid_and_id(&TOKEN.to_string(), &pid, &existing_doc_id, json_data)?;

    // check the ids response
    let ids_response = result.header;
    // we expect a result message
    assert_eq!(ids_response.type_message, MessageType::ResultMessage);
    // we have one recipient agent,
    assert_eq!(ids_response.recipient_agent.as_ref().unwrap().len(), 1);
    // which is the sender of the query message
    assert_eq!(ids_response.recipient_agent.as_ref().unwrap()[0], SimpleId(query_message.sender_agent));
    // we have one recipient connector
    assert_eq!(ids_response.recipient_connector.as_ref().unwrap().len(), 1);
    // which is the sender of the query message
    assert_eq!(ids_response.recipient_connector.clone().unwrap().pop().unwrap(), query_message.issuer_connector);
    // sender agent is the clearing house (check config.yml on failure!)
    assert_eq!(ids_response.sender_agent, EXPECTED_SENDER_AGENT.to_string());
    // issuer connector is the clearing house (check config.yml on failure!)
    assert_eq!(ids_response.issuer_connector, SimpleId(EXPECTED_ISSUER_CONNECTOR.to_string()));
    // our message is the answer to the log_message
    assert_eq!(ids_response.correlation_message, query_message.id);
    //TODO: check security token
    //TODO: check auth token

    // tear down
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    Ok(())
}

/// Testcase: Query existing document
#[test]
fn test_query_existing_document() -> Result<()> {
    // configure client_api
    let ch_api = ClearingHouseApiClient::new(CH_API);

    // prepare test data
    let dt_id = DOC_TYPE.to_string();
    let pid = String::from("test_query_existing_document");

    // clean up doc type (in case of previous test failure)
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;
    insert_test_doc_type_into_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    let expected_message: ClearingHouseMessage = serde_json::from_str(&util::read_file("tests/integration/json/log_message.json")?)?;
    println!("expected: {:#?}", &expected_message);
    let json_data = util::read_file("tests/integration/json/log_message.json")?;
    let message_in_ch = ch_api.log_message(&TOKEN.to_string(), &pid, json_data)?;
    println!("message in ch: {:#?}", &message_in_ch);
    let existing_doc: Receipt = serde_json::from_str(message_in_ch.payload.as_ref().unwrap())?;
    let existing_doc_id = DataTransaction::from(existing_doc).document_id;

    // run the test
    println!("########################### QUERY #####################################");
    let json_data = util::read_file("tests/integration/json/query_message.json")?;
    let result = ch_api.query_with_pid_and_id(&TOKEN.to_string(), &pid, &existing_doc_id, json_data)?;
    let result_doc: IdsMessage = serde_json::from_str(result.payload.as_ref().unwrap())?;
    println!("########################### QUERY RESULT ##############################");
    println!("result: {:#?}", result);

    // check
    // check message_id
    assert_eq!(expected_message.header.clone().id, result_doc.id);
    // check pid
    assert_eq!(expected_message.header.clone().pid, result_doc.pid);
    // check model_version
    assert_eq!(expected_message.header.clone().model_version, result_doc.model_version);
    // check correlation message
    assert_eq!(expected_message.header.clone().correlation_message, result_doc.correlation_message);
    // check issued
    assert_eq!(expected_message.header.clone().issued, result_doc.issued);
    //TODO: check issuer connector
    //assert_eq!(expected_message.header.clone().issuer_connector, result_doc.issuer_connector);
    // check sender agent
    assert_eq!(expected_message.header.clone().sender_agent, result_doc.sender_agent);
    // check transfer contract
    assert_eq!(expected_message.header.clone().transfer_contract, result_doc.transfer_contract);
    // check content version
    assert_eq!(expected_message.header.clone().content_version, result_doc.content_version);
    //TODO: check security token
    //TODO: check authorization token
    //TODO: check payload
    //assert_eq!(expected_message.header.clone().payload, result_doc.payload);
    //TODO: check payload type
    //assert_eq!(expected_message.header.clone().payload_type, result_doc.payload_type);

    // tear down
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    Ok(())
}

//TODO: Testcase: Query non-existing document
#[test]
fn test_query_non_existing_document() -> Result<()> {
    // configure client_api
    let ch_api = ClearingHouseApiClient::new(CH_API);

    // prepare test data
    let dt_id = DOC_TYPE.to_string();
    let pid = String::from("test_query_non_existing_document");

    // clean up doc type (in case of previous test failure)
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;
    insert_test_doc_type_into_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    let json_data = util::read_file("tests/integration/json/log_message.json")?;
    let message_in_ch = ch_api.log_message(&TOKEN.to_string(), &pid, json_data)?;
    let existing_doc: Receipt = serde_json::from_str(message_in_ch.payload.as_ref().unwrap())?;
    let _existing_doc_id = DataTransaction::from(existing_doc).document_id;
    // there's a very slim chance this fails because we request a random doc_id
    let non_existing_doc_id = new_uuid();

    // run the test
    println!("########################### QUERY #####################################");
    let json_data = util::read_file("tests/integration/json/query_message.json")?;

    // should result in error
    let result = ch_api.query_with_pid_and_id(&TOKEN.to_string(), &pid, &non_existing_doc_id, json_data);
    assert!(result.err().unwrap().description().contains("404"));

    // tear down
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    Ok(())
}

///Testcase: Check correctness of IDS response when querying for pid
#[test]
fn check_ids_message_when_querying_for_pid() -> Result<()> {
    // configure client_api
    let ch_api = ClearingHouseApiClient::new(CH_API);

    // prepare test data
    let dt_id = DOC_TYPE.to_string();
    let pid = String::from("check_ids_message_when_querying_for_pid");

    let ch_message: ClearingHouseMessage = serde_json::from_str(&util::read_file("tests/integration/json/query_message.json")?)?;
    let query_message = ch_message.header;

    // clean up doc type (in case of previous test failure)
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;
    insert_test_doc_type_into_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    let json_data = util::read_file("tests/integration/json/log_message.json")?;
    let existing_message_1 = ch_api.log_message(&TOKEN.to_string(), &pid, json_data)?;
    let existing_doc_1: Receipt = serde_json::from_str(existing_message_1.payload.as_ref().unwrap())?;
    let _existing_doc_id_1 = DataTransaction::from(existing_doc_1).document_id;

    let json_data = util::read_file("tests/integration/json/log_message_2.json")?;
    let existing_message_2 = ch_api.log_message(&TOKEN.to_string(), &pid, json_data)?;
    let existing_doc_2: Receipt = serde_json::from_str(existing_message_2.payload.as_ref().unwrap())?;
    let _existing_doc_id_2 = DataTransaction::from(existing_doc_2).document_id;

    // run the test
    let json_data = util::read_file("tests/integration/json/query_message.json")?;
    let result = ch_api.query_with_pid(&TOKEN.to_string(), &pid, json_data)?;

    // check the ids response
    let ids_response = result.header;
    // we expect a result message
    assert_eq!(ids_response.type_message, MessageType::ResultMessage);
    // we have one recipient agent,
    assert_eq!(ids_response.recipient_agent.as_ref().unwrap().len(), 1);
    // which is the sender of the query message
    assert_eq!(ids_response.recipient_agent.as_ref().unwrap()[0], SimpleId(query_message.sender_agent));
    // we have one recipient connector
    assert_eq!(ids_response.recipient_connector.as_ref().unwrap().len(), 1);
    // which is the sender of the query message
    assert_eq!(ids_response.recipient_connector.clone().unwrap().pop().unwrap(), query_message.issuer_connector);
    // sender agent is the clearing house (check config.yml on failure!)
    assert_eq!(ids_response.sender_agent, EXPECTED_SENDER_AGENT.to_string());
    // issuer connector is the clearing house (check config.yml on failure!)
    assert_eq!(ids_response.issuer_connector, SimpleId(EXPECTED_ISSUER_CONNECTOR.to_string()));
    // our message is the answer to the log_message
    assert_eq!(ids_response.correlation_message, query_message.id);
    //TODO: check security token
    //TODO: check auth token

    // tear down
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    Ok(())
}

//TODO: Testcase: Query existing pid with multiple documents
#[test]
fn test_query_for_pid() -> Result<()> {
    // configure client_api
    let ch_api = ClearingHouseApiClient::new(CH_API);

    // prepare test data
    let dt_id = DOC_TYPE.to_string();
    let pid = String::from("test_query_for_pid");

    // clean up doc type (in case of previous test failure)
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;
    insert_test_doc_type_into_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    let json_data = util::read_file("tests/integration/json/log_message.json")?;
    let existing_message_1 = ch_api.log_message(&TOKEN.to_string(), &pid, json_data)?;
    let existing_doc_1: Receipt = serde_json::from_str(existing_message_1.payload.as_ref().unwrap())?;
    let _existing_doc_id_1 = DataTransaction::from(existing_doc_1).document_id;

    let json_data = util::read_file("tests/integration/json/log_message_2.json")?;
    let existing_message_2 = ch_api.log_message(&TOKEN.to_string(), &pid, json_data)?;
    let existing_doc_2: Receipt = serde_json::from_str(existing_message_2.payload.as_ref().unwrap())?;
    let _existing_doc_id_2 = DataTransaction::from(existing_doc_2).document_id;

    // run the test
    let json_data = util::read_file("tests/integration/json/query_message.json")?;
    let result = ch_api.query_with_pid(&TOKEN.to_string(), &pid, json_data)?;

    // check that we got two ids messages
    let payload_messages: Vec<IdsMessage> = serde_json::from_str(result.payload.as_ref().unwrap())?;
    assert_eq!(payload_messages.len(), 2);

    // tear down
    delete_test_doc_type_from_keyring(&TOKEN.to_string(), &pid, &dt_id)?;

    Ok(())
}

///Testcase: Query existing pid with no documents
#[test]
fn test_query_for_pid_with_no_docs() -> Result<()> {
    // configure client_api
    let ch_api = ClearingHouseApiClient::new(CH_API);

    // prepare test data i.e. create a process
    let pid_without_docs = String::from("test_pid_with_no_docs");
    let json_data = util::read_file("tests/integration/json/request_message.json")?;
    ch_api.create_process(&TOKEN.to_string(), &pid_without_docs, json_data)?;

    // run the test
    let json_data = util::read_file("tests/integration/json/query_message.json")?;
    let result = ch_api.query_with_pid(&TOKEN.to_string(), &pid_without_docs, json_data)?;

    // check that we got two ids messages
    let payload_messages: Vec<IdsMessage> = serde_json::from_str(result.payload.as_ref().unwrap())?;
    assert_eq!(payload_messages.len(), 0);

    Ok(())
}

///Testcase: Query non-existing pid
#[test]
fn test_query_for_non_existing_pid() -> Result<()> {
    // configure client_api
    let ch_api = ClearingHouseApiClient::new(CH_API);

    // prepare test data i.e. create a process
    let non_existing_pid = String::from("test_this_pid_does_not_exist_pid");

    // run the test
    let json_data = util::read_file("tests/integration/json/query_message.json")?;
    let result = ch_api.query_with_pid(&TOKEN.to_string(), &non_existing_pid, json_data);

    // We choose to send back "not authorized". If we did send back "not found", users could use this
    // oracle to find out which pids are already created
    assert!(result.err().unwrap().description().contains("401"));

    Ok(())
}

///Testcase: Query pid with unauthorized user
#[test]
fn test_query_for_unauthorized_user() -> Result<()> {
    // configure client_api
    let ch_api = ClearingHouseApiClient::new(CH_API);

    // prepare test data i.e. create a process and store a document
    let pid = String::from("test_query_for_unauthorized_user_pid");
    let json_data = util::read_file("tests/integration/json/log_message.json")?;
    let existing_message = ch_api.log_message(&TOKEN.to_string(), &pid, json_data)?;
    let _r: Receipt = serde_json::from_str(existing_message.payload.as_ref().unwrap())?;

    // run the test
    let json_data = util::read_file("tests/integration/json/query_message.json")?;
    let result = ch_api.query_with_pid(&OTHER_TOKEN.to_string(), &pid, json_data);

    // check the status code of the result message
    assert!(result.err().unwrap().description().contains("401"));

    Ok(())
}

///Testcase: Query pid with multiple authorized users
#[test]
fn test_query_for_multiple_authorized_users() -> Result<()> {
    // configure client_api
    let ch_api = ClearingHouseApiClient::new(CH_API);

    // prepare test data -- create a process
    let pid = String::from("test_query_for_multiple_authorized_users");
    let mut message: ClearingHouseMessage = serde_json::from_str(&util::read_file("tests/integration/json/request_message.json")?)?;
    let ownerlist = OwnerList::new(vec!(String::from("7A:2B:DD:2A:14:22:A3:50:3D:EA:FB:60:72:6A:FB:2E:58:41:CB:C0:keyid:CB:8C:C7:B6:85:79:A8:23:A6:CB:15:AB:17:50:2F:E6:65:43:5D:E8")));
    println!("old payload: {:#?}", &message.payload);
    message.payload = Some(serde_json::to_string(&ownerlist)?);
    println!("new payload: {:#?}", &message.payload);
    let json_data = serde_json::to_string(&message)?;
    let process_result = ch_api.create_process(&TOKEN.to_string(), &pid, json_data)?;
    assert!(process_result.payload.unwrap().contains("test_query_for_multiple_authorized_users"));

    // prepare test data -- add documents from two users
    let json_data = util::read_file("tests/integration/json/log_message.json")?;
    let existing_message_1 = ch_api.log_message(&TOKEN.to_string(), &pid, json_data)?;
    let existing_doc_1: Receipt = serde_json::from_str(existing_message_1.payload.as_ref().unwrap())?;
    let _existing_doc_id_1 = DataTransaction::from(existing_doc_1).document_id;

    let json_data = util::read_file("tests/integration/json/log_message_2.json")?;
    let existing_message_2 = ch_api.log_message(&OTHER_TOKEN.to_string(), &pid, json_data)?;
    let existing_doc_2: Receipt = serde_json::from_str(existing_message_2.payload.as_ref().unwrap())?;
    let _existing_doc_id_2 = DataTransaction::from(existing_doc_2).document_id;

    // run the test
    let json_data = util::read_file("tests/integration/json/query_message.json")?;
    let result = ch_api.query_with_pid(&TOKEN.to_string(), &pid, json_data)?;

    // check that we got two ids messages
    let payload_messages: Vec<IdsMessage> = serde_json::from_str(result.payload.as_ref().unwrap())?;
    assert_eq!(payload_messages.len(), 2);

    Ok(())
}