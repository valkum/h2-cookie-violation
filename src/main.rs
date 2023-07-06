use std::error::Error;
use std::net::SocketAddr;

use std::time::Duration;

use bytes::Bytes;
use h2::server::{self, SendResponse};
use h2::RecvStream;
use http::Request;
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;
use tokio::select;
use tokio::sync::oneshot::{channel, Sender};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let _ = env_logger::try_init();

    let listener = TcpListener::bind("127.0.0.1:0").await?;

    let (tx, mut rx) = channel::<(String, String)>();
    tokio::spawn(fire_request(listener.local_addr().unwrap(), tx));

    loop {
        println!("listening on {:?}", listener.local_addr());
        select! {
             Ok(res) = &mut rx => {
                tokio::time::sleep(Duration::from_secs(1)).await;
                println!("{}", res.0);
                println!("{}", res.1);
                break;
            }
            x = listener.accept() => {
                match x {
                    Ok((socket, _peer_addr)) => {
                        tokio::spawn(async move {
                            if let Err(e) = serve(socket).await {
                                println!("  -> err={:?}", e);
                            }
                        });
                    },
                    Err(e) => {
                        println!("accept error: {:?}", e);
                    }
                }
            }
        }
    }
    println!("finished listening");

    Ok(())
}

async fn fire_request(addr: SocketAddr, tx: Sender<(String, String)>) {
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("fire request");
    let res = Command::new("go")
        .arg("run")
        .arg("src/client.go")
        .arg(format!("http://{addr}"))
        .output()
        .await
        .unwrap();

    tx.send((
        String::from_utf8(res.stdout).unwrap(),
        String::from_utf8(res.stderr).unwrap(),
    ))
    .unwrap();
}

async fn serve(socket: TcpStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut connection = server::handshake(socket).await?;
    println!("H2 connection bound");

    while let Some(result) = connection.accept().await {
        let (request, respond) = result?;

        tokio::spawn(async move {
            if let Err(e) = handle_request(request, respond).await {
                println!("error while handling request: {}", e);
            }
        });
    }

    println!("~~~~~~~~~~~ H2 connection CLOSE !!!!!! ~~~~~~~~~~~");
    Ok(())
}

async fn handle_request(
    mut request: Request<RecvStream>,
    mut respond: SendResponse<Bytes>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("GOT request: {:?}", request);

    let body = request.body_mut();
    while let Some(data) = body.data().await {
        let data = data?;
        println!("<<<< recv {:?}", data);
        let _ = body.flow_control().release_capacity(data.len());
    }

    let response = http::Response::new(());
    let mut send = respond.send_response(response, false)?;
    println!(">>>> send");
    send.send_data(Bytes::from_static(b"hello "), false)?;
    send.send_data(Bytes::from_static(b"world\n"), true)?;

    Ok(())
}
