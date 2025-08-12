use eframe::{
    CreationContext,
    egui::{FontData, FontDefinitions, FontFamily},
};

pub fn setup_style(cctx: &CreationContext) {
    setup_fonts(cctx);
}

fn setup_fonts(cctx: &CreationContext) {
    let noto_sans = include_bytes!("../../assets/NotoSans-Regular.ttf");
    let noto_emoji = include_bytes!("../../assets/NotoColorEmoji-Regular.ttf");

    let noto_cjk_hk = include_bytes!("../../assets/NotoSansCJKhk-VF.otf");
    let noto_cjk_jp = include_bytes!("../../assets/NotoSansCJKjp-VF.otf");
    let noto_cjk_kr = include_bytes!("../../assets/NotoSansCJKkr-VF.otf");
    let noto_cjk_sc = include_bytes!("../../assets/NotoSansCJKsc-VF.otf");
    let noto_cjk_tc = include_bytes!("../../assets/NotoSansCJKtc-VF.otf");

    let mut fonts = FontDefinitions::default();

    fonts
        .font_data
        .insert("NotoSans".to_owned(), FontData::from_static(noto_sans).into());
    fonts
        .font_data
        .insert("NotoEmoji".to_owned(), FontData::from_static(noto_emoji).into());
    fonts
        .font_data
        .insert("NotoSansCJKhk".to_owned(), FontData::from_static(noto_cjk_hk).into());
    fonts
        .font_data
        .insert("NotoSansCJKjp".to_owned(), FontData::from_static(noto_cjk_jp).into());
    fonts
        .font_data
        .insert("NotoSansCJKkr".to_owned(), FontData::from_static(noto_cjk_kr).into());
    fonts
        .font_data
        .insert("NotoSansCJKsc".to_owned(), FontData::from_static(noto_cjk_sc).into());
    fonts
        .font_data
        .insert("NotoSansCJKtc".to_owned(), FontData::from_static(noto_cjk_tc).into());

    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();

    proportional.insert(0, "NotoSansCJKhk".to_owned());
    proportional.insert(0, "NotoSansCJKjp".to_owned());
    proportional.insert(0, "NotoSansCJKkr".to_owned());
    proportional.insert(0, "NotoSansCJKsc".to_owned());
    proportional.insert(0, "NotoSansCJKtc".to_owned());

    proportional.push("NotoSans".to_owned());

    let monospace = fonts.families.entry(FontFamily::Monospace).or_default();
    monospace.insert(0, "NotoEmoji".to_owned());

    cctx.egui_ctx.set_fonts(fonts);
}
