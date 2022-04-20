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

    /// Number of invites to generate.
    #[clap(short, long, default_value_t = 1000)]
    count: u16,

    /// Invite lifetime in seconds. Max: 604800 (one week).
    #[clap(short, long, default_value_t = 604800)]
    expiration: u32,
}

fn main() {
    let mut count = 0;
    let args = Args::parse();
    let token = &args.token.to_owned();
    let ds = Discord::from_user_token(token).unwrap();
    let file_name = format!("invites_{:?}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()).to_owned();
    let mut file = OpenOptions::new().append(true).create(true).open(file_name).unwrap();
    println!();
    for _ in 0..=args.count / 5  {
        for _ in 0..5 {
            count += 1;
            let code = ds.create_invite(ChannelId(966448749222166581), 604800 - count, 1, false).unwrap().code;
            let invite = format!("discord.gg/{code}\n");
            println!("{invite}");
            file.write_all(invite.as_bytes()).unwrap();
        }
        sleep(Duration::from_secs(15));
    }
}
