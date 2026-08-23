use base64::{
    Engine as _,
    prelude::BASE64_STANDARD,
};
use decal::prelude::*;

pub fn setup_engine() -> Engine {
    Engine::new(EngineOptions {
        fonts: FontRegistry::new()
            .load_font(
                "Texturina",
                include_bytes!("../tests/fixtures/Texturina.ttf"),
            )
            .load_font(
                "Texturina",
                include_bytes!("../tests/fixtures/Texturina-Italic.ttf"),
            )
            .load_font(
                "Noto Sans JP",
                include_bytes!("../tests/fixtures/NotoSansJP.ttf"),
            )
            .load_font(
                "Inconsolata",
                include_bytes!("../tests/fixtures/Inconsolata.ttf"),
            )
            .default_family("Texturina"),
        ..Default::default()
    })
}

pub fn get_sample_image() -> ImageSource {
    let bytes = include_bytes!("../tests/fixtures/4_dice.png");
    ImageSource::href(format!(
        "data:image/png;base64,{}",
        BASE64_STANDARD.encode(bytes)
    ))
}
