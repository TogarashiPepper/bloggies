use std::time::Duration;

use anyhow::Result;
use serenity::all::{
	ActionRowComponent, CommandDataOption, CommandInteraction, Context, CreateActionRow, CreateChannel,
	CreateInputText, CreateInteractionResponse, CreateInteractionResponseMessage, CreateModal, CreateWebhook,
	EditChannel, InputTextStyle, ModalInteractionCollector, Webhook,
};

use crate::blogs::Blogs;

pub async fn claim(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
	let mut blogs = Blogs::new(ctx, interaction)?;

	let builder = EditChannel::new()
		.audit_log_reason("Used blog-claim command")
		.permissions(blogs.permissions());

	blogs.channel()?.edit(ctx, builder).await?;

	let message = CreateInteractionResponseMessage::new().content("Your blog permissions have been fixed!");
	let response = CreateInteractionResponse::Message(message);

	interaction.create_response(ctx, response).await?;

	Ok(())
}

pub async fn create(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
	let mut blogs = Blogs::new(ctx, interaction)?;

	if blogs.channel().is_ok() {
		anyhow::bail!("You already have a blog channel!");
	}

	let builder = CreateChannel::new(&interaction.user.name)
		.audit_log_reason("Used blog-create command")
		.category(blogs.category_id)
		.permissions(blogs.permissions())
		.topic(blogs.user_id.to_string());

	let channel = blogs.guild_id.create_channel(ctx, builder).await?;

	blogs.channels.push(channel);
	blogs.reorder(ctx).await?;

	let message = CreateInteractionResponseMessage::new().content("Your blog channel has been created!");
	let response = CreateInteractionResponse::Message(message);

	interaction.create_response(ctx, response).await?;

	Ok(())
}

pub async fn delete(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
	let mut blogs = Blogs::new(ctx, interaction)?;
	let channel = blogs.channel()?;

	let label = format!("Enter your blog name: {}", channel.name);
	let row = CreateActionRow::InputText(CreateInputText::new(InputTextStyle::Short, label, "id"));

	let modal = CreateModal::new("id", "Confirm Deletion").components(vec![row]);
	let response = CreateInteractionResponse::Modal(modal);

	interaction.create_response(ctx, response).await?;

	let interaction = ModalInteractionCollector::new(&ctx.shard)
		.author_id(interaction.user.id)
		.timeout(Duration::from_mins(1))
		.await
		.ok_or_else(|| anyhow::anyhow!("Modal timeout exceeded!"))?;

	let row = interaction
		.data
		.components
		.first()
		.ok_or_else(|| anyhow::anyhow!("Modal response has no action rows!"))?;

	let input = match row.components.first() {
		Some(ActionRowComponent::InputText(input)) => input.value.as_deref().unwrap_or_default(),
		Some(_) => anyhow::bail!("Component is not an input text!"),
		None => anyhow::bail!("Action row has no components!"),
	};

	if input != channel.name {
		anyhow::bail!("Input ({input}) did not match the blog name for {channel}!");
	}

	ctx.http
		.delete_channel(channel.id, Some("Used blog-delete command"))
		.await?;

	let message = CreateInteractionResponseMessage::new().content("Your blog channel has been deleted!");
	let response = CreateInteractionResponse::Message(message);

	interaction.create_response(ctx, response).await?;

	Ok(())
}

pub async fn rename(ctx: &Context, interaction: &CommandInteraction, options: &[CommandDataOption]) -> Result<()> {
	let mut blogs = Blogs::new(ctx, interaction)?;
	let channel = blogs.channel()?;

	let name = options
		.first()
		.and_then(|option| option.value.as_str())
		.unwrap_or(&interaction.user.name);

	let builder = EditChannel::new()
		.audit_log_reason("Used blog-rename command")
		.name(name);

	channel.edit(ctx, builder).await?;
	blogs.reorder(ctx).await?;

	let message = CreateInteractionResponseMessage::new().content("Your blog channel has been renamed!");
	let response = CreateInteractionResponse::Message(message);

	interaction.create_response(ctx, response).await?;

	Ok(())
}

pub async fn webhook(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
	let mut blogs = Blogs::new(ctx, interaction)?;
	let channel = blogs.channel()?;

	let builder = CreateWebhook::new("Blog").audit_log_reason("Used blog-webhook command");
	let webhooks = channel.webhooks(ctx).await?;

	let url = match webhooks.iter().flat_map(Webhook::url).next() {
		Some(url) => url,
		None => channel.create_webhook(ctx, builder).await?.url()?,
	};

	let message = CreateInteractionResponseMessage::new()
		.content(format!("Creation of your webhook was successful!\n\n-# {url}"))
		.ephemeral(true);

	let response = CreateInteractionResponse::Message(message);

	interaction.create_response(ctx, response).await?;

	Ok(())
}
