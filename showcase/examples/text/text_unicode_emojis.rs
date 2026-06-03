/*******************************************************************************************
*
*   raylib [text] example - unicode emojis
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 4.0
*
*   Example contributed by Vlad Adrian (@demizdor) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Vlad Adrian (@demizdor) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::drawing::RaylibDraw;
use raylib::core::text::RaylibFont;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const EMOJI_PER_WIDTH: usize = 8;
const EMOJI_PER_HEIGHT: usize = 4;
const EMOJI_COUNT: usize = EMOJI_PER_WIDTH * EMOJI_PER_HEIGHT;

//--------------------------------------------------------------------------------------
// Global Variables Definition
//--------------------------------------------------------------------------------------
// Arrays that holds the random emojis
#[derive(Clone, Copy)]
struct Emoji {
    index: i32,   // Index inside `EMOJI_CODEPOINTS`
    message: i32, // Message index
    color: Color, // Emoji color
}

// String containing 180 emoji codepoints separated by a '\0' char
// The C example uses raw \xNN escapes; we paste the same bytes as a byte literal so the layout
// (4-byte UTF-8 emoji + NUL terminator separator, every 5 bytes) is preserved.
const EMOJI_CODEPOINTS: &[u8] = b"\xF0\x9F\x8C\x80\x00\xF0\x9F\x98\x80\x00\xF0\x9F\x98\x82\x00\xF0\x9F\xA4\xA3\x00\xF0\x9F\x98\x83\x00\xF0\x9F\x98\x86\x00\xF0\x9F\x98\x89\x00\xF0\x9F\x98\x8B\x00\xF0\x9F\x98\x8E\x00\xF0\x9F\x98\x8D\x00\xF0\x9F\x98\x98\x00\xF0\x9F\x98\x97\x00\xF0\x9F\x98\x99\x00\xF0\x9F\x98\x9A\x00\xF0\x9F\x99\x82\x00\xF0\x9F\xA4\x97\x00\xF0\x9F\xA4\xA9\x00\xF0\x9F\xA4\x94\x00\xF0\x9F\xA4\xA8\x00\xF0\x9F\x98\x90\x00\xF0\x9F\x98\x91\x00\xF0\x9F\x98\xB6\x00\xF0\x9F\x99\x84\x00\xF0\x9F\x98\x8F\x00\xF0\x9F\x98\xA3\x00\xF0\x9F\x98\xA5\x00\xF0\x9F\x98\xAE\x00\xF0\x9F\xA4\x90\x00\xF0\x9F\x98\xAF\x00\xF0\x9F\x98\xAA\x00\xF0\x9F\x98\xAB\x00\xF0\x9F\x98\xB4\x00\xF0\x9F\x98\x8C\x00\xF0\x9F\x98\x9B\x00\xF0\x9F\x98\x9D\x00\xF0\x9F\xA4\xA4\x00\xF0\x9F\x98\x92\x00\xF0\x9F\x98\x95\x00\xF0\x9F\x99\x83\x00\xF0\x9F\xA4\x91\x00\xF0\x9F\x98\xB2\x00\xF0\x9F\x99\x81\x00\xF0\x9F\x98\x96\x00\xF0\x9F\x98\x9E\x00\xF0\x9F\x98\x9F\x00\xF0\x9F\x98\xA4\x00\xF0\x9F\x98\xA2\x00\xF0\x9F\x98\xAD\x00\xF0\x9F\x98\xA6\x00\xF0\x9F\x98\xA9\x00\xF0\x9F\xA4\xAF\x00\xF0\x9F\x98\xAC\x00\xF0\x9F\x98\xB0\x00\xF0\x9F\x98\xB1\x00\xF0\x9F\x98\xB3\x00\xF0\x9F\xA4\xAA\x00\xF0\x9F\x98\xB5\x00\xF0\x9F\x98\xA1\x00\xF0\x9F\x98\xA0\x00\xF0\x9F\xA4\xAC\x00\xF0\x9F\x98\xB7\x00\xF0\x9F\xA4\x92\x00\xF0\x9F\xA4\x95\x00\xF0\x9F\xA4\xA2\x00\xF0\x9F\xA4\xAE\x00\xF0\x9F\xA4\xA7\x00\xF0\x9F\x98\x87\x00\xF0\x9F\xA4\xA0\x00\xF0\x9F\xA4\xAB\x00\xF0\x9F\xA4\xAD\x00\xF0\x9F\xA7\x90\x00\xF0\x9F\xA4\x93\x00\xF0\x9F\x98\x88\x00\xF0\x9F\x91\xBF\x00\xF0\x9F\x91\xB9\x00\xF0\x9F\x91\xBA\x00\xF0\x9F\x92\x80\x00\xF0\x9F\x91\xBB\x00\xF0\x9F\x91\xBD\x00\xF0\x9F\x91\xBE\x00\xF0\x9F\xA4\x96\x00\xF0\x9F\x92\xA9\x00\xF0\x9F\x98\xBA\x00\xF0\x9F\x98\xB8\x00\xF0\x9F\x98\xB9\x00\xF0\x9F\x98\xBB\x00\xF0\x9F\x98\xBD\x00\xF0\x9F\x99\x80\x00\xF0\x9F\x98\xBF\x00\xF0\x9F\x8C\xBE\x00\xF0\x9F\x8C\xBF\x00\xF0\x9F\x8D\x80\x00\xF0\x9F\x8D\x83\x00\xF0\x9F\x8D\x87\x00\xF0\x9F\x8D\x93\x00\xF0\x9F\xA5\x9D\x00\xF0\x9F\x8D\x85\x00\xF0\x9F\xA5\xA5\x00\xF0\x9F\xA5\x91\x00\xF0\x9F\x8D\x86\x00\xF0\x9F\xA5\x94\x00\xF0\x9F\xA5\x95\x00\xF0\x9F\x8C\xBD\x00\xF0\x9F\x8C\xB6\x00\xF0\x9F\xA5\x92\x00\xF0\x9F\xA5\xA6\x00\xF0\x9F\x8D\x84\x00\xF0\x9F\xA5\x9C\x00\xF0\x9F\x8C\xB0\x00\xF0\x9F\x8D\x9E\x00\xF0\x9F\xA5\x90\x00\xF0\x9F\xA5\x96\x00\xF0\x9F\xA5\xA8\x00\xF0\x9F\xA5\x9E\x00\xF0\x9F\xA7\x80\x00\xF0\x9F\x8D\x96\x00\xF0\x9F\x8D\x97\x00\xF0\x9F\xA5\xA9\x00\xF0\x9F\xA5\x93\x00\xF0\x9F\x8D\x94\x00\xF0\x9F\x8D\x9F\x00\xF0\x9F\x8D\x95\x00\xF0\x9F\x8C\xAD\x00\xF0\x9F\xA5\xAA\x00\xF0\x9F\x8C\xAE\x00\xF0\x9F\x8C\xAF\x00\xF0\x9F\xA5\x99\x00\xF0\x9F\xA5\x9A\x00\xF0\x9F\x8D\xB3\x00\xF0\x9F\xA5\x98\x00\xF0\x9F\x8D\xB2\x00\xF0\x9F\xA5\xA3\x00\xF0\x9F\xA5\x97\x00\xF0\x9F\x8D\xBF\x00\xF0\x9F\xA5\xAB\x00\xF0\x9F\x8D\xB1\x00\xF0\x9F\x8D\x98\x00\xF0\x9F\x8D\x9D\x00\xF0\x9F\x8D\xA0\x00\xF0\x9F\x8D\xA2\x00\xF0\x9F\x8D\xA5\x00\xF0\x9F\x8D\xA1\x00\xF0\x9F\xA5\x9F\x00\xF0\x9F\xA5\xA1\x00\xF0\x9F\x8D\xA6\x00\xF0\x9F\x8D\xAA\x00\xF0\x9F\x8E\x82\x00\xF0\x9F\x8D\xB0\x00\xF0\x9F\xA5\xA7\x00\xF0\x9F\x8D\xAB\x00\xF0\x9F\x8D\xAF\x00\xF0\x9F\x8D\xBC\x00\xF0\x9F\xA5\x9B\x00\xF0\x9F\x8D\xB5\x00\xF0\x9F\x8D\xB6\x00\xF0\x9F\x8D\xBE\x00\xF0\x9F\x8D\xB7\x00\xF0\x9F\x8D\xBB\x00\xF0\x9F\xA5\x82\x00\xF0\x9F\xA5\x83\x00\xF0\x9F\xA5\xA4\x00\xF0\x9F\xA5\xA2\x00\xF0\x9F\x91\x81\x00\xF0\x9F\x91\x85\x00\xF0\x9F\x91\x84\x00\xF0\x9F\x92\x8B\x00\xF0\x9F\x92\x98\x00\xF0\x9F\x92\x93\x00\xF0\x9F\x92\x97\x00\xF0\x9F\x92\x99\x00\xF0\x9F\x92\x9B\x00\xF0\x9F\xA7\xA1\x00\xF0\x9F\x92\x9C\x00\xF0\x9F\x96\xA4\x00\xF0\x9F\x92\x9D\x00\xF0\x9F\x92\x9F\x00\xF0\x9F\x92\x8C\x00\xF0\x9F\x92\xA4\x00\xF0\x9F\x92\xA2\x00\xF0\x9F\x92\xA3\x00";

struct Message {
    text: &'static str,
    language: &'static str,
}

// Array containing all of the emojis messages
const MESSAGES: &[Message] = &[
    Message {
        text: "Falsches Üben von Xylophonmusik quält jeden größeren Zwerg",
        language: "German",
    },
    Message {
        text: "Beiß nicht in die Hand, die dich füttert.",
        language: "German",
    },
    Message {
        text: "Außerordentliche Übel erfordern außerordentliche Mittel.",
        language: "German",
    },
    Message {
        text: "Կրնամ ապակի ուտել և ինծի անհանգիստ չընել",
        language: "Armenian",
    },
    Message {
        text: "Երբ որ կացինը եկաւ անտառ, ծառերը ասացին... «Կոտը մերոնցից է:»",
        language: "Armenian",
    },
    Message {
        text: "Գառը՝ գարնան, ձիւնը՝ ձմռան",
        language: "Armenian",
    },
    Message {
        text: "Jeżu klątw, spłódź Finom część gry hańb!",
        language: "Polish",
    },
    Message {
        text: "Dobrymi chęciami jest piekło wybrukowane.",
        language: "Polish",
    },
    Message {
        text: "Îți mulțumesc că ai ales raylib.\nȘi sper să ai o zi bună!",
        language: "Romanian",
    },
    Message {
        text: "Эх, чужак, общий съём цен шляп (юфть) вдрызг!",
        language: "Russian",
    },
    Message {
        text: "Я люблю raylib!",
        language: "Russian",
    },
    Message {
        text: "Молчи, скрывайся и таи\nИ чувства и мечты свои –\nПускай в душевной глубине\nИ всходят и зайдут оне\nКак звезды ясные в ночи-\nЛюбуйся ими – и молчи.",
        language: "Russian",
    },
    Message {
        text: "Voix ambiguë d’un cœur qui au zéphyr préfère les jattes de kiwi",
        language: "French",
    },
    Message {
        text: "Benjamín pidió una bebida de kiwi y fresa; Noé, sin vergüenza, la más exquisita champaña del menú.",
        language: "Spanish",
    },
    Message {
        text: "Ταχίστη αλώπηξ βαφής ψημένη γη, δρασκελίζει υπέρ νωθρού κυνός",
        language: "Greek",
    },
    Message {
        text: "Η καλύτερη άμυνα είναι η επίθεση.",
        language: "Greek",
    },
    Message {
        text: "Χρόνια και ζαμάνια!",
        language: "Greek",
    },
    Message {
        text: "Πώς τα πας σήμερα;",
        language: "Greek",
    },
    Message {
        text: "我能吞下玻璃而不伤身体。",
        language: "Chinese",
    },
    Message {
        text: "你吃了吗?",
        language: "Chinese",
    },
    Message {
        text: "不作不死。",
        language: "Chinese",
    },
    Message {
        text: "最近好吗?",
        language: "Chinese",
    },
    Message {
        text: "塞翁失马,焉知非福。",
        language: "Chinese",
    },
    Message {
        text: "千军易得, 一将难求",
        language: "Chinese",
    },
    Message {
        text: "万事开头难。",
        language: "Chinese",
    },
    Message {
        text: "风无常顺,兵无常胜。",
        language: "Chinese",
    },
    Message {
        text: "活到老,学到老。",
        language: "Chinese",
    },
    Message {
        text: "一言既出,驷马难追。",
        language: "Chinese",
    },
    Message {
        text: "路遥知马力,日久见人心",
        language: "Chinese",
    },
    Message {
        text: "有理走遍天下,无理寸步难行。",
        language: "Chinese",
    },
    Message {
        text: "猿も木から落ちる",
        language: "Japanese",
    },
    Message {
        text: "亀の甲より年の功",
        language: "Japanese",
    },
    Message {
        text: "うらやまし  思ひ切る時  猫の恋",
        language: "Japanese",
    },
    Message {
        text: "虎穴に入らずんば虎子を得ず。",
        language: "Japanese",
    },
    Message {
        text: "二兎を追う者は一兎をも得ず。",
        language: "Japanese",
    },
    Message {
        text: "馬鹿は死ななきゃ治らない。",
        language: "Japanese",
    },
    Message {
        text: "枯野路に\u{3000}影かさなりて\u{3000}わかれけり",
        language: "Japanese",
    },
    Message {
        text: "繰り返し麦の畝縫ふ胡蝶哉",
        language: "Japanese",
    },
    Message {
        text: "아득한 바다 위에 갈매기 두엇 날아 돈다.\n너훌너훌 시를 쓴다. 모르는 나라 글자다.\n널따란 하늘 복판에 나도 같이 시를 쓴다.",
        language: "Korean",
    },
    Message {
        text: "제 눈에 안경이다",
        language: "Korean",
    },
    Message {
        text: "꿩 먹고 알 먹는다",
        language: "Korean",
    },
    Message {
        text: "로마는 하루아침에 이루어진 것이 아니다",
        language: "Korean",
    },
    Message {
        text: "고생 끝에 낙이 온다",
        language: "Korean",
    },
    Message {
        text: "개천에서 용 난다",
        language: "Korean",
    },
    Message {
        text: "안녕하세요?",
        language: "Korean",
    },
    Message {
        text: "만나서 반갑습니다",
        language: "Korean",
    },
    Message {
        text: "한국말 하실 줄 아세요?",
        language: "Korean",
    },
];

//--------------------------------------------------------------------------------------
// Module Functions Declaration
//--------------------------------------------------------------------------------------
// Fills the emoji array with random emoji (only those emojis present in fontEmoji)
fn randomize_emoji(
    emoji: &mut [Emoji; EMOJI_COUNT],
    hovered: &mut i32,
    selected: &mut i32,
    rl: &mut RaylibHandle,
) {
    *hovered = -1;
    *selected = -1;
    let start = rl.get_random_value::<i32>(45..=360);

    #[expect(
        clippy::needless_range_loop,
        reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
    )]
    for i in 0..EMOJI_COUNT {
        // 0-179 emoji codepoints (from emoji char array) each 4bytes + null char
        emoji[i].index = rl.get_random_value::<i32>(0..=179) * 5;

        // Generate a random color for this emoji
        emoji[i].color =
            Color::color_from_hsv(((start * (i as i32 + 1)) % 360) as f32, 0.6, 0.85).alpha(0.8);

        // Set a random message for this emoji
        emoji[i].message = rl.get_random_value::<i32>(0..=(MESSAGES.len() as i32 - 1));
    }
}

// Draw text using font inside rectangle limits
#[expect(
    clippy::too_many_arguments,
    reason = "C-parity: mirrors the C function signature"
)]
fn draw_text_boxed<D: RaylibDraw>(
    d: &mut D,
    font: &Font,
    text: &str,
    rec: Rectangle,
    font_size: f32,
    spacing: f32,
    word_wrap: bool,
    tint: Color,
) {
    draw_text_boxed_selectable(
        d,
        font,
        text,
        rec,
        font_size,
        spacing,
        word_wrap,
        tint,
        0,
        0,
        Color::WHITE,
        Color::WHITE,
    );
}

// Draw text using font inside rectangle limits with support for text selection
#[allow(clippy::too_many_arguments)]
fn draw_text_boxed_selectable<D: RaylibDraw>(
    d: &mut D,
    font: &Font,
    text: &str,
    rec: Rectangle,
    font_size: f32,
    spacing: f32,
    word_wrap: bool,
    tint: Color,
    mut select_start: i32,
    select_length: i32,
    select_tint: Color,
    select_back_tint: Color,
) {
    let bytes = text.as_bytes();
    let length = bytes.len() as i32;

    let mut text_offset_y: f32 = 0.0;
    let mut text_offset_x: f32 = 0.0;

    let scale_factor = font_size / font.base_size() as f32;

    const MEASURE_STATE: i32 = 0;
    const DRAW_STATE: i32 = 1;
    let mut state = if word_wrap { MEASURE_STATE } else { DRAW_STATE };

    let mut start_line: i32 = -1;
    let mut end_line: i32 = -1;
    let mut lastk: i32 = -1;

    let mut i: i32 = 0;
    let mut k: i32 = 0;
    while i < length {
        let (codepoint, codepoint_byte_count) = next_codepoint(bytes, i as usize);
        let mut codepoint_byte_count = codepoint_byte_count;
        let index = font.get_glyph_index(codepoint);

        if codepoint as i32 == 0x3f {
            codepoint_byte_count = 1;
        }
        i += codepoint_byte_count - 1;

        let mut glyph_width: f32 = 0.0;
        if codepoint != '\n' {
            let advance_x = font.chars()[index as usize].advanceX;
            glyph_width = if advance_x == 0 {
                font_recs(font)[index as usize].width * scale_factor
            } else {
                advance_x as f32 * scale_factor
            };

            if i + 1 < length {
                glyph_width += spacing;
            }
        }

        if state == MEASURE_STATE {
            if codepoint == ' ' || codepoint == '\t' || codepoint == '\n' {
                end_line = i;
            }

            if (text_offset_x + glyph_width) > rec.width {
                end_line = if end_line < 1 { i } else { end_line };
                if i == end_line {
                    end_line -= codepoint_byte_count;
                }
                if (start_line + codepoint_byte_count) == end_line {
                    end_line = i - codepoint_byte_count;
                }
                state = 1 - state;
            } else if (i + 1) == length {
                end_line = i;
                state = 1 - state;
            } else if codepoint == '\n' {
                state = 1 - state;
            }

            if state == DRAW_STATE {
                text_offset_x = 0.0;
                i = start_line;
                glyph_width = 0.0;

                let tmp = lastk;
                lastk = k - 1;
                k = tmp;
            }
        } else {
            if codepoint == '\n' {
                if !word_wrap {
                    text_offset_y +=
                        (font.base_size() as f32 + font.base_size() as f32 / 2.0) * scale_factor;
                    text_offset_x = 0.0;
                }
            } else {
                if !word_wrap && (text_offset_x + glyph_width) > rec.width {
                    text_offset_y +=
                        (font.base_size() as f32 + font.base_size() as f32 / 2.0) * scale_factor;
                    text_offset_x = 0.0;
                }

                if (text_offset_y + font.base_size() as f32 * scale_factor) > rec.height {
                    break;
                }

                let mut is_glyph_selected = false;
                if select_start >= 0 && k >= select_start && k < (select_start + select_length) {
                    d.draw_rectangle_rec(
                        Rectangle::new(
                            rec.x + text_offset_x - 1.0,
                            rec.y + text_offset_y,
                            glyph_width,
                            font.base_size() as f32 * scale_factor,
                        ),
                        select_back_tint,
                    );
                    is_glyph_selected = true;
                }

                if codepoint != ' ' && codepoint != '\t' {
                    d.draw_text_codepoint(
                        font,
                        codepoint as i32,
                        Vector2::new(rec.x + text_offset_x, rec.y + text_offset_y),
                        font_size,
                        if is_glyph_selected { select_tint } else { tint },
                    );
                }
            }

            if word_wrap && i == end_line {
                text_offset_y +=
                    (font.base_size() as f32 + font.base_size() as f32 / 2.0) * scale_factor;
                text_offset_x = 0.0;
                start_line = end_line;
                end_line = -1;
                glyph_width = 0.0;
                select_start += lastk - k;
                k = lastk;

                state = 1 - state;
            }
        }

        text_offset_x += glyph_width;

        i += 1;
        k += 1;
    }
}

fn font_recs(font: &Font) -> &[Rectangle] {
    let inner: &raylib::ffi::Font = std::convert::AsRef::as_ref(font);
    unsafe { std::slice::from_raw_parts(inner.recs, inner.glyphCount as usize) }
}

fn next_codepoint(bytes: &[u8], i: usize) -> (char, i32) {
    if i >= bytes.len() {
        return ('\u{3f}', 1);
    }
    let b0 = bytes[i];
    let (cp, n) = if b0 < 0x80 {
        (b0 as u32, 1)
    } else if (b0 & 0xE0) == 0xC0 && i + 1 < bytes.len() {
        let b1 = bytes[i + 1];
        (((b0 as u32 & 0x1F) << 6) | (b1 as u32 & 0x3F), 2)
    } else if (b0 & 0xF0) == 0xE0 && i + 2 < bytes.len() {
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];
        (
            ((b0 as u32 & 0x0F) << 12) | ((b1 as u32 & 0x3F) << 6) | (b2 as u32 & 0x3F),
            3,
        )
    } else if (b0 & 0xF8) == 0xF0 && i + 3 < bytes.len() {
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];
        let b3 = bytes[i + 3];
        (
            ((b0 as u32 & 0x07) << 18)
                | ((b1 as u32 & 0x3F) << 12)
                | ((b2 as u32 & 0x3F) << 6)
                | (b3 as u32 & 0x3F),
            4,
        )
    } else {
        (0x3f, 1)
    };
    (char::from_u32(cp).unwrap_or('\u{3f}'), n)
}

// Read a NUL-terminated emoji UTF-8 token from EMOJI_CODEPOINTS at the given byte index
fn emoji_token(start: usize) -> &'static str {
    let mut end = start;
    while end < EMOJI_CODEPOINTS.len() && EMOJI_CODEPOINTS[end] != 0 {
        end += 1;
    }
    std::str::from_utf8(&EMOJI_CODEPOINTS[start..end]).unwrap_or("?")
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [text] example - unicode emojis")
        .msaa_4x()
        .vsync()
        .build();

    // Load the font resources
    // NOTE: fontAsian is for asian languages,
    // fontEmoji is the emojis and fontDefault is used for everything else
    let font_default = rl
        .load_font(&thread, "resources/text/dejavu.fnt")
        .expect("dejavu.fnt load"); // Requires "resources/dejavu.png"
    let font_asian = rl
        .load_font(&thread, "resources/text/noto_cjk.fnt")
        .expect("noto_cjk.fnt load"); // Requires "resources/noto_cjk.png"
    let font_emoji = rl
        .load_font(&thread, "resources/text/symbola.fnt")
        .expect("symbola.fnt load"); // Requires "resources/symbola.png"

    let mut hovered_pos = Vector2::new(0.0, 0.0);
    let mut selected_pos = Vector2::new(0.0, 0.0);

    let mut emoji: [Emoji; EMOJI_COUNT] = [Emoji {
        index: 0,
        message: 0,
        color: Color::WHITE,
    }; EMOJI_COUNT];
    let mut hovered: i32 = -1;
    let mut selected: i32 = -1;

    // Set a random set of emojis when starting up
    randomize_emoji(&mut emoji, &mut hovered, &mut selected, &mut rl);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Add a new set of emojis when SPACE is pressed
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            randomize_emoji(&mut emoji, &mut hovered, &mut selected, &mut rl);
        }

        // Set the selected emoji
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            && hovered != -1
            && hovered != selected
        {
            selected = hovered;
            selected_pos = hovered_pos;
        }

        let mouse = rl.get_mouse_position();
        let mut position = Vector2::new(28.8, 10.0);
        hovered = -1;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw random emojis in the background
        //------------------------------------------------------------------------------
        #[expect(
            clippy::needless_range_loop,
            reason = "C-parity: mirrors the C for (i = 0; i < n; i++) indexed loop"
        )]
        for i in 0..EMOJI_COUNT {
            let txt = emoji_token(emoji[i].index as usize);
            let emoji_rect = Rectangle::new(
                position.x,
                position.y,
                font_emoji.base_size() as f32,
                font_emoji.base_size() as f32,
            );

            if !emoji_rect.check_collision_point_rec(mouse) {
                d.draw_text_ex(
                    &font_emoji,
                    txt,
                    position,
                    font_emoji.base_size() as f32,
                    1.0,
                    if selected == i as i32 {
                        emoji[i].color
                    } else {
                        Color::LIGHTGRAY.alpha(0.4)
                    },
                );
            } else {
                d.draw_text_ex(
                    &font_emoji,
                    txt,
                    position,
                    font_emoji.base_size() as f32,
                    1.0,
                    emoji[i].color,
                );
                hovered = i as i32;
                hovered_pos = position;
            }

            if i != 0 && i % EMOJI_PER_WIDTH == 0 {
                position.y += font_emoji.base_size() as f32 + 24.25;
                position.x = 28.8;
            } else {
                position.x += font_emoji.base_size() as f32 + 28.8;
            }
        }
        //------------------------------------------------------------------------------

        // Draw the message when a emoji is selected
        //------------------------------------------------------------------------------
        if selected != -1 {
            let message = emoji[selected as usize].message as usize;
            let horizontal_padding: i32 = 20;
            let vertical_padding: i32 = 30;
            let font: &Font = if MESSAGES[message].language == "Chinese"
                || MESSAGES[message].language == "Korean"
                || MESSAGES[message].language == "Japanese"
            {
                &font_asian
            } else {
                &font_default
            };

            // Calculate size for the message box (approximate the height and width)
            let mut sz = font.measure_text(MESSAGES[message].text, font.base_size() as f32, 1.0);
            if sz.x > 300.0 {
                sz.y *= sz.x / 300.0;
                sz.x = 300.0;
            } else if sz.x < 160.0 {
                sz.x = 160.0;
            }

            let mut msg_rect = Rectangle::new(
                selected_pos.x - 38.8,
                selected_pos.y,
                2.0 * horizontal_padding as f32 + sz.x,
                2.0 * vertical_padding as f32 + sz.y,
            );
            msg_rect.y -= msg_rect.height;

            // Coordinates for the chat bubble triangle
            let mut a = Vector2::new(selected_pos.x, msg_rect.y + msg_rect.height);
            let mut b = Vector2::new(a.x + 8.0, a.y + 10.0);
            let mut c = Vector2::new(a.x + 10.0, a.y);

            // Don't go outside the screen
            if msg_rect.x < 10.0 {
                msg_rect.x += 28.0;
            }
            if msg_rect.y < 10.0 {
                msg_rect.y = selected_pos.y + 84.0;
                a.y = msg_rect.y;
                c.y = a.y;
                b.y = a.y - 10.0;

                // Swap values so we can actually render the triangle :(
                std::mem::swap(&mut a, &mut b);
            }

            if msg_rect.x + msg_rect.width > screen_width as f32 {
                msg_rect.x -= (msg_rect.x + msg_rect.width) - screen_width as f32 + 10.0;
            }

            // Draw chat bubble
            d.draw_rectangle_rec(msg_rect, emoji[selected as usize].color);
            d.draw_triangle(a, b, c, emoji[selected as usize].color);

            // Draw the main text message
            let text_rect = Rectangle::new(
                msg_rect.x + horizontal_padding as f32 / 2.0,
                msg_rect.y + vertical_padding as f32 / 2.0,
                msg_rect.width - horizontal_padding as f32,
                msg_rect.height,
            );
            draw_text_boxed(
                &mut d,
                font,
                MESSAGES[message].text,
                text_rect,
                font.base_size() as f32,
                1.0,
                true,
                Color::WHITE,
            );

            // Draw the info text below the main message
            let size = MESSAGES[message].text.len();
            let length = MESSAGES[message].text.chars().count();
            let info = format!(
                "{} {} characters {} bytes",
                MESSAGES[message].language, length, size
            );
            // SAFETY: GetFontDefault returns a static raylib font; MeasureTextEx is pure.
            let sz2: Vector2 = unsafe {
                let c = std::ffi::CString::new(info.as_str()).unwrap();
                raylib::ffi::MeasureTextEx(raylib::ffi::GetFontDefault(), c.as_ptr(), 10.0, 1.0)
            };

            d.draw_text(
                &info,
                (text_rect.x + text_rect.width - sz2.x) as i32,
                (msg_rect.y + msg_rect.height - sz2.y - 2.0) as i32,
                10,
                Color::RAYWHITE,
            );
        }
        //------------------------------------------------------------------------------

        // Draw the info text
        d.draw_text(
            "These emojis have something to tell you, click each to find out!",
            (screen_width - 650) / 2,
            screen_height - 40,
            20,
            Color::GRAY,
        );
        d.draw_text(
            "Each emoji is a unicode character from a font, not a texture... Press [SPACEBAR] to refresh",
            (screen_width - 484) / 2,
            screen_height - 16,
            10,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadFont(fontDefault); UnloadFont(fontAsian); UnloadFont(fontEmoji) — RAII drop.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
