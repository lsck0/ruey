use eframe::{
    CreationContext,
    egui::{FontData, FontDefinitions, FontFamily},
};

macro_rules! insert_font {
    ($fonts:expr, $name:expr, $data:expr) => {
        $fonts
            .font_data
            .insert($name.to_owned(), FontData::from_static($data).into());
    };
}

const NOTO_SANS: &[u8; 629024] = include_bytes!("../../assets/fonts/NotoSans-Regular.ttf");
const NOTO_EMOJI: &[u8; 24270832] = include_bytes!("../../assets/fonts/NotoColorEmoji-Regular.ttf");
const NOTO_CJK_HK: &[u8; 30727704] = include_bytes!("../../assets/fonts/NotoSansCJKhk-VF.otf");
const NOTO_CJK_JP: &[u8; 30766960] = include_bytes!("../../assets/fonts/NotoSansCJKjp-VF.otf");
const NOTO_CJK_KR: &[u8; 30733192] = include_bytes!("../../assets/fonts/NotoSansCJKkr-VF.otf");
const NOTO_CJK_SC: &[u8; 30737452] = include_bytes!("../../assets/fonts/NotoSansCJKsc-VF.otf");
const NOTO_CJK_TC: &[u8; 30735992] = include_bytes!("../../assets/fonts/NotoSansCJKtc-VF.otf");
const NOTO_SANS_NAME: &str = "NotoSans";
const NOTO_EMOJI_NAME: &str = "NotoEmoji";
const NOTO_CJK_HK_NAME: &str = "NotoSansCJKhk";
const NOTO_CJK_JP_NAME: &str = "NotoSansCJKjp";
const NOTO_CJK_KR_NAME: &str = "NotoSansCJKkr";
const NOTO_CJK_SC_NAME: &str = "NotoSansCJKsc";
const NOTO_CJK_TC_NAME: &str = "NotoSansCJKtc";

pub fn load_fonts(cctx: &CreationContext) {
    let mut fonts = FontDefinitions::default();
    insert_font!(fonts, NOTO_SANS_NAME, NOTO_SANS);
    insert_font!(fonts, NOTO_EMOJI_NAME, NOTO_EMOJI);
    insert_font!(fonts, NOTO_CJK_HK_NAME, NOTO_CJK_HK);
    insert_font!(fonts, NOTO_CJK_JP_NAME, NOTO_CJK_JP);
    insert_font!(fonts, NOTO_CJK_KR_NAME, NOTO_CJK_KR);
    insert_font!(fonts, NOTO_CJK_SC_NAME, NOTO_CJK_SC);
    insert_font!(fonts, NOTO_CJK_TC_NAME, NOTO_CJK_TC);

    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
    proportional.push(NOTO_SANS_NAME.to_owned());
    proportional.push(NOTO_CJK_HK_NAME.to_owned());
    proportional.push(NOTO_CJK_JP_NAME.to_owned());
    proportional.push(NOTO_CJK_KR_NAME.to_owned());
    proportional.push(NOTO_CJK_SC_NAME.to_owned());
    proportional.push(NOTO_CJK_TC_NAME.to_owned());

    let monospace = fonts.families.entry(FontFamily::Monospace).or_default();
    monospace.push(NOTO_EMOJI_NAME.to_owned());

    cctx.egui_ctx.set_fonts(fonts);

    // // global style
    // cctx.egui_ctx.all_styles_mut(|style| {
    //     style.text_styles = [
    //         (TextStyle::Heading, FontId::new(25.0, Proportional)),
    //         (TextStyle::Name("Heading2".into()), FontId::new(22.0, Proportional)),
    //         (
    //             TextStyle::Name("ContextHeading".into()),
    //             FontId::new(19.0, Proportional),
    //         ),
    //         (TextStyle::Body, FontId::new(16.0, Proportional)),
    //         (TextStyle::Monospace, FontId::new(12.0, Monospace)),
    //         (TextStyle::Button, FontId::new(12.0, Proportional)),
    //         (TextStyle::Small, FontId::new(8.0, Proportional)),
    //     ]
    //     .into();
    // });
    //
    // // light mode
    // cctx.egui_ctx.style_mut_of(Theme::Light, |style| {
    //     style.visuals.hyperlink_color = Color32::from_rgb(18, 180, 85);
    //     style.visuals.text_cursor.stroke.color = Color32::from_rgb(28, 92, 48);
    //     style.visuals.selection = Selection {
    //         bg_fill: Color32::from_rgb(157, 218, 169),
    //         stroke: Stroke::new(1.0, Color32::from_rgb(28, 92, 48)),
    //     };
    // });
    //
    // // dark mode
    // cctx.egui_ctx.style_mut_of(Theme::Dark, |style| {
    //     style.visuals.hyperlink_color = Color32::from_rgb(202, 135, 227);
    //     style.visuals.text_cursor.stroke.color = Color32::from_rgb(234, 208, 244);
    //     style.visuals.selection = Selection {
    //         bg_fill: Color32::from_rgb(105, 67, 119),
    //         stroke: Stroke::new(1.0, Color32::from_rgb(234, 208, 244)),
    //     };
    // });
}
