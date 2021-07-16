use std::time::Instant;
use std::{env, fs};
use ncurses::*;

mod internal;
use internal::*;

/// 記録をスコアファイルに追記します。
/// * `filename` - 保存するファイル名
/// * `score` - 保存するスコア
fn save_record(filename: &str, score: u64) {
  let score_file = get_score_file(filename);
  let new_score_file = format!("{}\n{}", score_file, score);
  fs::write(&filename, new_score_file.trim())
    .expect("highscore.datの書き込みに失敗しました");
}

/// 記録をファイルから読み込み、文字列として返します。
/// * `filename` - 記録のファイル名
fn get_score_file(filename: &str) -> String {
  match fs::read_to_string(&filename) {
    Ok(val) => val.trim().to_string(),
    _ => "".to_string(),
  }
}

/// 記録のうち最もはやいものを返します。
/// * `score_file` - 記録ファイルの文字列データ
fn get_highscore(score_file: &str) -> u64 {
  if score_file != "" {
    let score_str: Vec<&str> = score_file.split('\n').collect::<Vec<&str>>();
    let mut scores: Vec<u64> = score_str.iter().map(|x| x.parse()
        .expect("highscore.datの形式が違います。")).collect();
    scores.sort();
    scores[scores.len()-1]
  } else {
    u64::MAX
  }
}

/// ncursesの初期化を行います。
fn ncurses_setup() {
  initscr();
  noecho();
  nonl();
  intrflush(stdscr(), true);
  keypad(stdscr(), true);
  waddstr(stdscr(), "***MSweeper***");
  refresh();
}

/// コマンドライン引数からフィールドのサイズ、地雷の数を受け取り、タプルとして返します。
/// 戻り値の形式：(x_size: usize, y_size: usize, mines: u32, scoreable: bool);
fn get_args() -> (usize, usize, u32, bool) {
  let args: Vec<String> = env::args().collect();
  let (x_size, y_size, mines, scoreable);
  if args.len() == 4 {
    x_size = args[1].parse()
      .expect("不正な引数です。x_size y_size minesの順に指定してください。");
    y_size = args[2].parse()
      .expect("不正な引数です。x_size y_size minesの順に指定してください。");
    mines = args[3].parse()
      .expect("不正な引数です。x_size y_size minesの順に指定してください。");
    scoreable = false;
  } else {
    x_size = 10;
    y_size = 8;
    mines = 10;
    scoreable = true;
  }
  (x_size, y_size, mines, scoreable)
}

fn main() {
  ncurses_setup();

  let fieldpos = Point::new(5,5);
  let (x_size, y_size, mines, scoreable) = get_args();
  let mut field = Field::new(x_size,y_size,mines,fieldpos);
  let field_size = (x_size * y_size) as u32;

  const KEY_QUIT:    i32 = b'q' as i32;
  const KEY_OPEN:    i32 = b'o' as i32;
  const KEY_FLAG:    i32 = b'f' as i32;
  const KEY_UNKNOWN: i32 = b'?' as i32;
  const KEY_LEFT:    i32 = b'h' as i32;
  const KEY_DOWN:    i32 = b'j' as i32;
  const KEY_UP:      i32 = b'k' as i32;
  const KEY_RIGHT:   i32 = b'l' as i32;

  mv((y_size+fieldpos.y+2) as i32,0);
  waddstr(stdscr(),
    &format!("open:{}, flag:{}, unknown:{}, move:hjkl, quit:{}",
      std::char::from_u32(KEY_OPEN as u32).unwrap(),
      std::char::from_u32(KEY_FLAG as u32).unwrap(),
      std::char::from_u32(KEY_UNKNOWN as u32).unwrap(),
      std::char::from_u32(KEY_QUIT as u32).unwrap()
    )
  );

  let mut x = 0;
  let mut y = 0;
  let mut is_mine = true;
  let start = Instant::now();
  loop {
    field.print();
    mv((field.position.y + y) as i32, (field.position.x + x) as i32);

    // キー入力
    match getch() {
      KEY_RIGHT    => if x < field.x_size-1 {x += 1},
      KEY_LEFT     => if x > 0 {x -= 1},
      KEY_DOWN     => if y < field.y_size-1 {y += 1},
      KEY_UP       => if y > 0 {y -= 1},
      KEY_OPEN     => {is_mine = field.open(Point::new(x, y));},
      KEY_QUIT     => {endwin(); return;},
      KEY_FLAG     => {field.set_flag(Point::new(x, y));},
      KEY_UNKNOWN  => {field.set_unknown(Point::new(x, y));},
      _ => continue,
    };

    // 負け
    if !is_mine {
      mv(1, 0);
      waddstr(stdscr(), "You Lose...");
      break;
    }
    // 勝ち
    if (field_size - field.open_count()) == field.mines as u32 {
      let end = start.elapsed();

      mv(1, 0);
      waddstr(stdscr(), "You Win!!!");
      mv(2, 0);
      waddstr(stdscr(), &format!("Time:{}s", end.as_secs()));

      if scoreable {
        let filename = "./highscore.dat";
        let score_file = get_score_file(&filename);
        let score = end.as_secs();
        if get_highscore(&score_file) > score {
          mv(3, 0);
          waddstr(stdscr(), "highscore!!!");
        }
        save_record(&filename, score);
      }
      break;
    }
  }
  while getch() != KEY_QUIT { }
  endwin();
}
