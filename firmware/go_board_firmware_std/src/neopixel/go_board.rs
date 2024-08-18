use anyhow::{anyhow, Result};
use tokio::sync::mpsc::Sender;
use super::led_ctrl::LedChange;
use super::rgb::Rgb;

pub async fn show_board(tx: &Sender<LedChange>, board: &Vec<Vec<i32>>, height: usize, width: usize) -> Result<()> {
    if height > 16 || width > 16 {
        return Err(anyhow!(
            "Board is too long or wide W:{width}>16 or H:{height}>16 "
        ));
    }
    for x in 0..height {
        for y in 0..width {
            println!("SHOWING {}",board[x][y]);
            tx.send(LedChange::new(x as u8, y as u8,
                                   match board[x][y] {
                                       1 => Rgb::new(0, 0, 50),
                                       2 => Rgb::new(0, 50, 0),
                                       _ => Rgb::new(0, 0, 0),
                                   } )).await?;
        }
    }
   
   
    
    Ok(())
}
