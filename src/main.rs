use dotenv;
use std::{collections::{HashMap, HashSet}, sync::Arc};
use serenity::{utils::Colour, async_trait, client::bridge::gateway::ShardManager,
    framework::standard::{
        Args, CommandResult, CommandGroup,
        DispatchError, HelpOptions, help_commands, StandardFramework,
        macros::{command, group, help, hook},
    }, futures::StreamExt, http::Http, model::id::*, model::{channel::{Message, ReactionType},
    gateway::Ready, id::UserId}};
use serenity::prelude::*;
use tokio::sync::Mutex;
use std::convert::TryFrom;

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Handler;

struct ChannelMap;
struct EmoteMap;
struct ThresholdMap;

impl TypeMapKey for ChannelMap {
    type Value = Arc<RwLock<HashMap<GuildId, ChannelId>>>;
}

impl TypeMapKey for EmoteMap {
    type Value = Arc<RwLock<HashMap<GuildId, ReactionType>>>;
}

impl TypeMapKey for ThresholdMap {
    type Value = Arc<RwLock<HashMap<GuildId, u32>>>;
}

use futures::executor;
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let c_ctx = ctx.clone();
        let c_msg = msg.clone();
        let mut thresh: u32 = 5;
        let collector = msg.await_reactions(&ctx)
            .collect_limit({
                let data_read = c_ctx.data.read().await;
                let map = data_read.get::<ThresholdMap>().unwrap();
                let read_map = map.read().await;

                thresh = match read_map.get::<GuildId>(&msg.guild_id.unwrap()) {
                    Some(a) => *a,
                    _ => 5,
                };
                thresh
            })
            .filter(move |x| {
                let res = async {
                    let data_read = c_ctx.data.read().await;
                    let map = data_read.get::<EmoteMap>().unwrap();
                    let read_map = map.read().await;

                    let my_emote = read_map.get::<GuildId>(&msg.guild_id.unwrap())
                        .get_or_insert(&ReactionType::try_from("⭐").unwrap())
                        .clone();

                    x.as_ref().emoji.as_data() == my_emote.as_data()
                };

                executor::block_on(res)
            })
            .await;
        let collected: Vec<_> = collector.then(|reaction| async move {
            reaction
        }).collect().await;
        if collected.len() >= (thresh as usize) {
            let read_data = &ctx.data.read().await;
            let map_lock = read_data.get::<ChannelMap>()
                .expect("Expected channel map")
                .clone();
            let map = map_lock.read().await;
            let starboard = match map.get(&c_msg.guild_id.unwrap()) {
                Some(a) => a,
                _ => &ChannelId(0),
            };
            let chan = ctx.cache.guild_channel(starboard).await.unwrap();
            // build the message
            // let mut starred = format!("----------------------\n{}:\n> {}", &msg.author.name, &msg.content);
            // for embed in &msg.embeds {
            //     starred.push_str(format!("\n{}", embed.url.as_deref().unwrap()).as_str());
            // }
            // starred.push_str(format!("\n{}\n----------------------", &msg.link()).as_str());
            // let _ = chan.say(ctx.http, &starred).await;
            let _ = chan.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("Jump to the message.").url(&c_msg.link())
                    .author(|a| {
                        a.icon_url(&c_msg.author.face()).name(&c_msg.author.name)
                    })
                    .color(Colour::GOLD)
                    .timestamp(&c_msg.timestamp)
                    .description(&c_msg.content);

                    if c_msg.embeds.len() > 0 {
                        e.image(&c_msg.embeds.get(0).unwrap().url.as_ref().unwrap());
                    } else if c_msg.attachments.len() > 0 {
                        e.image(&c_msg.attachments.get(0).unwrap().url);
                    }

                    e
                });

                m
            })
            .await;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(about, channel, emote, threshold)]
struct General;

// The framework provides two built-in help commands for you to use.
// But you can also make your own customized help command that forwards
// to the behaviour of either of them.
#[help]
// This replaces the information that a user can pass
// a command-name as argument to gain specific information about it.
#[individual_command_tip =
"Hello! こんにちは！Hola! Bonjour! 您好!\n\
If you want more information about a specific command, just pass the command as argument."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name
// and commands. If the distance is lower than or equal the set distance,
// it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate
// how deeply an item is indented.
// The default value is "-", it will be changed to "+".
#[indention_prefix = "+"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip_{dm, guild}` aren't specified.
// If you pass in a value, it will be displayed instead.
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}
#[hook]
async fn before(ctx: &Context, msg: &Message, command_name: &str) -> bool {
    println!("Got command '{}' by user '{}'", command_name, msg.author.name);

    // Increment the number of times this command has been run once. If
    // the command's name does not exist in the counter, add a default
    // value of 0.
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true // if `before` returns false, command processing doesn't happen.
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => {
            let res = format!("Command '{}' returned error {:?}", command_name, why);
            let _ = _msg.reply(&_ctx.http, format!("{}", res));

            println!("{}", res)
        },
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(duration) = error {
        let _ = msg
            .channel_id
            .say(&ctx.http, &format!("Try this again in {} seconds.", duration.as_secs()))
            .await;
    }
}

