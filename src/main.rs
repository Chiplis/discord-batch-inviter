use std::cmp::{max};
use std::fs::OpenOptions;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use discord::Discord;
use discord::model::{ChannelId};
use clap::Parser;

/// Simple Discord script to automatically generate mass invites
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// User token - https://www.online-tech-tips.com/computer-tips/what-is-a-discord-token-and-how-to-get-one/
    #[clap(short, long)]
    token: String,

    /// Invite channel ID
    #[clap(short, long)]
    id: u64,

    /// Number of invites to generate.
    #[clap(short, long, default_value_t = 1000)]
    count: u16,

    /// Invite lifetime in seconds. Max: 604800 (one week).
    #[clap(short, long, default_value_t = 604800)]
    expiration: u64,
}

fn main() {
    let args = Args::parse();
    let token = &args.token.to_owned();
    let id = &args.id;
    let expiration = max(args.expiration, args.count as u64 * 2) as u64;
    let ds = Discord::from_user_token(token).unwrap();
    let file_name = format!("invites_{:?}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()).to_owned();
    let mut file = OpenOptions::new().append(true).create(true).open(file_name).unwrap();
    println!();
    let mut count = 0;
    for _ in 0..=args.count / 5  {
        for _ in 0..5 {
            let code = ds.create_invite(ChannelId(*id), expiration - count, 1, false).unwrap().code;
            let invite = format!("discord.gg/{code}\n");
            println!("{invite}");
            file.write_all(invite.as_bytes()).unwrap();
            count += 1;
        }
        sleep(Duration::from_secs(15));
    }
}
