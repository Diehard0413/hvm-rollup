use crate::util::{gen_close, gen_req, parse_interface};
use crate::{add1, bench_message, BenchOpts, Error, MessageStats};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use tokio_tungstenite::MaybeTlsStream;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::{time, time::Duration};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use url::Url;

#[derive(Debug, Clone, Parser)]
pub struct ReqOpts {
    #[arg(value_name = "URL")]
    pub url: Url,

    #[arg(short = 'c', long, default_value = "100", value_name = "NUM")]
    pub count: usize,

    #[arg(short = 'r', long, default_value = "50", value_name = "NUM")]
    pub rate: usize,

    #[arg(short = 'k', long, default_value = "0", value_name = "NUM")]
    pub keepalive: u64,

    #[arg(short = 't', long, default_value = "0", value_name = "NUM")]
    pub threads: usize,

    #[arg(short = 'i', long, value_name = "IP", value_parser = parse_interface)]
    pub interface: Option<Vec<SocketAddr>>,

    #[arg(long, default_value = "1", value_name = "NUM")]
    pub limit: usize,

    #[arg(long)]
    pub json: bool,
}

pub async fn start(opts: ReqOpts) {
    let bench_opts = BenchOpts {
        url: opts.url,
        count: opts.count,
        rate: opts.rate,
        keepalive: opts.keepalive,
        threads: opts.threads,
        interface: opts.interface,
    };
    let event_stats = Arc::new(Mutex::new(MessageStats {
        total: 0,
        ..Default::default()
    }));
    let c_stats = event_stats.clone();
    let limit = opts.limit;

    bench_message(bench_opts, event_stats, opts.json, move |stream| {
        loop_req(stream, c_stats, limit)
    })
    .await;
}

pub async fn loop_req(
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    stats: Arc<Mutex<MessageStats>>,
    limit: usize,
) -> Result<(), Error> {
    let (mut write, mut read) = stream.split();
    time::sleep(Duration::from_secs(1)).await;
    let mut start = time::Instant::now();
    add1!(stats, total);
    let req = gen_req(None, None, limit);
    write.send(Message::Text(req)).await?;
    loop {
        let msg = read.next().await;
        match msg {
            Some(msg) => {
                let msg = msg?;
                if msg.is_text() {
                    let req = gen_req(None, None, limit);
                    let msg = msg.to_string();
                    {
                        let mut r = stats.lock();
                        r.size += msg.len() + req.len();
                    }
                    if msg.contains("EVENT") {
                        add1!(stats, event);
                    }
                    if msg.contains("EOSE") {
                        {
                            let mut r = stats.lock();
                            r.success_time = r.success_time.add(start.elapsed());
                        }
                        add1!(stats, complete, total);
                        write.send(Message::Text(gen_close(None))).await?;
                        start = time::Instant::now();
                        write.send(Message::Text(req)).await?;
                    }
                } else if msg.is_close() {
                    break;
                }
            }
            None => break,
        }
    }
    Ok(())
}
