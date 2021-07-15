//use rand::{Rng, thread_rng};
use rand::{seq::SliceRandom, thread_rng};
use std::time::Instant;
use std::{env, fs};
use ncurses::*;


#[derive(Copy, Clone, PartialEq)]
enum Dot {
  Mine,
  Safe(u8),
  Flag(bool),
  Null,
}

#[derive(Copy, Clone, Debug)]
struct Point {
  x: usize,
  y: usize,
}

impl Point {
  fn new(x: usize, y: usize) -> Point {
    Point { x: x, y: y }
  }
}


struct Field {
  x_size: usize,
  y_size: usize,
  mines: u8,
  field: Vec<Vec<Dot>>,
  opened: Vec<Vec<bool>>,
  opennum: i32,
  position: Point,
}

impl Field {
  fn new(x_size: usize, y_size: usize, mines: u8, pos: Point) -> Field {
    let field = vec![vec![Dot::Null; x_size]; y_size];
    let opened = vec![vec![false; x_size]; y_size];
    let mut obj = Field { x_size: x_size, y_size: y_size, mines: mines, field: field, opened: opened, opennum: 0, position: pos};
    obj.init();
    obj
  }

  fn init(&mut self) {
    let mut rng = thread_rng(); // 乱数発生源
    let mut cordinate = vec![Point{x:0,y:0}; self.x_size * self.y_size];
    let mut y_idx: usize = 0;
    let mut cord_idx = 0;

    // create field
    while y_idx < self.y_size {
      let mut x_idx: usize = 0;
      while x_idx < self.x_size {
        cordinate[cord_idx] = Point::new(x_idx,y_idx);
        x_idx += 1;
        cord_idx += 1;
      }
      y_idx += 1;
    }

    // shuffle
    cordinate.shuffle(&mut rng);

    // set mine
    cord_idx = 0;
    while cord_idx < self.mines as usize {
      let pos = cordinate[cord_idx];
      self.field[pos.y][pos.x] = Dot::Mine;
      cord_idx += 1;
    }

    // calc safe number
    for y_idx in 0..self.field.len() {
      for x_idx in 0..self.field[0].len() {
        match self.field[y_idx][x_idx] {
          Dot::Null => {
            let pos = Point::new(x_idx, y_idx);
            self.field[pos.y][pos.x] = Dot::Safe(self.count_mine(pos))
          },
          _ => (),
        }
      }
    }
  }

  // 指定した座標の周囲有効なマス(up, right, down, left)
  fn get_around(&self, pos: Point) -> (usize, usize, usize, usize) {
    let up = match pos.y > 0 {
      true => pos.y-1,
      _ => pos.y,
    };
    let down = match pos.y < self.y_size-1 {
      true => pos.y+1,
      _ => pos.y,
    };
    let right = match pos.x < self.x_size-1 {
      true => pos.x+1,
      _ => pos.x,
    };
    let left = match pos.x > 0 {
      true => pos.x-1,
      _ => pos.x,
    };
    (up, right, down, left)
  }
  
  fn count_mine(&self, pos: Point) -> u8 {
    let mut count = 0;
    let (up, right, down, left) = self.get_around(pos);

    // 上と下の6つのマスを検査
    for x in left..=right {
      if up != pos.y && self.field[up][x] == Dot::Mine {
        count += 1;
      }
      if down != pos.y && self.field[down][x] == Dot::Mine {
        count += 1;
      }
    }
    // 中段の左右のマスを検査
    if left != pos.x && self.field[pos.y][left] == Dot::Mine {
      count += 1;
    }
    if right != pos.x && self.field[pos.y][right] == Dot::Mine {
      count += 1;
    }
    
    count as u8
  }

  fn print(&self) {
    mv(self.position.y as i32 -1, self.position.x as i32);
    for _ in 0..self.x_size {
      waddstr(stdscr(), "-");
    }
    for y in 0..self.y_size {
      mv((self.position.y + y) as i32, self.position.x as i32 -1);
      waddstr(stdscr(), "|");
      mv((self.position.y + y) as i32, self.position.x as i32);
      for x in 0..self.x_size {
        waddstr(stdscr(), &match self.opened[y][x] {
          true => { 
            match &self.field[y][x] {
              Dot::Mine => "x".to_string(),
              Dot::Safe(i) => {
                match i {
                  0 => " ".to_string(),
                  _ => i.to_string(),
                }
              },
              _ => "_".to_string(),
            }
          },
          false => {
            match &self.field[y][x] {
              Dot::Flag(_) => "f".to_string(),
              _ => ".".to_string(),
            }
          },
        });
      }
      waddstr(stdscr(), "|");
    }
    mv((self.position.y + self.y_size) as i32, self.position.x as i32);
    for _ in 0..self.x_size {
      waddstr(stdscr(), "-");
    }
    mv(self.position.y as i32, self.position.x as i32);
    refresh();
  }
  
  fn is_opened(&self, pos: Point) -> bool {
    self.opened[pos.y][pos.x]
  }

  // 1点あける
  fn open(&mut self, pos: Point) -> bool {
    if pos.x >= self.x_size || pos.y >= self.y_size {
      return true;
    }
    let res = match &self.field[pos.y][pos.x] {
      Dot::Mine => false,
      _ => true,
    };
    if self.field[pos.y][pos.x] == Dot::Safe(0) {
      self.open_if_zero(pos);
    }
    if res && !self.opened[pos.y][pos.x] {
      self.opened[pos.y][pos.x] = true;
      self.opennum += 1;
    }
    res
  }

