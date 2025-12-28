use std::{sync::mpsc, time::Duration};

use chrono::Local;
use eframe::egui::{self, Color32, Popup, RichText, Ui};
use linkify::LinkFinder;
use twitch_api::helix::channels::ChannelInformation;
use twitch_irc::message::PrivmsgMessage;

use crate::{
    twitch::{
        api::{
            twitch_ban_user, twitch_delete_message, twitch_mod_user, twitch_shoutout_user, twitch_timeout_user,
            twitch_unban_user, twitch_unmod_user, twitch_vip_user,
        },
        types::{PrivmsgMessageExt, TwitchAccount, TwitchEvent},
    },
    ui::state::AppStateDiff,
};

pub fn render_chat_message(
    ui: &mut Ui,
    message: &PrivmsgMessage,
    diff_tx: &mpsc::Sender<AppStateDiff>,
    account: &Option<TwitchAccount>,
    channel: &Option<ChannelInformation>,
    chat_user_query: &mut String,
    logged_in_user_name: Option<String>,
    show_timestamps: bool,
) {
    ui.horizontal_wrapped(|ui| {
        ui.style_mut().spacing.item_spacing.x = 0.0;

        let mut message = message.to_owned();
        message.message_text = message.message_text.trim().to_owned();

        // timestamp
        if show_timestamps {
            ui.label(RichText::new(
                message
                    .server_timestamp
                    .with_timezone(&Local)
                    .format("%H:%M:%S ")
                    .to_string(),
            ));
        }

        // ping?
        if let Some(logged_in_user_name) = &logged_in_user_name
            && message.message_text.contains(&logged_in_user_name.to_lowercase())
        {
            ui.label(RichText::new("PING ").color(Color32::PURPLE));
        }

        // pseudo badge aka deleted/timouted/banned
        if message.is_banned() {
            ui.label(RichText::new("[BANNED] ").color(Color32::RED));
        } else if message.is_timeouted() {
            ui.label(RichText::new("[TIMED OUT] ").color(Color32::RED));
        } else if message.is_deleted() {
            ui.label(RichText::new("[DELETED] ").color(Color32::RED));
        }

        // badges
        if message.is_first_message() {
            ui.label(RichText::new("FIRST ").color(Color32::YELLOW));
        }
        if message.is_by_broadcaster() {
            ui.label(RichText::new("CAST ").color(Color32::RED));
        } else if message.is_by_lead_mod() {
            ui.label(RichText::new("LMOD ").color(Color32::DARK_GREEN));
        } else if message.is_by_mod() {
            ui.label(RichText::new("MOD ").color(Color32::GREEN));
        } else if message.is_by_vip() {
            ui.label(RichText::new("VIP ").color(Color32::MAGENTA));
        } else if message.is_by_subscriber() {
            ui.label(RichText::new("SUB ").color(Color32::CYAN));
        }

        // sender
        let sender = ui
            .label(
                RichText::new(format!("{}: ", message.sender.name)).color(if let Some(color) = message.name_color {
                    Color32::from_rgb(color.r, color.g, color.b)
                } else {
                    Color32::WHITE
                }),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand);

        // sender menu
        Popup::menu(&sender).show(|ui| {
            ui.set_width(200.0);

            ui.colored_label(egui::Color32::WHITE, format!("User: {}", message.sender.name));
            ui.separator();

            if ui.button("History").clicked() {
                *chat_user_query = message.sender.name.clone();
                ui.close();
            }

            if ui.button("Copy Username").clicked() {
                ui.ctx().copy_text(message.sender.name.clone());
                ui.close();
            }

            if ui.button("Copy Message").clicked() {
                ui.ctx().copy_text(message.message_text.clone());
                ui.close();
            }

            let Some(account) = account else {
                return;
            };
            let Some(channel) = channel else {
                return;
            };

            ui.separator();

            if !message.is_deleted()
                && !message.is_timeouted()
                && !message.is_banned()
                && ui.button("Delete Message").clicked()
            {
                twitch_delete_message(diff_tx, account, channel, &message.message_id);
                ui.close();
            }

            if message.is_by_broadcaster() {
                return;
            }

            if !message.is_timeouted() && !message.is_banned() && ui.button("Delete All Messages").clicked() {
                twitch_timeout_user(diff_tx, account, channel, &message.sender.name, Duration::from_secs(1));
                ui.close();
            }

            if !message.is_timeouted() && !message.is_banned() {
                ui.menu_button("Timeout", |ui| {
                    if ui.button("30 seconds").clicked() {
                        twitch_timeout_user(diff_tx, account, channel, &message.sender.name, Duration::from_secs(30));
                        ui.close();
                    }

                    if ui.button("1 minute").clicked() {
                        twitch_timeout_user(diff_tx, account, channel, &message.sender.name, Duration::from_secs(60));
                        ui.close();
                    }

                    if ui.button("5 minutes").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(5 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("10 minutes").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(10 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("15 minutes").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(15 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("30 minutes").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(30 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("45 minutes").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(45 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("1 hour").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(60 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("2 hours").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(2 * 60 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("3 hours").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(3 * 60 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("6 hours").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(6 * 60 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("9 hours").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(9 * 60 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("12 hours").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(12 * 60 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("1 day").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(24 * 60 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("2 days").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(2 * 24 * 60 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("3 days").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(3 * 24 * 60 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("1 week").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(7 * 24 * 60 * 60),
                        );
                        ui.close();
                    }

                    if ui.button("2 weeks").clicked() {
                        twitch_timeout_user(
                            diff_tx,
                            account,
                            channel,
                            &message.sender.name,
                            Duration::from_secs(14 * 24 * 60 * 60),
                        );
                        ui.close();
                    }
                });
            }

            if message.is_timeouted() && !message.is_banned() && ui.button("Untimeout").clicked() {
                twitch_unban_user(diff_tx, account, channel, &message.sender.name);
                ui.close();
            }

            if !message.is_banned() && ui.button("Ban").clicked() {
                twitch_ban_user(diff_tx, account, channel, &message.sender.name);
                ui.close();
            }

            if message.is_banned() && ui.button("Unban").clicked() {
                twitch_unban_user(diff_tx, account, channel, &message.sender.name);
                ui.close();
            }

            ui.separator();

            if ui.button("Shoutout").clicked() {
                twitch_shoutout_user(diff_tx, account, channel, &message.sender.name);
                ui.close();
            }

            if !message.is_by_vip() && ui.button("Make VIP").clicked() {
                twitch_vip_user(diff_tx, account, channel, &message.sender.name);
                ui.close();
            }

            if message.is_by_vip() && ui.button("Remove VIP").clicked() {
                twitch_vip_user(diff_tx, account, channel, &message.sender.name);
                ui.close();
            }

            if !message.is_by_mod() && ui.button("Make Mod").clicked() {
                twitch_mod_user(diff_tx, account, channel, &message.sender.name);
                ui.close();
            }

            if message.is_by_mod() && ui.button("Remove Mod").clicked() {
                twitch_unmod_user(diff_tx, account, channel, &message.sender.name);
                ui.close();
            }
        });

        // NOTE: if this has to be extended in anyway, parse the message before into [Text(String),
        // Link(String), Emote(Img), Text(Text)] etc and then match on the kind to display.

        // message text
        let links: Vec<_> = LinkFinder::new().links(&message.message_text).collect();

        if links.is_empty() {
            ui.label(RichText::new(&message.message_text).color(egui::Color32::WHITE));
        } else {
            let mut last_end = 0;
            for link in &links {
                let start = link.start();
                let end = link.end();

                // text before link
                if start > last_end {
                    ui.label(RichText::new(&message.message_text[last_end..start]).color(egui::Color32::WHITE));
                }

                // link
                ui.hyperlink(link.as_str());

                last_end = end;
            }

            // text after last link
            if last_end < message.message_text.len() {
                ui.label(RichText::new(&message.message_text[last_end..]).color(egui::Color32::WHITE));
            }
        }
    });
}

pub fn render_event_for_log(buffer: &mut String, event: &TwitchEvent) {
    match event {
        TwitchEvent::Join(join) => {
            buffer.push_str(&format!("Joined channel {}.\n", join.channel_login));
        }
        TwitchEvent::Notice(notice) => {
            buffer.push_str(&format!("{}\n", notice.message_text.trim()));
        }
        TwitchEvent::Privmsg(msg) => {
            let badge1 = if msg.is_banned() {
                "[BANNED] "
            } else if msg.is_timeouted() {
                "[TIMED OUT] "
            } else if msg.is_deleted() {
                "[DELETED] "
            } else {
                ""
            };

            let badge2 = if msg.is_first_message() {
                "FIRST "
            } else if msg.is_by_broadcaster() {
                "CAST "
            } else if msg.is_by_lead_mod() {
                "LMOD "
            } else if msg.is_by_mod() {
                "MOD "
            } else if msg.is_by_vip() {
                "VIP "
            } else if msg.is_by_subscriber() {
                "SUB "
            } else {
                ""
            };

            buffer.push_str(&format!(
                "{} {badge1}{badge2}{}: {}\n",
                msg.server_timestamp.format("%H:%M:%S"),
                msg.sender.name,
                msg.message_text
            ));
        }
        _ => {}
    }
}
