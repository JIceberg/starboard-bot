use dotenv;
use std::{collections::{HashMap, HashSet}, env, fmt::Write, sync::Arc};
use serenity::{async_trait, client::bridge::gateway::{ShardId, ShardManager}, collector::MessageCollectorBuilder, collector::ReactionCollector, futures::StreamExt, framework::standard::{
        Args, CheckResult, CommandOptions, CommandResult, CommandGroup,
        DispatchError, HelpOptions, help_commands, StandardFramework,
        macros::{command, group, help, check, hook},
    }, http::Http, model::channel::Reaction, model::{channel::{Channel, Message, MessageReference, ReactionType},
    gateway::Ready, id::UserId, permissions::Permissions}, utils::{content_safe, ContentSafeOptions}};
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

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // if let Some(reaction) = msg.await_reaction(&ctx).message_id(msg.id).await {
        //     let emoji = reaction.as_ref();
        //     if emoji.as_inner_ref().emoji == ReactionType::try_from("<:pogsire:727216449432846476>").unwrap() {
        //         let _ = msg.reply(&ctx.http, "Poggers").await;
        //     }
        // }
        let collector = msg.await_reactions(&ctx)
            .collect_limit(10)
            .filter(|x| x.as_ref().emoji == ReactionType::try_from("<:pogsire:727216449432846476>").unwrap())
            .await;
        let collected: Vec<_> = collector.then(|reaction| async move {
            reaction
        }).collect().await;
        if collected.len() >= 10 {
            let _ = msg.reply(&ctx.http, "Poggers").await;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(about)]
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
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    println!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("Message is not a command '{}'", msg.content);
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

// You can construct a hook without the use of a macro, too.
// This requires some boilerplate though and the following additional import.
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
    //
    // You can not use this to determine whether a command should be
    // executed. Instead, the `#[check]` macro gives you this functionality.
    //
    // **Note**: Async closures are unstable, you may use them in your
    // application if you are fine using nightly Rust.
    // If not, we need to provide the function identifiers to the
    // hook-functions (before, after, normal, ...).
        .before(before)
    // Similar to `before`, except will be called directly _after_
    // command execution.
        .after(after)
    // Set a function that's called whenever an attempted command-call's
    // command could not be found.
        .unrecognised_command(unknown_command)
    // Set a function that's called whenever a message is not a command.
        .normal_message(normal_message)
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
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "This is a small test-bot! : )").await?;

    Ok(())
}
