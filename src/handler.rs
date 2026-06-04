use serenity::all::{
	CommandDataOptionValue, Context, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler,
	Interaction, Message, Ready,
};

use crate::commands;

pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		let Interaction::Command(command) = interaction else {
			return;
		};

		let Some(subcommand) = command.data.options.first() else {
			return;
		};

		let CommandDataOptionValue::SubCommand(options) = &subcommand.value else {
			return;
		};

		let result = match command.data.name.as_str() {
			"blog" => match subcommand.name.as_str() {
				"claim" => commands::blog::claim(&ctx, &command).await,
				"create" => commands::blog::create(&ctx, &command).await,
				"delete" => commands::blog::delete(&ctx, &command).await,
				"rename" => commands::blog::rename(&ctx, &command, options).await,
				"webhook" => commands::blog::webhook(&ctx, &command).await,
				name => Err(anyhow::anyhow!("Invalid blog subcommand: {name}")),
			},
			"timeout" => match subcommand.name.as_str() {
				"me" => commands::timeout::me(&ctx, &command, options).await,
				name => Err(anyhow::anyhow!("Invalid timeout subcommand: {name}")),
			},
			name => Err(anyhow::anyhow!("Invalid command: {name}")),
		};

		if let Err(error) = result {
			let message = CreateInteractionResponseMessage::new()
				.content(format!(":no_entry_sign: {error}"))
				.ephemeral(true);

			let response = CreateInteractionResponse::Message(message);

			if command.create_response(&ctx, response).await.is_err() {
				eprintln!("An error occurred: {error}");
			}
		}
	}

	async fn message(&self, ctx: Context, message: Message) {
		if message.author.bot {
			return;
		}

		let topic = message
			.guild_id
			.and_then(|guild_id| ctx.cache.guild(guild_id))
			.and_then(|guild| guild.channels.get(&message.channel_id)?.topic.clone());

		if topic.is_some_and(|topic| !topic.is_empty() && topic != message.author.id.to_string()) {
			message.delete(&ctx).await.ok();
		}
	}

	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is running!", ready.user.name);
	}
}
