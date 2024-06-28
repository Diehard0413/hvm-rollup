use crate::{bench, util::parse_interface, BenchOpts, Error};
use clap::Parser;
use futures_util::{StreamExt, TryStreamExt};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};
use url::Url;

#[derive(Debug, Clone, Parser)]
pub struct ConnectOpts {
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

    #[arg(long)]
    pub json: bool,
}

pub async fn start(opts: ConnectOpts) {
    let bench_opts = BenchOpts {
        url: opts.url,
        count: opts.count,
        rate: opts.rate,
        keepalive: opts.keepalive,
        threads: opts.threads,
        interface: opts.interface,
    };
    bench(
        bench_opts,
        |stream| wait(stream),
        move |now, stats| {
            if opts.json {
                let json = serde_json::json!({
                    "elapsed": now.elapsed().as_millis(),
                    "connect_stats": stats,
                });
                println!("{}", serde_json::to_string(&json).unwrap());
            } else {
                let time = stats.success_time;
                let time = format!(
                    "avg: {}ms max: {}ms min: {}ms",
                    time.avg.as_millis(),
                    time.max.as_millis(),
                    time.min.as_millis(),
                );
                println!(
                    "elapsed: {}ms connections: {} error: {} connect time: [{}] ",
                    now.elapsed().as_millis(),
                    stats.alive,
                    stats.error,
                    time,
                );
            }
        },
    )
    .await;
}

async fn wait(stream: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<(), Error> {
    println!("Waiting...");

    if let MaybeTlsStream::Plain(ref tcp_stream) = *stream.get_ref() {
        println!("Local address: {:?}", tcp_stream.local_addr());
        println!("Peer address: {:?}", tcp_stream.peer_addr());
    } else {
        println!("TLS stream is not supported in this example.");
    }
    
    let (_write, read) = stream.split();

    read.try_for_each(|_message| async { Ok(()) }).await?;
    Ok(())
}
