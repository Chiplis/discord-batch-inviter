# discord-batch-inviter

## Usage

```./discord-batch-inviter (-u||-b) {TOKEN} -i {CHANNEL_ID} # -u = User token, -b = Bot token```

Should generate a file "invites_X" with all the requested invite codes.

```./discord-batch-inviter (-u||-b) {TOKEN} -i {CHANNEL_ID} -d <FILE>```

Should delete all invites in the specified file. If a blank string is provided, all invites from the channel are deleted instead.
