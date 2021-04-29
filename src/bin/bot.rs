use composer::generate::Generator;
use composer::parse::parse;
use composer::tokenize::tokenize;
use lame::Lame;
use serenity::{
    async_trait,
    framework::standard::{
        macros::{command, group, hook},
        Args, CommandResult, StandardFramework,
    },
    model::{
        channel::Message,
        gateway::Ready,
        id::{ChannelId, GuildId},
    },
    prelude::*,
};
use songbird::{
    input::{Codec, Container, Input, Reader},
    SerenityInit, Songbird,
};
use std::collections::VecDeque;
use std::env;
use std::sync::{Arc, Mutex};

type MMLQueueItem = (Arc<Songbird>, (GuildId, ChannelId), I16Reader);

struct MMLQueue;

impl TypeMapKey for MMLQueue {
    type Value = Arc<Mutex<VecDeque<MMLQueueItem>>>;
}

union I16U8Convert {
    as_u8: [u8; 2],
    as_i16: i16,
}

struct I16Reader {
    iter: Box<dyn Iterator<Item = i16> + Send + Sync>,
}

impl I16Reader {
    fn new(iter: Box<dyn Iterator<Item = i16> + Send + Sync>) -> Self {
        I16Reader { iter }
    }
}

impl std::io::Read for I16Reader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for (i, chunk) in buf.chunks_mut(2).enumerate() {
            let item = if let Some(item) = self.iter.next() {
                item
            } else {
                return Ok(i * 2);
            };
            let as_u8 = unsafe { I16U8Convert { as_i16: item }.as_u8 };
            chunk[0] = as_u8[0];
            chunk[1] = as_u8[1];
        }
        Ok(buf.len() / 2 * 2)
    }
}

static TOKEN_NAME: &str = "DISCHORD_TOKEN";
static MANUAL: &str = "```
Dischord
dc!help Dischordのヘルプを表示
dc!play [MML] MMLを音声ファイルに書き出し
dc!playraw [MML] MMLを圧縮されていない音声ファイルに書き出し

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
@Tn 実際に鳴らされる周波数をn‰にします。
@Fxn,... エフェクトを適用します。
@FDd,f ディレイ(f‰フィードバック, dミリ秒)
```

詳細なヘルプはこちら: https://github.com/Raclett3/dischord-rs/blob/master/MML.md";

fn to_riff(mml: &str) -> Result<Vec<u8>, String> {
    let tokens = tokenize(&mml)?;
    let parsed = parse(&tokens).map_err(|x| x.to_string())?;
    Ok(Generator::new(44100.0, &parsed).into_riff())
}

fn to_i16_samples(mml: &str) -> Result<Vec<i16>, String> {
    let tokens = tokenize(&mml)?;
    let parsed = parse(&tokens).map_err(|x| x.to_string())?;
    Ok(Generator::new(44100.0, &parsed).into_i16_stream().collect())
}

fn to_i16_stream(mml: &str) -> Result<I16Reader, String> {
    let tokens = tokenize(&mml)?;
    let parsed = parse(&tokens).map_err(|x| x.to_string())?;
    Ok(I16Reader::new(Box::new(
        Generator::new(48000.0, &parsed).into_i16_stream(),
    )))
}

fn to_mp3(samples: &[i16]) -> Option<Vec<u8>> {
    let mut lame = Lame::init().ok()?;

    lame.set_quality(2).ok()?;
    lame.set_kilobitrate(192).ok()?;
    lame.set_channels(1).ok()?;
    lame.set_samplerate(44100).ok()?;

    let mp3 = lame.encode_mono(&samples).ok()?;
    Some(mp3)
}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, MANUAL).await?;
    Ok(())
}

#[command]
async fn playraw(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
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

#[command]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.say(&ctx.http, "生成しています...").await?;

    let mml = args.rest();
    let samples = match to_i16_samples(mml) {
        Ok(riff) => riff,
        Err(err) => {
            msg.channel_id.say(&ctx.http, &err).await?;
            return Ok(());
        }
    };

    if let Some(mp3) = to_mp3(&samples) {
        let files = vec![(mp3.as_slice(), "result.mp3")];
        msg.channel_id.send_files(&ctx.http, files, |x| x).await?;
    } else {
        msg.channel_id.say(&ctx.http, "予期せぬエラーが発生しました。").await?;
    }

    Ok(())
}

#[command]
async fn vcplay(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let channel_id = if let Some(channel) = channel_id {
        channel
    } else {
        msg.channel_id
            .say(&ctx.http, "ボイスチャンネルに参加してからご使用ください。")
            .await?;

        return Ok(());
    };

    let mml = args.rest();
    let stream = match to_i16_stream(mml) {
        Ok(stream) => stream,
        Err(err) => {
            msg.channel_id.say(&ctx.http, err).await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.unwrap().clone();
    {
        let data = ctx.data.write().await;

        data.get::<MMLQueue>().unwrap().lock().unwrap().push_back((
            manager,
            (guild_id, channel_id),
            stream,
        ));
    }

    msg.channel_id
        .say(&ctx.http, "キューに追加しました。")
        .await?;

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
#[commands(help, play, playraw, vcplay)]
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
        .register_songbird()
        .await
        .expect("Failed to init the client");

    let mml_queue = Arc::new(Mutex::new(VecDeque::new()));

    {
        let mut data = client.data.write().await;

        data.insert::<MMLQueue>(mml_queue.clone());
    }

    tokio::spawn(async move {
        loop {
            let (manager, (guild_id, channel_id), stream) =
                if let Some(popped) = mml_queue.lock().unwrap().pop_front() {
                    popped
                } else {
                    continue;
                };
            let (handler_lock, result) = manager.join(guild_id, channel_id).await;

            if result.is_err() {
                continue;
            }

            let mut handler = handler_lock.lock().await;

            let source = Input::new(
                false,
                Reader::Extension(Box::new(stream)),
                Codec::Pcm,
                Container::Raw,
                None,
            );
            let song = handler.play_source(source.into());
            let _ = song.set_volume(0.3);
            let _ = song.play();
            loop {
                let info = song.get_info().await;
                if let Ok(state) = info {
                    if state.playing == songbird::tracks::PlayMode::End {
                        break;
                    }
                } else {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await
            }

            let _ = handler.leave().await;
        }
    });

    if let Err(reason) = client.start().await {
        println!("Client error: {:?}", reason);
    }
}
