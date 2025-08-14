use crate::state::AppState;
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

        #[allow(clippy::collapsible_if)]
        if flex.add(item(), Button::new("Send")).clicked()
            || (enter_pressed && input.lost_focus()) && !state.chat.message_input.is_empty()
        {
            if let Some(account) = &state.twitch_account
                && let Some(channel) = &account.channel
            {
                let message = state.chat.message_input.trim().to_string();
                let token = account.token.clone();
                let client = account.client.clone();
                let broadcaster_id = channel.broadcaster_id.clone();
                let user_id = account.token.user_id.clone();

                tokio::spawn(async move {
                    let _ = client
                        .send_chat_message(broadcaster_id, user_id, &*message, &token)
                        .await;
                });

                state.chat.message_input.clear();
                input.request_focus();
            }
        }
    });
}
