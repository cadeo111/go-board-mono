use crate::neopixel::led_ctrl::LedChange;
use crate::neopixel::rgb::{Rgb, BLUE, GREEN, RED, WHITE};

pub fn score_board(
    black_score: u16,
    white_score: u16,
    start_x: u8,
    start_y: u8,
) -> heapless::Vec<LedChange, { 16 + 18 + 16 + 18 }> {
    // start at (start_x, start_y) + (0,0) for b
    let letter_b = write_b(start_x, start_y, WHITE);
    // start at (start_x, start_y) + (6,0) for score blocks
    let black_score_indicator =
        write_number_in_colors(start_x + 6, start_y, black_score, 3, RED, BLUE, GREEN);

    // start at (start_x, start_y) + (0,5) for w
    let letter_w = write_b(start_x, start_y + 5, WHITE);
    // start at (start_x, start_y) + (6,5) for score blocks
    let white_score_indicator =
        write_number_in_colors(start_x + 6, start_y + 5, black_score, 3, RED, BLUE, GREEN);
    let mut vec = heapless::Vec::new();
    vec.extend(letter_b);
    vec.extend(black_score_indicator);
    vec.extend(letter_w);
    vec.extend(white_score_indicator);
    vec
}

///
///  start x/y is the top left corner
///  creates 5x5 pattern
///
///  X X X X
///  X       X
///  X X X X
///  X       X
///  X X X X
///
const fn write_b(start_x: u8, start_y: u8, color: Rgb) -> [LedChange; 16] {
    [
        //  X X X X
        LedChange {
            x: start_x,
            y: start_y,
            color,
        },
        LedChange {
            x: start_x + 1,
            y: start_y,
            color,
        },
        LedChange {
            x: start_x + 2,
            y: start_y,
            color,
        },
        LedChange {
            x: start_x + 3,
            y: start_y,
            color,
        },
        //  X    X
        LedChange {
            x: start_x,
            y: start_y + 1,
            color,
        },
        LedChange {
            x: start_x + 4,
            y: start_y + 1,
            color,
        },
        //  X X X
        LedChange {
            x: start_x,
            y: start_y + 2,
            color,
        },
        LedChange {
            x: start_x + 1,
            y: start_y + 2,
            color,
        },
        LedChange {
            x: start_x + 2,
            y: start_y + 2,
            color,
        },
        LedChange {
            x: start_x + 3,
            y: start_y + 2,
            color,
        },
        //  X    X
        LedChange {
            x: start_x,
            y: start_y + 3,
            color,
        },
        LedChange {
            x: start_x + 4,
            y: start_y + 3,
            color,
        },
        LedChange {
            x: start_x,
            y: start_y + 4,
            color,
        },
        LedChange {
            x: start_x + 1,
            y: start_y + 4,
            color,
        },
        LedChange {
            x: start_x + 2,
            y: start_y + 4,
            color,
        },
        LedChange {
            x: start_x + 3,
            y: start_y + 4,
            color,
        },
    ]
}

///
/// start x/y is the top left corner
///  creates 5x5 pattern
///  X       X
///  X       X
///  X   X   X
///  X X   X X
///  X       X
///
const fn write_w(start_x: u8, start_y: u8, color: Rgb) -> [LedChange; 13] {
    [
        //  X       X
        LedChange {
            x: start_x,
            y: start_y,
            color,
        },
        LedChange {
            x: start_x + 4,
            y: start_y,
            color,
        },
        //  X       X
        LedChange {
            x: start_x,
            y: start_y + 1,
            color,
        },
        LedChange {
            x: start_x + 4,
            y: start_y + 1,
            color,
        },
        //  X   X   X
        LedChange {
            x: start_x,
            y: start_y + 2,
            color,
        },
        LedChange {
            x: start_x + 2,
            y: start_y + 2,
            color,
        },
        LedChange {
            x: start_x + 4,
            y: start_y + 2,
            color,
        },
        //  X X   X X
        LedChange {
            x: start_x,
            y: start_y + 3,
            color,
        },
        LedChange {
            x: start_x + 1,
            y: start_y + 3,
            color,
        },
        LedChange {
            x: start_x + 3,
            y: start_y + 3,
            color,
        },
        LedChange {
            x: start_x + 4,
            y: start_y + 3,
            color,
        },
        //  X       X
        LedChange {
            x: start_x,
            y: start_y + 4,
            color,
        },
        LedChange {
            x: start_x + 4,
            y: start_y + 4,
            color,
        },
    ]
}

///
/// start x/y is the top left corner
/// will print blocks of colors to indicate score
fn write_number_in_colors(
    start_x: u8,
    start_y: u8,
    number: u16,
    block_height: u8,
    hundred_color: Rgb,
    ten_color: Rgb,
    one_color: Rgb,
) -> heapless::Vec<LedChange, 18> {
    let mut arr = [None; 18]; // 99 = 9 + 9 = 18, 256 = 2+5+6 = 13
    let mut current_index = 0;
    let mut current_x = start_x;
    let mut current_y = start_y;

    for (base, color) in [(100, hundred_color), (10, ten_color), (1, one_color)] {
        if number > base {
            let base_amount = number / base;
            for _ in 0..base_amount {
                arr[current_index] = Some(LedChange {
                    x: current_x,
                    y: current_y,
                    color,
                });

                current_index += 1;

                // roll over to create block that is `height` high
                if current_y - start_y >= block_height {
                    current_x += 1;
                    current_y = start_y;
                } else {
                    current_y += 1;
                }
            }
        }
    }
    heapless::Vec::from_iter(
        arr.iter()
            .filter(|v| match v {
                None => false,
                Some(_) => true,
            })
            .map(|v| v.unwrap()),
    )
}
