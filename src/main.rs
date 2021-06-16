use std::io::Write;
use rand::{Rng, thread_rng};


#[derive(Copy, Clone, PartialEq)]
enum Dot {
  Mine,
  Safe(u8),
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

  /// 標準入力からpointを生成します
  fn input() -> Point {
    let mut x;
    let mut y;
    let xx;
    loop {
      print!("x >");
      std::io::stdout().flush().unwrap();
      x = String::new();
      std::io::stdin().read_line(&mut x).expect("標準入力に失敗しました。");
      xx = match x.trim().parse::<usize>() {
        Ok(val) => val,
        _ => continue,
      };
      break;
    }
    let yy;
    loop {
      print!("y >");
      std::io::stdout().flush().unwrap();
      y = String::new();
      std::io::stdin().read_line(&mut y).expect("標準入力に失敗しました。");
      yy = match y.trim().parse::<usize>() {
        Ok(val) => val,
        _ => continue,
      };
      break;
    }
    Point::new(xx, yy)
  }
}


struct Field {
  x_size: usize,
  y_size: usize,
  mines: u8,
  field: Vec<Vec<Dot>>,
  opened: Vec<Vec<bool>>,
  opennum: i32,
}

impl Field {
  fn new(x_size: usize, y_size: usize, mines: u8) -> Field {
    let field = vec![vec![Dot::Null; x_size]; y_size];
    let opened = vec![vec![false; x_size]; y_size];
    let mut obj = Field { x_size: x_size, y_size: y_size, mines: mines, field: field, opened: opened, opennum: 0};
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
    cord_idx = 0;
    while cord_idx < cordinate.len() {
      let rand_idx = rng.gen_range(0..cordinate.len()) as usize;
      let tmp = cordinate[rand_idx];
      cordinate[rand_idx] = cordinate[cord_idx];
      cordinate[cord_idx] = tmp;
      cord_idx += 1;
    }

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

  fn print_debug(&self) {
    for y in 0..self.y_size {
      for x in 0..self.x_size {
        print!("{} ", match &self.field[y][x] {
          Dot::Mine => "x".to_string(),
          Dot::Safe(i) => i.to_string(),
          _ => "_".to_string(),
        });
      }
      println!("");
    }
  }

  fn print(&self) {
    print!("   ");
    for x in 0..self.x_size {
      print!("{:>3}", x);
    }
    println!("");
    for y in 0..self.y_size {
      print!("{:>3}", y);
      for x in 0..self.x_size {
        print!("{:>3}", match self.opened[y][x] {
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
          false => ".".to_string(),
        });
      }
      println!("");
    }
  }
  
  fn is_opened(&self, pos: Point) -> bool {
    self.opened[pos.y][pos.x]
  }

  // 1点あける
  fn open(&mut self, pos: Point) -> bool {
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

  // あけて、0だったときに周りの0もあける
  fn open_if_zero(&mut self, pos: Point) -> bool {
    let mut res = false;
    if self.field[pos.y][pos.x] == Dot::Safe(0) && !self.is_opened(pos) {
      self.opened[pos.y][pos.x] = true;
      res = true;
      self.opennum += 1;

      let (up, right, down, left) = self.get_around(pos);
      let up = Point::new(pos.x, up);
      let right = Point::new(right, pos.y);
      let down = Point::new(pos.x, down);
      let left = Point::new(left, pos.y);
      if !self.is_opened(up) {
        self.open_if_zero(up);
      }
      if !self.is_opened(right) {
        self.open_if_zero(right);
      }
      if !self.is_opened(down) {
        self.open_if_zero(down);
      }
      if !self.is_opened(left) {
        self.open_if_zero(left);
      }
    }
    res
  }
}
fn main() {
  let mut field = Field::new(10,10,5);
  field.print();

  loop {
    let pos = Point::input();
    let res = field.open(pos);
    field.print();
    if !res {
      println!("you lose!");
      break;
    }
    if field.opennum >= (field.x_size * field.y_size) as i32 - (field.mines as i32) {
      println!("you win!");
      break;
    }
  }
}
