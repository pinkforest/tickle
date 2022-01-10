use std::time::{Instant, Duration};
use std::error::Error;
use futures::{Stream, StreamExt};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};

use tokio::time::timeout;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tonic::transport::Channel;

use tickle::tickle_core_client::TickleCoreClient;
use tickle::{Tickle, Laughter, Gibberish, Avail, Safeword, Ack};

pub mod tickle {
    tonic::include_proto!("tickle");
}

async fn run_tickles(client: &mut TickleCoreClient<Channel>) -> Result<(), Box<dyn Error>> {

    let avail = Avail { capacity: 1 };
    
    let mut stream = client
        .hello(Request::new(avail))
        .await?
        .into_inner();

    // tokio while let Some(tickle) = stream.next().await {
    //tokio::pin!(stream);    
    //    while let Some(tickle) = stream.message().await? {

    let done = Arc::new(AtomicBool::new(false));

    while !done.load(SeqCst) {

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        if let Ok(tickle) = timeout(tokio::time::Duration::from_secs(10), stream.message()).await {
            println!("TICKLE = {:?}", tickle);
        }
        else {
            println!("Boo..");
        }

    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut client = TickleCoreClient::connect("http://[::1]:10000").await?;

    console_subscriber::ConsoleLayer::builder()
    // set how long the console will retain data from completed tasks
        .retention(Duration::from_secs(60))
    // set the address the server is bound to
        .server_addr(([127, 0, 0, 1], 4243))
    // ... other configurations ...
        .init();

    dbg!("Server streaming tickles...");
    run_tickles(&mut client).await?;

    Ok(())
}
