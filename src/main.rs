use clap::{CommandFactory, Parser};
use discord::model::ChannelId;
use discord::Discord;
use std::cmp::min;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Simple Discord script to automatically generate mass invites
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Bot token: https://www.writebots.com/discord-bot-token/
    #[clap(short, long, conflicts_with = "user")]
    bot: Option<String>,

    /// User token: https://www.online-tech-tips.com/computer-tips/what-is-a-discord-token-and-how-to-get-one/
    #[clap(short, long)]
    user: Option<String>,

    /// Delete all active invites contained in the specified file.
    /// A blank filename will delete all invites for specified channel ID (requires -i).
    #[clap(short, long)]
    delete: Option<String>,

    /// Invite channel ID.
    #[clap(short, long)]
    id: Option<u64>,

    /// Amount of invites to generate (max = 1000).
    #[clap(short, long, default_value_t = 1000)]
    amount: u16,

    /// Invite lifetime in seconds (0 = unlimited, max = 604800).
    #[clap(short, long, default_value_t = 604800)]
    lifetime: u32,

    /// Max uses per invite (0 = unlimited, max = 100).
    #[clap(short, long, default_value_t = 1)]
    max_uses: u8,

    /// Number of operations per round.
    #[clap(short, long, default_value_t = 5)]
    round_size: u8,

    /// Seconds to wait between round of operations.
    #[clap(short, long, default_value_t = 15)]
    timeout: u8,
}

struct RateLimit {
    timeout: u8,
    round_size: u8,
}

impl RateLimit {
    fn execute<T, F>(&self, args: Vec<T>, mut op: F)
    where
        F: FnMut(T),
    {
        let mut rate_limiter = 0;
        for arg in args {
            rate_limiter += 1;
            op(arg);
            rate_limiter %= self.round_size;
            if rate_limiter == 0 {
                sleep(Duration::from_secs(self.timeout as u64));
            }
        }
    }
}

fn main() {
    let _ = Args::command().term_width(0).get_matches();
    let args = Args::parse();
    let Args {
        bot,
        user,
        delete,
        id,
        amount,
        lifetime,
        max_uses,
        timeout,
        round_size,
    } = args;

    let rate_limit = RateLimit {
        timeout,
        round_size,
    };

    let channel = id.map(ChannelId);
    let max_uses = min(max_uses, 100);
    let amount = min(amount, 1000);
    let lifetime = min(lifetime, 604800);

    let ds = if let Some(u) = user {
        Discord::from_user_token(&u)
    } else if let Some(b) = bot {
        Discord::from_bot_token(&b)
    } else {
        unreachable!()
    }
    .expect("Error creating Discord client.");

    println!();

    match delete {
        Some(d) => delete_invites(ds, d, channel, rate_limit),
        None => create_invites(
            ds,
            amount,
            id.expect("Channel ID (-i) required when creating invites"),
            lifetime,
            max_uses,
            rate_limit,
        ),
    }
}

fn delete_invites(
    ds: Discord,
    invites_file: String,
    channel: Option<ChannelId>,
    rate_limit: RateLimit,
) {
    let invite_codes: Vec<String> = match (channel, invites_file.as_str().trim()) {
        (Some(channel), "") => {
            println!(
                "No invite file specified, deleting all channel invites for channel #{channel}."
            );
            ds.get_channel_invites(channel)
                .expect("Error fetching channel invites to delete.")
                .iter()
                .map(|ri| ri.code.clone())
                .collect()
        }
        (None, "") => panic!(
            "Attempted to delete all invites for channel without the required channel ID (-i)"
        ),
        (channel, path) => {
            if channel != None {
                println!("Ignoring channel ID (-i) argument because a file was specified: {path}")
            }
            BufReader::new(File::open(path).expect("Error opening file with invites to delete."))
        }
        .lines()
        .into_iter()
        .map(|l| l.unwrap())
        .collect(),
    };
    rate_limit.execute(invite_codes, |code| {
        if let Err(e) = ds.delete_invite(&code) {
            println!("Error while deleting invite {code}, continuing with next: {e}.");
        } else {
            println!("Successfully deleted {code}.");
        }
    })
}

fn create_invites(
    ds: Discord,
    amount: u16,
    id: u64,
    max_age: u32,
    max_uses: u8,
    rate_limit: RateLimit,
) {
    let file_name = format!(
        "invites_{:?}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    )
    .to_owned();
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_name)
        .unwrap();
    rate_limit.execute((1..=amount).into_iter().collect(), move |invite_count| {
        println!("{invite_count} / {amount}");
        let code = ds
            .create_invite(ChannelId(id), max_age, max_uses, false, true)
            .unwrap()
            .code;
        let invite = format!("discord.gg/{code}\n");
        println!("{invite}");
        file.write_all(invite.as_bytes()).unwrap();
    });
}
