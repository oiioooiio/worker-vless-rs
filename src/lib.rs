mod vless;

use bytes::Bytes;
use futures::stream::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;
use worker::*;

async fn handle_tcp(server: WebSocket, request: vless::Request) -> Result<()> {
    let mut buf = [0; 0x2000];
    let mut stream = Socket::builder().connect(&request.addr.to_string(), request.port)?;

    if !request.payload.is_empty() {
        stream.write(&request.payload).await?;
    }

    let mut first_response = true;

    let mut events = server.events()?;
    loop {
        tokio::select! {
            result = stream.read(&mut buf) => {
                let n = result?;
                if n == 0 {
                    // Normal Closure
                    server.close(Some(1000), Some("target clolsed"))?;
                    break;
                }
                if first_response {
                    first_response = false;
                    server.send_with_bytes(&[0, 0])?;
                }
                server.send_with_bytes(&buf[..n])?;
            }
            result = events.next() => {
                match result.ok_or("no event")?? {
                    ws_events::WebsocketEvent::Message(message) => {
                        let data = match message.bytes() {
                            Some(data) => data,
                            None => continue,
                        };
                        stream.write(&data).await?;
                    }
                    ws_events::WebsocketEvent::Close(_) => {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_ws(server: WebSocket, uuid: Uuid) -> Result<()> {
    let request = match server.events()?.next().await.ok_or("no first event")?? {
        ws_events::WebsocketEvent::Message(message) => {
            let data = message.bytes().ok_or("first data event is empty")?;
            match vless::Request::parse_from(Bytes::from(data), uuid) {
                Ok(request) => request,
                Err(e) => {
                    // Unsupported Data
                    server.close(Some(1003), Some("invalid request"))?;
                    return Err(format!("invalid request: {}", e).into());
                }
            }
        }
        ws_events::WebsocketEvent::Close(close) => {
            return Err(format!(
                "first event is close: code {}, reason {}",
                close.code(),
                close.reason()
            )
            .into());
        }
    };

    #[cfg(debug_assertions)]
    console_debug!("{:?}", request);

    match request.cmd {
        vless::Command::Tcp => {
            handle_tcp(server, request).await?;
        }
        _ => {
            // Unsupported Data
            server.close(Some(1003), Some("unsupported command"))?;
            return Err(format!("unsupported command: {}", request.cmd).into());
        }
    }
    Ok(())
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let uuid = env.var("UUID")?.to_string();

    if req
        .headers()
        .get("Upgrade")
        .is_ok_and(|upgrade| upgrade.is_some_and(|upgrade| upgrade == "websocket"))
    {
        let pair = WebSocketPair::new()?;

        let server = pair.server;
        server.accept()?;

        wasm_bindgen_futures::spawn_local(async move {
            match handle_ws(server, uuid.parse().unwrap_or_default()).await {
                Ok(_) => {}
                Err(e) => {
                    console_error!("handle websocket failed: {:?}", e);
                }
            }
        });

        Response::from_websocket(pair.client)
    } else {
        Response::ok(format!("{:?}", req.headers()))
    }
}
