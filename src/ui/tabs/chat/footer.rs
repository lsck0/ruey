use crate::{state::AppState, twitch::api::twitch_send_message};
use eframe::egui::{self, Button, TextEdit, Ui};
use egui_flex::{Flex, item};

pub fn render_chat_footer(ui: &mut Ui, state: &mut AppState) {
    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

    Flex::horizontal().w_full().show(ui, |flex| {
        let input = flex.add(
            item().grow(1.0),
            TextEdit::singleline(&mut state.chat.message_input)
                .hint_text("Chat...")
                .char_limit(255),
        );

        if (flex.add(item(), Button::new("Send")).clicked() || (enter_pressed && input.lost_focus()))
            && !state.chat.message_input.is_empty()
            && let Some(account) = &state.twitch_account
            && let Some(channel) = &state.connected_channel_info
        {
            twitch_send_message(&state.diff_tx, account, channel, &state.chat.message_input);
            state.chat.message_input.clear();
            input.request_focus();
        }
    });
}
