use crate::{state::AppState, twitch::events::TwitchEvent, ui::tabs::chat::message::render_chat_message};
use eframe::egui::{ScrollArea, Ui, scroll_area::ScrollSource};

pub fn render_chat_history(ui: &mut Ui, state: &mut AppState) {
    ScrollArea::vertical()
        .max_height(ui.available_height() - 35.0)
        .max_width(ui.available_width() - 5.0)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .scroll_source(ScrollSource {
            drag: false,
            mouse_wheel: true,
            scroll_bar: true,
        })
        .show(ui, |ui| {
            state.events.ui(ui, 50, |ui, _, event| match event {
                TwitchEvent::Join(join) => {
                    ui.label(format!("Joined channel {}.", join.channel_login));
                }
                TwitchEvent::Privmsg(msg) => {
                    render_chat_message(ui, &mut state.chat_user_query, msg);
                }
                _ => {}
            });
        });
}
