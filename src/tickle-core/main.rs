use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::task::{Context, Poll};

use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::{watch, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use tickle::tickle_core_server::{TickleCore, TickleCoreServer};
use tickle::{Tickle, Laughter, Gibberish, Avail, Safeword, Ack};

pub mod tickle {
    tonic::include_proto!("tickle");
}

#[derive(Debug)]
pub struct TickleCoreService {
    tickles: Arc<RwLock<HashMap<String, Tickle>>>,
}

struct StreamingOut(oneshot::Sender<Result<Tickle, Status>>);

impl Stream for StreamingOut {
    type Item = Result<Tickle, Status>;
    
    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // A stream that never resolves to anything....
        Poll::Pending
    }
}        


#[tonic::async_trait]
impl TickleCore for TickleCoreService {

//    type HelloStream = Pin<Box<dyn Stream<Item = Result<Tickle, Status>> + Send + 'static>>;
    type HelloStream = Pin<Box<dyn Stream<Item = Result<Tickle, Status>> + Send>>;

    
    async fn hello(
        &self,
        request: Request<Avail>
    ) -> Result<Response<Self::HelloStream>, Status> {
        
        println!("Client connected from: {:?}", request.remote_addr());
        
//        let (tx, rx) = oneshot::channel::<()>();
        let (tx, rx) = oneshot::channel();

        /*
        match rx.await {
            Ok(v) => println!("got = {:?}", v),
            Err(_) => println!("the sender dropped"),
        } */

        
        tokio::spawn(async move {
            let _ = rx.await;
            println!("The rx resolved therefore the client disconnected!");
        });

        dbg!("Self = {}, Request = {}, TX = {}", &self, &request, &tx);

        let foobar = Tickle { tickle: String::from("tickle.."), instance: String::from("instance.."), intent: None };

        tokio::spawn(async move {
//            tx.send(Ok(foobar.clone())).await.unwrap();
//            tx.send(foobar.clone());
        });
                     
        // Ok(Response::new(Box::pin(output) as Self::HelloStream))
        
        Ok(Response::new(
            Box::pin(StreamingOut(tx)) as Self::HelloStream
        ))
        

    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:10000".parse().unwrap();

    println!("TickleCoreServer listening on: {}", addr);

    console_subscriber::ConsoleLayer::builder()
    // set how long the console will retain data from completed tasks
        .retention(Duration::from_secs(60))
    // set the address the server is bound to
        .server_addr(([127, 0, 0, 1], 4242))
    // ... other configurations ...
        .init();
    
    //console_subscriber::init();
    
    let tickle = TickleCoreService {
        tickles: Arc::new(RwLock::new( HashMap::new() ) ),
//        features: Arc::new(data::load()),
    };

    let svc = TickleCoreServer::new(tickle);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
