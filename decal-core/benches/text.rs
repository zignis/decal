use crate::utils::setup_engine;
use criterion::Criterion;
use decal::prelude::*;
use std::{
    hint::black_box,
    time::Duration,
};

pub fn bench(c: &mut Criterion) {
    let mut engine = setup_engine();

    let mut group = c.benchmark_group("text");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(15));

    let mut scene = decal! {
        Column {
            Text("
    Far far away, behind the word mountains, far from the countries Vokalia and Consonantia, \
    there live the blind texts. Separated they live in Bookmarksgrove right at the coast of \
    the Semantics, a large language ocean. A small river named Duden flows by their place and \
    supplies it with the necessary regelialia. It is a paradisematic country, in which roasted \
    parts of sentences fly into your mouth. Even the all-powerful Pointing has no control about \
    the blind texts it is an almost unorthographic life One day however a small line of blind \
    text by the name of Lorem Ipsum decided to leave for the far World of Grammar. The Big Oxmox \
    advised her not to do so, because there were thousands of bad Commas, wild Question Marks \
    and devious Semikoli, but the Little Blind Text didn’t listen. She packed her seven versalia, \
    put her initial into the belt and made herself on the way. When she reached the first hills \
    of the Italic Mountains, she had a last view back on the skyline of her hometown Bookmarksgrove, \
    the headline of Alphabet Village and the subline of her own road, the Line Lane."
            )
                .font_size(16.0)
                .width(120.0)
        }
    };

    group.bench_function("english_text", |b| {
        b.iter(|| {
            black_box(
                engine
                    .rasterize(&mut scene, &RasterizeOptions::default())
                    .unwrap(),
            );
        })
    });

    // Japanese

    let mut scene = decal! {
        Column {
            Text(
    "じんるい しゃかい の すべて の こうせいいん の こゆう の そんげん と びょうどう で ゆずる こと の \
    できない けんり と を しょうにんする こと は、 せかい における じゆう、 まさよし および へいわ の きそ で ある ので、 \
    じんけん の むし および けいぶ が、 じんるい の りょうしん を ふみにじった やばん こうい を もたらし、 \
    げんろん および しんこう の じゆう が うけられ、 きょうふ および けつぼう の ない せかい の とうらい が、 \
    いっぱん の ひとびと の さいこう の がんぼう として せんげんされた ので、 にんげん が せんせい と あっぱく \
    と にたいする さいご の しゅだん として はんぎゃく に うったえる こと が ない よう に する ために は、 \
    ほう の しはい によって じんけん を ほごする こと が かんよう で ある ので、 しょこくかん の ゆうこう \
    かんけい の はってん を そくしんする こと が かんよう で ある ので、 こくさい れんごう の しょ こくみん は、 \
    こくれん けんしょう において、 きほんてき じんけん、 にんげん の そんげん および かち ならびに だんじょ の \
    どうけん について の しんねん を さい かくにんし、 かつ、 いっそう おおきな じゆう の うち で しゃかいてき \
    しんぽ と せいかつ すいじゅん の こうじょう と を そくしんする こと を けついした ので、 かめいこく は、 \
    こくさい れんごう と きょうりょくして、 じんけん および きほんてき じゆう の ふへんてき な そんちょう および \
    じゅんしゅ の そくしん を たっせいする こと を せいやくした ので、 これら の けんり および じゆう にたいする \
    きょうつう の りかい は、 この せいやく を かんぜん に する ために もっとも じゅうよう で ある ので、 よ って、 \
    ここ に、 こくれん そうかい は、 しゃかい の かく こじん および かく きかん が、 この せかい じんけん せんげん \
    を つねに ねんとう に おき ながら、 かめいこく じしん の じんみん の ま に も、 また、 かめいこく の かんかつか \
    に ある ちいき の じんみん の ま に も、 これら の けんり と じゆう と の そんちょう を しどう および きょういく \
    によって そくしんする こと ならびに それら の ふへんてき そち によって かくほする こと に どりょくする ように、 \
    すべて の じんみん と すべて の くに と が たっせいす べき きょうつう の きじゅん として、 この じんけん \
    せんげん を こうふする"
            )
                .font_family("Noto Sans JP")
                .font_size(16.0)
                .width(120.0)
        }
    };

    group.bench_function("japanese_text", |b| {
        b.iter(|| {
            black_box(
                engine
                    .rasterize(&mut scene, &RasterizeOptions::default())
                    .unwrap(),
            );
        })
    });

    // complex text

    let mut scene = decal! {
        Column {
            Text(text! {
                ("Large title: ", { size: 48.0, weight: FontWeight::Bold }),
                ("Subtitle ", { size: 24.0, style: FontStyle::Italic }),
                ("with mixed spans.\n", { size: 24.0 })
            })
                .font_family("PhantomFont")
                .width(600.0)

            Text(text! {
                ("Paragraph 1. ", { color: Paint::color(rgb(0xff0000)) }),
                "This is a massive block of text with ",
                ("inline bold styling", { weight: FontWeight::Bold }),
                " and ",
                ("italic styling", { style: FontStyle::Italic }),
                ". Far far away, behind the word mountains, far from the countries Vokalia and Consonantia, there live the blind texts. ".repeat(10).as_str(),
                ("End of paragraph 1.\n", { color: Paint::color(rgb(0x00ff00)) })
            })
                .size(16.0)
                .width(600.0)

            Text(text! {
                ("Paragraph 2. ", { color: Paint::color(rgb(0x0000ff)) }),
                "Far far away, behind the word mountains, far from the countries Vokalia and Consonantia, there live the blind texts. ".repeat(10).as_str()
            })
                .font_family("Inconsolata")
                .size(16.0)
                .width(600.0)
        }
    };

    group.bench_function("complex_text_spans", |b| {
        b.iter(|| {
            black_box(
                engine
                    .rasterize(&mut scene, &RasterizeOptions::default())
                    .unwrap(),
            );
        })
    });

    group.finish();
}