  fn open_around(&mut self, pos:Point) {
    let (up, right, down, left) = self.get_around(pos);
    self.opened[up][left] = true;
    self.opened[up][pos.x] = true;
    self.opened[up][right] = true;
    self.opened[pos.y][left] = true;
    self.opened[pos.y][right] = true;
    self.opened[down][left] = true;
    self.opened[down][pos.x] = true;
    self.opened[down][right] = true;
  }

  // あけて、0だったときに周りの0もあける
  fn open_if_zero(&mut self, pos: Point) -> bool {
    let mut res = false;
    if self.field[pos.y][pos.x] == Dot::Safe(0) && !self.is_opened(pos) {
      self.opened[pos.y][pos.x] = true;
      res = true;
      self.opennum += 1;

      let (up, right, down, left) = self.get_around(pos);
      let directions: Vec<Point> = vec![
        Point::new(right, up),
        Point::new(right, down),
        Point::new(left, down),
        Point::new(left, up),
        Point::new(pos.x, up),
        Point::new(right, pos.y),
        Point::new(pos.x, down),
        Point::new(left, pos.y)
      ];
      for dir in directions {
        if !self.is_opened(dir) {
          self.open_if_zero(dir);
        }
      }
      self.open_around(pos);
    }
    res
  }

  fn set_flag(&mut self, pos: Point) {
    let exist_mine = match self.field[pos.y][pos.x] {
      Dot::Mine => true,
      _ => false,
    };
    if !self.opened[pos.y][pos.x] {
      self.field[pos.y][pos.x] = Dot::Flag(exist_mine);
    }
  }

  fn open_count(&self) -> u32 {
    let mut cnt = 0;
    for y in 0..self.y_size {
      for x in 0..self.x_size {
        if self.opened[y][x] {
          cnt += 1;
        }
      }
    }
    cnt
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let fieldpos = Point::new(5,5);
  let x_size;
  let y_size;
  let mines;
  let scoreable;
  if args.len() == 4 {
    x_size = args[1].parse().expect("不正な引数です。x_size y_size minesの順に指定してください。");
    y_size = args[2].parse().expect("不正な引数です。x_size y_size minesの順に指定してください。");
    mines = args[3].parse().expect("不正な引数です。x_size y_size minesの順に指定してください。");
    scoreable = false;
  } else {
    x_size = 10;
    y_size = 8;
    mines = 10;
    scoreable = true;
  }
  let _window = initscr();
  noecho();
  nonl();
  intrflush(stdscr(), true);
  keypad(stdscr(), true);
  addstr("***MSweeper***");
  refresh();

  let mut field = Field::new(x_size,y_size,mines,fieldpos);

  const KEY_QUIT: i32 = b'q' as i32;
  const KEY_OPEN: i32 = b'o' as i32;
  const KEY_FLAG: i32 = b'f' as i32;

  let mut x = 0;
  let mut y = 0;
  let mut res = true;
  let start = Instant::now();
  loop {
    field.print();
    mv((field.position.y + y) as i32, (field.position.x + x) as i32);

    // キー入力
    match getch() {
      KEY_RIGHT => if x < field.x_size-1 {x += 1},
      KEY_LEFT => if x > 0 {x -= 1},
      KEY_DOWN => if y < field.y_size-1 {y += 1},
      KEY_UP => if y > 0 {y -= 1},
      KEY_OPEN => {res = field.open(Point::new(x, y));},
      KEY_QUIT => {endwin(); return;},
      KEY_FLAG => {field.set_flag(Point::new(x, y));},
      _ => continue,
    };

    // 負け
    if !res {
      mv((y_size+fieldpos.y+5) as i32, 0);
      waddstr(stdscr(), "You Lose...");
      break;
    }
    // クリア
    if ((field.x_size * field.y_size) as u32 - field.open_count()) == field.mines as u32 {
      let end = start.elapsed();
      mv((y_size+fieldpos.y+5) as i32, 0);
      waddstr(stdscr(), "You Win!!!");
      mv((y_size+fieldpos.y+6) as i32, 0);
      waddstr(stdscr(), &format!("Time:{}s", end.as_secs()));
      if !scoreable {
        break;
      }
      let score_file = match fs::read_to_string("./highscore.dat") {
        Ok(val) => val.trim().to_string(),
        _ => "".to_string(),
      };
      let score = end.as_secs();
      let high;
      if score_file != "" {
        let score_str: Vec<&str> = score_file.split('\n').collect::<Vec<&str>>();
        let mut scores: Vec<u64> = score_str.iter().map(|x| x.parse()
          .expect("highscore.datの形式が違います。")).collect();
        scores.sort();
        high = scores[scores.len()-1];
      } else {
        high = u64::MAX;
      }
      if high > score {
        mv((y_size+fieldpos.y+7) as i32, 0);
        waddstr(stdscr(), "highscore!!!");
        let new_score_file = format!("{}\n{}", score_file, end.as_secs());
        fs::write("./highscore.dat", &new_score_file)
          .expect("highscore.datの書き込みに失敗しました");
      }
      break;
    }
  }
  while getch() != KEY_QUIT { }
  endwin();
}
