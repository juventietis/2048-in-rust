#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate cursive;
extern crate rand;
extern crate fern;
extern crate chrono;
#[macro_use]
extern crate log;

use std::fmt::{self};
use cursive::Cursive;
use cursive::Printer;
use cursive::vec::Vec2;
use cursive::views::{Dialog, LinearLayout, Panel, TextView};
use cursive::event::{Event, EventResult, Key};
use cursive::direction::Direction;
use cursive::theme::{BaseColor, Color, ColorStyle};
use rand::{Rng, thread_rng};
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::{Read, Write};
use std::cmp::Ordering;

fn main() {
	fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
	.chain(fern::log_file("program.log").expect("Error setting up logs")).apply().expect("Error setting up logs");

	let mut siv = Cursive::new();

	siv.add_global_callback('q', |s| s.quit());

    let board_view = BoardView::new();

    siv.add_layer(Dialog::new()
				  .title("2048")
                  .content(
                      LinearLayout::vertical()
                        .child(Panel::new(board_view))
						.child(TextView::new("
Control with arrows
Undo move: u
Save game: k
Load game: l
Quit game: q"))
                  )
                 //.button("Save game", move |s| {serialize_board(&board_view.board); 
                 //      s.add_layer(Dialog::text("You have won!")
                 //          .button("Continue", |s| {s.pop_layer();}))
                 //  })
                  .button("Quit", |s| s.quit())
                  );

	siv.run();
}


#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Eq, PartialOrd)]
enum Cell {
    Occupied(usize),
    Empty,
}

impl Ord for Cell{
	fn cmp(&self, other: &Cell) -> Ordering{
		match self{
			&Cell::Occupied(n) => {
				match other {
					&Cell::Occupied(m) => n.cmp(&m),
					&Cell::Empty => Ordering::Greater,
				}
			}
			&Cell::Empty => { 
				match other {
					&Cell::Occupied(_) => Ordering::Less,
					&Cell::Empty => Ordering::Equal,
				}
			}
		}
	}
}

#[derive(Clone, Copy, PartialEq)]
enum MoveDirection {
	Up,
	Down,
	Left,
	Right,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			&Cell::Occupied(val) => { return write!(f, "{}", val); }
			&Cell::Empty => write!(f, "{}", 0)
		}
    }
}

#[derive(Clone, PartialEq)]
struct BoardView { 
    board: Vec<Cell>,
    size: Vec2,
	number_of_cells: usize,
	size_x: usize,
	size_y: usize,
    has_won: bool,
	previous_boards: Vec<Vec<Cell>>,
}

const SIZE_X : usize = 6;
const SIZE_Y : usize = 6;
const STARTING_CELL_NUMBER: usize = 2;
const WINNING_CELL_NUMBER: usize = 2048;
const MAX_NUMBER_OF_NEW_CELLS_TO_ADD : usize = 2;
const NUMBER_OF_FILLED_CELL_AT_START: usize = 4;
const CHANCE_OF_ADDING_CELLS: usize = 4; // Chance is calculated as 1/CHANCE_OF_ADDING_CELLS.

impl BoardView {
    pub fn new() -> Self{
		let number_of_cells = SIZE_X * SIZE_Y;
        let board = vec![Cell::Empty; number_of_cells];
        let size = Vec2::new(SIZE_X, SIZE_Y);
		let size_x = SIZE_X;
		let size_y = SIZE_Y;
        let has_won = false;
		let previous_boards = Vec::new();
        let mut board_view = BoardView {
            board,
            size,
		    number_of_cells,
		    size_x,
			size_y,
            has_won,
			previous_boards,
        };
        board_view.set_up_board();
        board_view
    }

    fn set_up_board(&mut self){
		for _ in 0..NUMBER_OF_FILLED_CELL_AT_START{
			let cell: usize = thread_rng().gen_range(0, self.number_of_cells);
			self.board[cell] = Cell::Occupied(STARTING_CELL_NUMBER);
		}
    }
	
	fn maybe_add_new_cells(&mut self){
		let chance = thread_rng().gen_range(0, CHANCE_OF_ADDING_CELLS);
		if chance == 0 {
			let current_num_of_filled = self.number_of_filled_cels();
            let num_of_free = self.number_of_cells - current_num_of_filled;
			if num_of_free != 0{
				let cells_to_add = std::cmp::min(num_of_free, MAX_NUMBER_OF_NEW_CELLS_TO_ADD);
				for _ in 0..cells_to_add{
					loop{
						let cell: usize = thread_rng().gen_range(0, self.number_of_cells);
						match self.board[cell] {
							Cell::Empty => {
								self.board[cell] = Cell::Occupied(STARTING_CELL_NUMBER);
								break;
							}
							Cell::Occupied(_) => ()
						}
					}
				}
			}
		}
 
	}
    
    fn number_of_filled_cels(&mut self) -> usize{
        self.board.iter().filter(|&x| {
            match x {
                &Cell::Occupied(_) => {true}
                _ => {false}
            }
        }).count()

    }

