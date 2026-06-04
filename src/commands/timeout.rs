use anyhow::Result;
use serenity::all::{
	CommandDataOption, CommandInteraction, Context, CreateAllowedMentions, CreateInteractionResponse,
	CreateInteractionResponseMessage, EditMember, Timestamp,
};

pub async fn me(ctx: &Context, interaction: &CommandInteraction, options: &[CommandDataOption]) -> Result<()> {
	let duration = options
		.first()
		.and_then(|option| option.value.as_i64())
		.ok_or_else(|| anyhow::anyhow!("Duration is not present as an integer!"))?;

	let member = interaction
		.member
		.as_deref()
		.ok_or_else(|| anyhow::anyhow!("Command was not used in a guild!"))?;

	let seconds = Timestamp::now().unix_timestamp() + duration * 3600;
	let timestamp = Timestamp::from_unix_timestamp(seconds)?;

	let builder = EditMember::new()
		.audit_log_reason("Used timeout-me command")
		.disable_communication_until_datetime(timestamp);

	member.guild_id.edit_member(ctx, member.user.id, builder).await?;

	let message = CreateInteractionResponseMessage::new()
		.allowed_mentions(CreateAllowedMentions::new())
		.content(format!("{member} is now muted until <t:{seconds}:t>"));

	let response = CreateInteractionResponse::Message(message);

	interaction.create_response(ctx, response).await?;

	Ok(())
}
