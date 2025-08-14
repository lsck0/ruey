use crate::twitch::events::{PrivmsgMessageExt, TwitchEvent};
use chrono::Local;
use eframe::egui::{self, Color32, Popup, RichText, Ui};
use linkify::LinkFinder;
use twitch_irc::message::PrivmsgMessage;

pub fn render_chat_message(
    ui: &mut Ui,
    chat_user_query: &mut String,
    message: &PrivmsgMessage,
    logged_in_user_name: Option<String>,
) {
    ui.horizontal_wrapped(|ui| {
        ui.style_mut().spacing.item_spacing.x = 0.0;

        let mut message = message.to_owned();
        message.message_text = message.message_text.trim().to_owned();

        // timestamp
        ui.label(RichText::new(
            message
                .server_timestamp
                .with_timezone(&Local)
                .format("%H:%M:%S ")
                .to_string(),
        ));

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
                RichText::new(format!("{}: ", message.sender.name)).color(if message.is_first_message() {
                    Color32::YELLOW
                } else if message.is_by_broadcaster() {
                    Color32::RED
                } else if message.is_by_mod() {
                    Color32::GREEN
                } else if message.is_by_vip() {
                    Color32::MAGENTA
                } else if message.is_by_subscriber() {
                    Color32::CYAN
                } else {
                    Color32::WHITE
                }),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand);

        // sender menu
        Popup::menu(&sender).show(|ui| {
            ui.colored_label(egui::Color32::WHITE, message.sender.name.clone());
            ui.separator();

            // TODO: implement the buttons

            if ui.button("History").clicked() {
                *chat_user_query = message.sender.name.clone();
                ui.close();
            }

            if ui.button("Delete Message").clicked() {
                ui.close();
            }

            if ui.button("Delete All Messages").clicked() {
                ui.close();
            }

            ui.menu_button("Timeout", |ui| {
                if ui.button("1 minute").clicked() {
                    ui.close();
                }
                if ui.button("5 minutes").clicked() {
                    ui.close();
                }
                if ui.button("10 minutes").clicked() {
                    ui.close();
                }
                if ui.button("15 minutes").clicked() {
                    ui.close();
                }
                if ui.button("30 minutes").clicked() {
                    ui.close();
                }
                if ui.button("45 minutes").clicked() {
                    ui.close();
                }
                if ui.button("1 hour").clicked() {
                    ui.close();
                }
                if ui.button("2 hours").clicked() {
                    ui.close();
                }
                if ui.button("3 hours").clicked() {
                    ui.close();
                }
                if ui.button("6 hours").clicked() {
                    ui.close();
                }
                if ui.button("9 hours").clicked() {
                    ui.close();
                }
                if ui.button("12 hours").clicked() {
                    ui.close();
                }
                if ui.button("1 day").clicked() {
                    ui.close();
                }
                if ui.button("2 days").clicked() {
                    ui.close();
                }
                if ui.button("3 days").clicked() {
                    ui.close();
                }
                if ui.button("1 week").clicked() {
                    ui.close();
                }
                if ui.button("1 month").clicked() {
                    ui.close();
                }
            });

            if ui.button("Ban").clicked() {
                ui.close();
            }
        });

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