use serenity::{futures::future::BoxFuture, FutureExt};
fn _dispatch_error_no_macro<'fut>(ctx: &'fut mut Context, msg: &'fut Message, error: DispatchError) -> BoxFuture<'fut, ()> {
    async move {
        if let DispatchError::Ratelimited(duration) = error {
            let _ = msg
                .channel_id
                .say(&ctx.http, &format!("Try this again in {} seconds.", duration.as_secs()))
                .await;
        };
    }.boxed()
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Could not read .env");
    // Configure the client with your Discord bot token in the environment.
    let token = dotenv::var("DISCORD_TOKEN").expect(
        "Expected a token in the environment",
    );

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        },
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c
                   .with_whitespace(true)
                   .on_mention(Some(bot_id))
                   .prefix(".")
                   // In this case, if "," would be first, a message would never
                   // be delimited at ", ", forcing you to trim your arguments if you
                   // want to avoid whitespaces at the start of each.
                   .delimiters(vec![", ", ","])
                   // Sets the bot's owners. These will be used for commands that
                   // are owners only.
                   .owners(owners))

    // Set a function to be called prior to each command execution. This
    // provides the context of the command, the message that was received,
    // and the full name of the command that will be called.
        .before(before)
    // Similar to `before`, except will be called directly _after_
    // command execution.
        .after(after)
    // Set a function that's called whenever an attempted command-call's
    // command could not be found.
        .unrecognised_command(unknown_command)
    // Set a function that's called whenever a command's execution didn't complete for one
    // reason or another. For example, when a user has exceeded a rate-limit or a command
    // can only be performed by the bot owner.
        .on_dispatch_error(dispatch_error)
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<ChannelMap>(Arc::new(RwLock::new(HashMap::default())));
        data.insert::<EmoteMap>(Arc::new(RwLock::new(HashMap::default())));
        data.insert::<ThresholdMap>(Arc::new(RwLock::new(HashMap::default())));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "A bot to save memorable posts").await?;

    Ok(())
}

#[command]
#[required_permissions("ADMINISTRATOR")]
async fn channel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let new_channel = args.single::<u64>()?;

    let map_lock = {
        let data_read = ctx.data.read().await;

        data_read.get::<ChannelMap>().expect("Expected Channel Map in TypeMap").clone()
    };
    {
        let mut map = map_lock.write().await;
        let starboard = map.entry(msg.guild_id.unwrap()).or_insert(ChannelId(0));
        *starboard = ChannelId(new_channel);
        msg.channel_id.say(&ctx.http, &format!("Set the channel to {:?}", ctx.cache.guild_channel(*starboard)
            .await.unwrap().name)).await?;
    }

    Ok(())
}

#[command]
#[required_permissions("ADMINISTRATOR")]
async fn emote(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let react = args.single::<String>()?;

    let map = {
        let data_read = ctx.data.read().await;

        data_read.get::<EmoteMap>().expect("Expected Emote Map in TypeMap").clone()
    };
    {
        let mut write_map = map.write().await;
        let pogsire = write_map.entry(msg.guild_id.unwrap())
            .or_insert(ReactionType::try_from("<:pogsire:727216449432846476>").unwrap());
        *pogsire = ReactionType::try_from(react.as_str()).unwrap();
        msg.channel_id.say(&ctx.http, &format!("Set the emote to {}", pogsire.as_data().as_str())).await?;
    }

    Ok(())
}

#[command]
#[required_permissions("ADMINISTRATOR")]
async fn threshold(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let num = args.single::<u32>()?;

    let map = {
        let data_read = ctx.data.read().await;

        data_read.get::<ThresholdMap>().expect("Expected Threshold Map in TypeMap").clone()
    };
    {
        let mut write_map = map.write().await;
        let thres = write_map.entry(msg.guild_id.unwrap())
            .or_insert(5);
        *thres = num;
        msg.channel_id.say(&ctx.http, &format!("Set the threshold to {} reactions", num)).await?;
    }

    Ok(())
}
