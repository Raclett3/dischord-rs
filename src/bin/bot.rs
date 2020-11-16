use composer::generate::Generator;
use composer::parse::parse;
use composer::tokenize::tokenize;
use serenity::{
    async_trait,
    framework::standard::{
        macros::{command, group, hook},
        Args, CommandResult, StandardFramework,
    },
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::env;

static TOKEN_NAME: &str = "DISCHORD_TOKEN";
static MANUAL: &str = "```
Dischord
dc!help Dischordのヘルプを表示
dc!play [MML] MMLを音声ファイルに書き出し

Dischord MML 文法
以下の文字列を連ねて記述します。小文字のアルファベット部分はパラメータとして整数を入れます。
CDEFGABRn ドレミファソラシと休符に対応しています。数字を後ろにつけるとn分音符を表現します。
\".\"は付点音符を表現します。\"&\"で長さを連結すると2つの長さをタイで接続します。
<> オクターブを上げ(下げ)ます。
() 括弧で囲んだ範囲の音を同時に発音します。
Tn テンポを後ろに表記された値に変更します。
Vn 音量を変更します。デフォルトは100です。
Ln デフォルトの音符の長さを変更します。
[]n 括弧で囲んだ範囲をn回繰り返します。
; 複数の音を重ねるために、書き込み位置を先頭に戻します。
@ 音を編集します。以下のコマンドが存在します。
@n 音色を変更します。以下は指定できる波形の一覧です。
0: 矩形波(デューティ比50%), 1: 矩形波(25%), 2: 矩形波(12.5%), 3: 三角波, 4: ノコギリ波, 5: サイン波, 6: ホワイトノイズ
@Ea,d,s,r ADSRエンベロープを設定します。
@Dn,d n個の音をd/100%のデチューンで重ねて出力します。
@H{...} 括弧内の16進文字列を4bit PCMとして登録します。
@Pn 登録されたPCMをオシレーターとして使用します。
@Gn 音符の末尾に加える無音の時間を設定します。
```";

fn to_riff(mml: &str) -> Result<Vec<u8>, String> {
    let tokens = tokenize(&mml)?;
    let parsed = parse(&tokens).map_err(|x| x.to_string())?;
    Ok(Generator::new(48000.0, &parsed).into_riff())
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, MANUAL).await?;
    Ok(())
}

#[command]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.say(&ctx.http, "生成しています...").await?;

    let mml = args.rest();
    let riff = match to_riff(mml) {
        Ok(riff) => riff,
        Err(err) => {
            msg.channel_id.say(&ctx.http, &err).await?;
            return Ok(());
        }
    };

    let files = vec![(riff.as_slice(), "result.wav")];
    msg.channel_id.send_files(&ctx.http, files, |x| x).await?;
    Ok(())
}

#[hook]
async fn unknown_command(_: &Context, _: &Message, unknown_command_name: &str) {
    println!("Unknown commmand: {}", unknown_command_name);
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Dischord is ready: {}", ready.user.name);
    }
}

#[group]
#[commands(help, play)]
struct Commands;

#[tokio::main]
async fn main() {
    println!("Dischord v0.1.0");
    let error_msg = format!("Env variable {} must be present", TOKEN_NAME);
    let token = env::var(TOKEN_NAME).expect(&error_msg);

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("dc!"))
        .group(&COMMANDS_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Failed to init the client");

    if let Err(reason) = client.start().await {
        println!("Client error: {:?}", reason);
    }
}
