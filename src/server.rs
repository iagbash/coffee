use tonic::transport::Server;
use tonic::{Request, Response, Status};

use coffee::bean_command_service_server::{BeanCommandService, BeanCommandServiceServer};
use coffee::{BeansRequest, Empty};
use coffee_grpc::coffee;

#[derive(Default)]
pub struct MyServer {}

#[tonic::async_trait]
impl BeanCommandService for MyServer {
    async fn store_beans(&self, req: Request<BeansRequest>) -> Result<Response<Empty>, Status> {
        println!("{}",req.into_inner().amount);
        let response = Empty {};
        Ok(Response::new(response))
    }

    async fn validate_beans(&self,_:Request<BeansRequest>)-> Result<Response<Empty>, Status>{
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
