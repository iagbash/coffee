pub mod kafka;
pub mod config;

use std::borrow::Borrow;
use std::sync::Arc;
use rdkafka::ClientConfig;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use log::{debug, error, info};
use prost::bytes::BytesMut;
use prost::Message;

use coffee::bean_command_service_server::{BeanCommandService, BeanCommandServiceServer};
use coffee::{BeansRequest, Empty};
use coffee_grpc::coffee;
use kafka::{KafkaProducer};
use config::Config;

#[derive(Default)]
pub struct MyServer {}

#[tonic::async_trait]
impl BeanCommandService for MyServer {
    async fn store_beans(&self, _req: Request<BeansRequest>) -> Result<Response<Empty>, Status> {
        let req = _req.into_inner();
        let mut buffer = BytesMut::with_capacity(req.encoded_len());
        req.encode(&mut buffer).unwrap();

        let app_config = Arc::new(Config {
            debug: false,
            host: "0.0.0.0".into(),
            port: "8080".into(),
            postgres_database_url: "localhost".into(),
            kafka_brokers: "localhost:39092,localhost:29092".into(),
            kafka_topic: "metrics".into(),
            postgres_cert_path: None,
            kafka_ca_cert_path: None,
            kafka_username: None,
            kafka_password: None,
        });
        // Create a kafka producer
        let kproducer = create_producer(app_config.clone());
        kproducer.produce(buffer, &app_config.kafka_topic).await;
        // debug!("Received data on the incoming channel");
        // kproducer.produce(data, &config.kafka_topic).await;
        // info!(
        // 	"Published data successfully on kafka topic: {}",
        // 	&config.kafka_topic
        // );
        Ok(Response::new(Empty {}))
    }

    async fn validate_beans(&self, _: Request<BeansRequest>) -> Result<Response<Empty>, Status> {
        let response = Empty {};
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let greeter = MyServer::default();

    Server::builder()
        .add_service(BeanCommandServiceServer::new(greeter))
        .serve(addr)
        .await?;


    Ok(())
}

/// Create a producer based on the given configuration.
///
/// In case certificate path etc is provided then a sasl enabled client
/// is created else a normal client.
fn create_producer(conf: Arc<Config>) -> KafkaProducer {
    let is_tls = conf.kafka_ca_cert_path.is_some()
        && conf.kafka_password.is_some()
        && conf.kafka_username.is_some();

    if is_tls {
        let username = conf
            .kafka_username
            .as_deref()
            .expect("Kafka username is required.");
        let password = conf
            .kafka_password
            .as_deref()
            .expect("Kafka password is required.");
        let ca_path = conf
            .kafka_ca_cert_path
            .as_deref()
            .expect("Kafka ca certificate is required.");
        let producer = ClientConfig::new()
            .set("bootstrap.servers", &conf.kafka_brokers)
            .set("message.timeout.ms", "10000")
            .set("sasl.mechanisms", "PLAIN")
            .set("security.protocol", "SASL_SSL")
            .set("sasl.username", username)
            .set("sasl.password", password)
            .set("ssl.ca.location", ca_path)
            .create()
            .expect("Producer creation error");
        return KafkaProducer::new_with_producer(producer);
    }

    KafkaProducer::new(&conf.kafka_brokers)
}


