use ncurses::*;
use rand::{seq::SliceRandom, thread_rng};

#[derive(Clone, PartialEq)]
/// 1マスを表す列挙体
pub enum Dot {
  /// 地雷
  Mine,
  /// 安全地帯(周囲８マスの地雷数)
  Safe(u8),
  /// 地雷が存在することを示すフラグ(フラグを立てる前の値)
  Flag(Box<Dot>),
  /// 初期化時用のNull
  Null,
}

#[derive(Copy, Clone, Debug)]
/// フィールド上の位置を示します
pub struct Point {
  /// x座標
  pub x: usize,
  /// y座標
  pub y: usize,
}

impl Point {
  pub fn new(x: usize, y: usize) -> Point {
    Point { x: x, y: y }
  }
}


/// ゲームフィールド
pub struct Field {
  /// フィールドの横幅
  pub x_size: usize,
  /// フィールドの縦幅
  pub y_size: usize,
  /// フィールド内に生成される地雷の数
  pub mines: u32,
  /// フィールドの状態
  pub field: Vec<Vec<Dot>>,
  /// フィールドの開放状況
  pub opened: Vec<Vec<bool>>,
  /// フィールドが設置される画面上の位置
  pub position: Point,
}

impl Field {
  /// 新しいフィールドを生成します。
  /// * `x_size` - フィールドの横幅
  /// * `y_size` - フィールドの縦幅
  /// * `mines` - フィールドに生成される地雷の数
  /// * `pos` - フィールドが設置される画面上の位置
  pub fn new(x_size: usize, y_size: usize, mines: u32, pos: Point) -> Field {
    let field = vec![vec![Dot::Null; x_size]; y_size];
    let opened = vec![vec![false; x_size]; y_size];
    let mut obj = Field { 
      x_size: x_size,
      y_size: y_size,
      mines: mines,
      field: field,
      opened: opened,
      position: pos
    };
    obj.init();
    obj
  }

  /// `new()`によって渡されたパラメータをもとにフィールドを初期化します。
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

  /// `pos`で指定された座標の周囲８マスを調べ、(up, right, down, left)の順番で有効な座標を返します。
  /// * `pos` - フィールド上の位置
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
  
  /// `pos`で指定された座標の周囲８マスに含まれる地雷の数を返します。
  /// * `pos` - フィールド上の位置
  pub fn count_mine(&self, pos: Point) -> u8 {
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

  /// フィールドを枠とともに描画します。
  pub fn print(&self) {
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
  
  /// `pos`で指定された座標がすでに開放されているかを返します。
  /// * `pos` - フィールド上の位置
  fn is_opened(&self, pos: Point) -> bool {
    self.opened[pos.y][pos.x]
  }

  /// `pos`で指定された座標を開放し、その周囲も開放できることが自明である場合は開放します。
  /// * `pos` - フィールド上の位置
  pub fn open(&mut self, pos: Point) -> bool {
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
    }
    res
  }

  /// `pos`で指定された座標の周囲８マスを開放します。
  /// * `pos` - フィールド上の位置
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

  /// `pos`で指定された座標に地雷がなく、周囲８マスに地雷が存在しない場合は周囲も再帰的に開放します。
  /// * `pos` - フィールド上の位置
  fn open_if_zero(&mut self, pos: Point) -> bool {
    let mut res = false;
    if self.field[pos.y][pos.x] == Dot::Safe(0) && !self.is_opened(pos) {
      self.opened[pos.y][pos.x] = true;
      res = true;

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

  /// `pos`で指定された座標に地雷が存在することを示すフラグを立てます。
  /// * `pos` - フィールド上の位置
  pub fn set_flag(&mut self, pos: Point) {
    if !self.opened[pos.y][pos.x] {
      self.field[pos.y][pos.x] = match &self.field[pos.y][pos.x] {
        Dot::Flag(f) => (**f).clone(),
        _ => Dot::Flag(Box::new(self.field[pos.y][pos.x].clone())),
      }
    }
  }

  /// すでに開放されているマスの数を返します。
  pub fn open_count(&self) -> u32 {
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
