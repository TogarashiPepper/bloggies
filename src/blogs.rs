use anyhow::Result;
use serenity::all::{
	ChannelId, CommandInteraction, Context, GuildChannel, GuildId, PermissionOverwrite, PermissionOverwriteType,
	Permissions, UserId,
};

pub struct Blogs {
	pub category_id: ChannelId,
	pub channels: Vec<GuildChannel>,
	pub guild_id: GuildId,
	pub user_id: UserId,
}

impl Blogs {
	pub fn channel(&mut self) -> Result<&mut GuildChannel> {
		self.channels
			.iter_mut()
			.find(|channel| channel.topic == Some(self.user_id.to_string()))
			.ok_or_else(|| anyhow::anyhow!("You do not have a blog!"))
	}

	pub fn new(ctx: &Context, interaction: &CommandInteraction) -> Result<Self> {
		let guild_id = interaction
			.guild_id
			.ok_or_else(|| anyhow::anyhow!("Interaction was not sent from a guild!"))?;

		let channels = &ctx
			.cache
			.guild(guild_id)
			.ok_or_else(|| anyhow::anyhow!("Could not find the guild in cache!"))?
			.channels;

		let (&parent_id, _) = channels
			.iter()
			.find(|(_, channel)| channel.name == "Blogs")
			.ok_or_else(|| anyhow::anyhow!("Could not find the blog category!"))?;

		let children: Vec<_> = channels
			.values()
			.filter(|channel| channel.parent_id == Some(parent_id))
			.cloned()
			.collect();

		Ok(Self {
			category_id: parent_id,
			channels: children,
			guild_id,
			user_id: interaction.user.id,
		})
	}

	pub fn permissions(&self) -> [PermissionOverwrite; 2] {
		[
			PermissionOverwrite {
				allow: Permissions::SEND_MESSAGES,
				deny: Permissions::empty(),
				kind: PermissionOverwriteType::Member(self.user_id),
			},
			PermissionOverwrite {
				allow: Permissions::empty(),
				deny: Permissions::SEND_MESSAGES,
				kind: PermissionOverwriteType::Role(self.guild_id.everyone_role()),
			},
		]
	}

	pub async fn reorder(&mut self, ctx: &Context) -> Result<()> {
		self.channels.sort_by(|first, second| first.name.cmp(&second.name));

		let channels = self
			.channels
			.iter()
			.enumerate()
			.map(|(index, channel)| (channel.id, index as u64));

		self.guild_id.reorder_channels(ctx, channels).await?;

		Ok(())
	}
}
