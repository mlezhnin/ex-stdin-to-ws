use std::io::{self};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio_tungstenite::tungstenite::Utf8Bytes;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{future};
use tokio::io::BufReader;
use futures_util::{TryStreamExt, StreamExt};
use tokio_util::io::ReaderStream;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;

async fn server() {
    // Websocket & stdin setup for read and write
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let (stream, _) = listener.accept().await.expect("Listener.accept failed");
    let ws: WebSocketStream<TcpStream> = accept_async(stream).await.expect("Failure.");
    let (sink, stream) = ws.split();
    let stdin = tokio::io::stdin();

    // Begin reading
    let reader = BufReader::new(stdin);
    let reader_stream = ReaderStream::new(reader)
        .map_ok(|bytes| {
            // Convert bytes to string (falling back to binary if invalid -- not implemented, TODO)
            match std::str::from_utf8(&bytes) {
                // Unsafe: Uses from_utf8 to ensure it's valid UTF-8
                Ok(_) => Message::Text(unsafe {Utf8Bytes::from_bytes_unchecked(bytes)}),
                Err(_) => todo!(),
            }
        })
        // Maps to tungstenite errors for type correctness
        .map_err(|e| TungsteniteError::Io(e));

    // Forwards read bytes as utf-8, receives input from ws
    let recv = reader_stream.forward(sink);
    let inc = stream.try_for_each(|msg| {
        println!("Received input: {}", msg);
        future::ok(())
    });

    //pin_mut!(recv, inc);
    future::select(recv, inc).await;

}

#[tokio::main]
async fn main() -> io::Result<()> { 

    let s = tokio::spawn(server());

    let _ = s.await;

    Ok(())
}


// let stdin = tokio::io::stdin();
//     let lines = AsyncBufReadExt::lines(BufReader::new(stdin));
//     let ls = LinesStream::new(lines);
//     let ls_messages = ls.map(|line| Ok(Message::Text(line.unwrap().into())));
    
//     let recv = ls_messages.forward(sink); //messages received are forwarded to the sink

// let broadcast = ls.try_for_each({
//         let sink = Arc::clone(&sink);
//         move |line| {
//             let sink = Arc::clone(&sink);
//             async move {
//             sink.lock().await
//                 .send(line.into())
//                 .await
//                 .map(|_| ())
//                 .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
//         }
//         } // convert the error type if needed
//     });