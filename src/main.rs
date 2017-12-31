extern crate cursive;
extern crate rand;

use std::fmt::{self};
use cursive::Cursive;
use cursive::Printer;
use cursive::vec::Vec2;
use cursive::views::{Dialog, LinearLayout, Panel};
use cursive::event::{Event, EventResult, Key};
use cursive::direction::Direction;

use rand::{Rng, thread_rng};

fn main() {
	let mut siv = Cursive::new();

	siv.add_global_callback('q', |s| s.quit());

    siv.add_layer(Dialog::new()
				  .title("2048")
                  .content(
                      LinearLayout::horizontal()
                        .child(Panel::new(BoardView::new()))
                  ));

	siv.run();
}

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Occupied(usize),
    Empty,
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

struct BoardView { 
    board: Vec<Cell>,
    size: Vec2,
	number_of_cells: usize,
	size_x: usize,
	size_y: usize,
}

const SIZE_X : usize = 6;
const SIZE_Y : usize = 6;

impl BoardView {
    pub fn new() -> Self{
		let number_of_cells = SIZE_X * SIZE_Y;
        let mut board = vec![Cell::Empty; number_of_cells];
		for _ in 0..4{
			let cell: usize = thread_rng().gen_range(0, number_of_cells);
			board[cell] = Cell::Occupied(2);
		}
        let size = Vec2::new(SIZE_X, SIZE_Y);
		let size_x = SIZE_X;
		let size_y = SIZE_Y;
        BoardView {
            board,
            size,
		    number_of_cells,
		    size_x,
			size_y,
        }
    }
	
	fn maybe_add_new_cells(&mut self){
		let chance = thread_rng().gen_range(0, 4);
		if chance == 0 {
			let current_num_of_filled = self.board.iter().filter(|&x| {
				match x {
					&Cell::Occupied(_) => {true}
					_ => {false}
				}
			}).count();
			let num_of_free = self.number_of_cells - current_num_of_filled;
			if num_of_free != 0{
				let cells_to_add = std::cmp::min(num_of_free, 2);
				for _ in 0..cells_to_add{
					loop{
						let cell: usize = thread_rng().gen_range(0, self.number_of_cells);
						match self.board[cell] {
							Cell::Empty => {
								self.board[cell] = Cell::Occupied(2);
								break;
							}
							Cell::Occupied(_) => ()
						}
					}
				}
			}
		}
 
	}

	fn can_move(&mut self) -> bool{
		for cell in self.board.iter(){
			match cell {
				&Cell::Empty => {return true;}
				_ => ()
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

	fn apply_modifications(&mut self, mut modifications: &mut Vec<(usize, usize, Cell)>, direction: MoveDirection) {
		let mut applied_modifications = 0;
		self.sort_modifications(&mut modifications, direction);	
		for &(prev_i, i, cell) in modifications.iter(){
			match cell {
				Cell::Occupied(new_val) => {
					let current_cell = self.board[i];
					match current_cell {
						Cell::Occupied(val) => {
							if val == new_val {
								self.board[i] = Cell::Occupied(new_val + val);
								self.board[prev_i] = Cell::Empty;
								applied_modifications += 1;
							}
						}
						Cell::Empty => {
							self.board[i] = cell;
							self.board[prev_i] = Cell::Empty;
							applied_modifications += 1;
						}
					}
				}
				Cell::Empty => ()
			}
		}
		if applied_modifications != 0{
			self.maybe_add_new_cells();
		}
	}

	fn move_cells(&mut self, direction: MoveDirection) -> EventResult {
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
		self.apply_modifications(&mut modifications, direction);
		if !self.can_move(){
			return EventResult::with_cb(|s| {
                        s.add_layer(Dialog::text("No more moves left!").button("Ok", |s| {
                            s.pop_layer();
                            s.pop_layer();
                        }));
			})	
		} else {
			return EventResult::Consumed(None)
		}
	}
}


impl cursive::view::View for BoardView {
    fn draw(&self, printer: &Printer){
        for (i, cell) in self.board.iter().enumerate() {
            let x = (i % self.size.x) * 4;
            let y = i / self.size.y;
            printer.print((x, y), &cell.to_string());
        }
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        self.size.map_x(|x| 4*x)
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }

    fn on_event(&mut self, event: Event) -> EventResult {
		match event {
			Event::Char{0: 'd'} | Event::Key(Key::Right) => {
				self.move_cells(MoveDirection::Right)
			}
			Event::Char {0: 'a'} | Event::Key(Key::Left) => {
				self.move_cells(MoveDirection::Left)
			}
			Event::Char{0: 'w'} | Event::Key(Key::Up) => {
				self.move_cells(MoveDirection::Up)
			}
			Event::Char{0: 's'} | Event::Key(Key::Down) => {
				self.move_cells(MoveDirection::Down)
			}
			_ => EventResult::Ignored
		}
	}
}