	fn can_move(&mut self) -> bool{
        if self.number_of_filled_cels() < self.number_of_cells{
            return true;
        } else {
            for direction in [MoveDirection::Down, MoveDirection::Up, MoveDirection::Left, MoveDirection::Right].iter(){
                let mut modifications = self.move_cells(*direction);
                let current_board = self.board.to_vec();
                let (_, applied_modifications) = self.apply_modifications(current_board, &mut modifications, *direction);
                if applied_modifications != 0{
                    return true;
                }

            }
        }
		return false;
	}

	fn sort_modifications(&mut self, modifications: &mut Vec<(usize, usize, Cell)>, direction: MoveDirection){
		match direction {
			MoveDirection::Up => {
				modifications.sort_by(|a, b| {
					let row_a = a.0 / self.size_y;
					let row_b = b.0 / self.size_y;
					row_a.cmp(&row_b)
				})
			}
			MoveDirection::Down => {
				modifications.sort_by(|a, b| {
					let row_a = a.0 / self.size_y;
					let row_b = b.0 / self.size_y;
					row_b.cmp(&row_a)
				})
			}
			MoveDirection::Left => {
				modifications.sort_by(|a, b| {
					let col_a = a.0 % self.size_x;
					let col_b = b.0 % self.size_x;
					col_a.cmp(&col_b)
				})
			}
			MoveDirection::Right => {
				modifications.sort_by(|a, b| {
					let col_a = a.0 % self.size_x;
					let col_b = b.0 % self.size_x;
					col_b.cmp(&col_a)
				}) 
			}
		}
	}

    fn check_if_won(&mut self) -> bool {
        self.board.iter().find(|&cell| match *cell {
            Cell::Occupied(WINNING_CELL_NUMBER) => true,
            _ => false,
        }).is_some()
    }

	fn apply_modifications(&mut self, mut board: Vec<Cell>, mut modifications: &mut Vec<(usize, usize, Cell)>, direction: MoveDirection) -> (Vec<Cell>, usize) {
		let mut applied_modifications = 0;
		self.sort_modifications(&mut modifications, direction);	
		for &(prev_i, i, cell) in modifications.iter(){
			match cell {
				Cell::Occupied(new_val) => {
					let current_cell = board[i];
					match current_cell {
						Cell::Occupied(val) => {
							if val == new_val {
								board[i] = Cell::Occupied(new_val + val);
								board[prev_i] = Cell::Empty;
								applied_modifications += 1;
							}
						}
						Cell::Empty => {
							board[i] = cell;
							board[prev_i] = Cell::Empty;
							applied_modifications += 1;
						}
					}
				}
				Cell::Empty => ()
			}
		}
        (board, applied_modifications)
	}

	fn save_previous_board(&mut self) {
		self.previous_boards.push(self.board.clone());
	}

    fn process_action(&mut self, direction: MoveDirection) -> EventResult {
		self.save_previous_board();
        let mut modifications = self.move_cells(direction);
        let current_board = self.board.clone();
		let (updated_board, applied_modifications) = self.apply_modifications(current_board, &mut modifications, direction);
        self.board = updated_board;
		if applied_modifications != 0{
			self.maybe_add_new_cells();
		}
        if !self.has_won && self.check_if_won(){
            self.has_won = true; 
			return EventResult::with_cb(|s| {
                        s.add_layer(Dialog::text("You have won!")
                            .button("Continue", |s| {s.pop_layer();})
                            .button("Quit", |s| {s.quit();})
                        );
			})	
        }
        
		if !self.can_move(){
			return EventResult::with_cb(|s| {
                        s.add_layer(Dialog::text("No more moves left!").button("Quit", |s| {
                            s.quit();
                        }));
			})	
		} else {
			return EventResult::Consumed(None)
		}
    }

	fn undo_move(&mut self){
		let previous_board = self.previous_boards.pop();
		if previous_board.is_some(){
			self.board = previous_board.unwrap();
		}
		
	}


	fn move_cells(&mut self, direction: MoveDirection) -> Vec<(usize, usize, Cell)> {
		let mut modifications:Vec<(usize, usize, Cell)> = vec![];
		for (i, cell) in self.board.iter().enumerate() {
			match cell {
				&Cell::Occupied(_) => {
					match direction {
						MoveDirection::Right => {
							let current_col = i % self.size_x;
							if current_col + 1 < self.size_x{
								modifications.push((i, i + 1, *cell));
							}
						}
						MoveDirection::Left => {
							let current_col = i % self.size_x;
							if current_col > 0{
								modifications.push((i, i - 1, *cell));
							}
						}
						MoveDirection::Up => {
							let current_row = i / self.size_y;
							if current_row > 0{
								modifications.push((i, i - self.size_x, *cell));
							}
						}
						MoveDirection::Down => {
							let current_row = i / self.size_y;
							if current_row < self.size_y - 1{
								modifications.push((i, i + self.size_x, *cell));
							}
						}
					}
				}
				&Cell::Empty => ()
			}
		}
        modifications
	}
}

fn colorise(cell: Cell) -> Color {
    let color = match cell {
        Cell::Occupied(2) => Color::Dark(BaseColor::White),
        Cell::Occupied(4) => Color::Dark(BaseColor::Yellow),
        Cell::Occupied(8) => Color::Dark(BaseColor::Green),
        Cell::Occupied(16) => Color::Dark(BaseColor::Cyan),
        Cell::Occupied(32) => Color::Dark(BaseColor::Blue),
        Cell::Occupied(64) => Color::Dark(BaseColor::Magenta),
        Cell::Occupied(128) => Color::Dark(BaseColor::Red),
        Cell::Occupied(256) => Color::Light(BaseColor::Yellow),
        Cell::Occupied(512) => Color::Rgb(0,153,0),
        Cell::Occupied(1024) => Color::Light(BaseColor::Cyan),
        Cell::Occupied(2048) => Color::Light(BaseColor::Blue),
        Cell::Occupied(4096) => Color::Light(BaseColor::Magenta),
        Cell::Occupied(8192) => Color::Light(BaseColor::Red),
        Cell::Occupied(_) => {
            Color::Rgb(255,0,0)
        }
        Cell::Empty => Color::Dark(BaseColor::White),
    };
    color
}

fn required_x_padding(board: &Vec<Cell>)-> usize {
	let mut max_so_far = 0;
	for cell in board.iter(){
		match cell {
			&Cell::Occupied(n) => {
				if n > max_so_far{
					max_so_far = n;
				}
			}
			&Cell::Empty => ()
		}
	}
	let str_val: String = max_so_far.to_string();
	std::cmp::max(str_val.len(), 2) + 1
}

impl cursive::view::View for BoardView {
    fn draw(&self, printer: &Printer){
		let padding_x = required_x_padding(&self.board);
        for (i, cell) in self.board.iter().enumerate() {
            let x = (i % self.size_x) * padding_x;
            let y = i / self.size_y;
            let color = colorise(*cell);
            printer.with_color(
                ColorStyle::Custom {
                    back: color,
                    front: Color::Dark(BaseColor::Black),
                },
                |printer| printer.print((x, y), &cell.to_string()),
            )
        }
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
		let padding_x = required_x_padding(&self.board);
        self.size.map_x(|x| padding_x*x)
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }

    fn on_event(&mut self, event: Event) -> EventResult {
		match event {
			Event::Char{0: 'd'} | Event::Key(Key::Right) => {
				self.process_action(MoveDirection::Right)
			}
			Event::Char {0: 'a'} | Event::Key(Key::Left) => {
				self.process_action(MoveDirection::Left)
			}
			Event::Char{0: 'w'} | Event::Key(Key::Up) => {
				self.process_action(MoveDirection::Up)
			}
			Event::Char{0: 's'} | Event::Key(Key::Down) => {
				self.process_action(MoveDirection::Down)
			}
            Event::Char{0: 'k'} => {
                save_game(&self.board);
                EventResult::with_cb(|s| {
                        s.add_layer(Dialog::text("Game saved!").button("Continue", |s| {
                            s.pop_layer();
                        }))
                })
            }
            Event::Char{0: 'l'} => {
                let board = load_game();
				self.board = board;
                EventResult::with_cb(|s| {
                        s.add_layer(Dialog::text("Game loaded!").button("Continue", |s| {
                            s.pop_layer();
                        }))
                })
            }
            Event::Char{0: 'u'} => {
				self.undo_move();
				EventResult::Consumed(None)
            }
			_ => EventResult::Ignored
		}
	}
}

fn serialize_board(board: &Vec<Cell>) -> String{
    serde_json::to_string_pretty(&board).unwrap()
}

fn save_game(board: &Vec<Cell>) {
    let serialized = serialize_board(&board);
    let path = Path::new("save_game.json");
    let display = path.display();
    let mut file: File = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why.description()),
        Ok(file) => file,
    };
	match file.write_all(serialized.as_bytes()) {
		Err(why) => {
			panic!("couldn't write to {}: {}", display,
											   why.description())
		},
		Ok(_) => println!("successfully wrote to {}", display),
	}
}

fn deserialize_board(serialized_board: String) -> Vec<Cell> {
	serde_json::from_str(&serialized_board).unwrap()
}

fn load_game() -> Vec<Cell>{
    let path = Path::new("save_game.json");
    let display = path.display();
    let mut file: File = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };
	let mut serialized_board = String::new();
	match file.read_to_string(&mut serialized_board) {
		Err(why) => {
			panic!("couldn't read from {}: {}", display,
											   why.description())
		},
		Ok(_) => println!("successfully read from {}", display),
	}
	let board = deserialize_board(serialized_board);
	board	
}
