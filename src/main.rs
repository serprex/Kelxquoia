extern crate fnv;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use fnv::FnvHashSet;

fn main() {
	if let Some(a) = env::args().nth(1) {
		let mut field = Vec::new();
		let f = fs::File::open(&a).expect("Failed to open file");
		let mut sxy: Option<(usize, usize)> = None;
		let f = BufReader::new(f);
		for (y, line) in f.lines().enumerate() {
			if let Ok(line) = line {
				for (x, c) in line.chars().enumerate() {
					if c == '$' {
						if sxy.is_some() {
							println!("Duplicate $");
							return
						}
						sxy = Some((x, y));
					}
				}
				field.push(line.chars().collect::<Vec<char>>());
			}
		}
		let height = field.len();
		let mut width = 0;
		for row in field.iter() {
			if row.len() > width { width = row.len() }
		}
		let mut xy = if let Some(sxy) = sxy {
			sxy
		} else {
			println!("No $");
			return
		};
		let mut stack = Vec::new();
		let mut dir = Dir::E;
		loop {
			let quote = match dir {
				Dir::E => {
					xy.0 += 1;
					if xy.0 == field[xy.1].len() { break }
					xy.1+1 < height && xy.0 < field[xy.1+1].len() && field[xy.1+1][xy.0] == '\''
				},
				Dir::W => {
					if xy.0 == 0 { break }
					xy.0 -= 1;
					xy.1 > 0 && xy.0 < field[xy.1-1].len() && field[xy.1-1][xy.0] == '\''
				},
				Dir::S => {
					xy.1 += 1;
					if xy.1 == field.len() { break }
					xy.0 > 0 && field[xy.1][xy.0-1] == '\''
				},
				Dir::N => {
					if xy.1 == 0 { break }
					xy.1 -= 1;
					xy.0+1 < field[xy.1].len() && field[xy.1][xy.0+1] == '\''
				},
			};
			let ch = field[xy.1][xy.0];
			field[xy.1][xy.0] = ' ';
			if quote {
				if let Some(&mut Cell::Row(ref mut row)) = stack.last_mut() {
					row.push(ch);
				}
			} else {
				match ch {
					'-' => stack.push(Cell::Row(Default::default())),
					'+' => stack.push(Cell::Grid(Default::default())),
					'*' => {
						let len = stack.len();
						if len >= 2 && match stack[len-1] {
							Cell::Row(_) =>
								match stack[len-2] {
									Cell::Grid(_) => true,
									_ => false,
								},
							_ => false,
						} {
							if let Some(Cell::Row(row)) = stack.pop() {
								if let Some(&mut Cell::Grid(ref mut grid)) = stack.last_mut() {
									grid.add(row);
								}
							}
						}
					},
					'?' => {
						if let Some(&mut Cell::Row(ref mut row)) = stack.last_mut() {
							row.push('\0');
						}
					},
					'/' => {
						let len = stack.len();
						if len >= 2 && match stack[len-1] {
							Cell::Grid(_) =>
								match stack[len-2] {
									Cell::Grid(_) => true,
									_ => false,
								},
							_ => false,
						} {
							if let Some(Cell::Grid(rep)) = stack.pop() {
								if let Some(Cell::Grid(pat)) = stack.pop() {
									if rep.cols <= pat.cols && rep.rows.len() <= pat.rows.len() {
										let mut patwild = 0;
										for row in pat.rows.iter() {
											for &c in row.iter() {
												if c == '\0' {
													patwild += 1;
												}
											}
										}
										if patwild < 2 {
											let mut repwild = 0;
											for row in rep.rows.iter() {
												for &c in row.iter() {
													if c == '\0' {
														repwild += 1;
													}
												}
											}
											if repwild <= patwild {
												let mut matches = Vec::new();
												let mut wildch = '\0';
												for my in 0..height {
													'nextmatch:
													for mx in 0..width {
														for (py, row) in pat.rows.iter().enumerate() {
															if my + py == height { continue 'nextmatch }
															for (px, &pch) in row.iter().enumerate() {
																if mx + px == width { continue 'nextmatch }
																let mch = *field[my + py].get(mx + px).unwrap_or(&' ');
																if pch == '\0' {
																	wildch = mch;
																	continue
																}
																if pch != ' ' && mch != pch {
																	continue 'nextmatch
																}
															}
														}
														matches.push((mx, my, wildch));
													}
												}
												let pwidth = pat.cols;
												let pheight = pat.rows.len();
												let mut overlap = FnvHashSet::default();
												for (idx1, &(x1, y1, _)) in matches.iter().enumerate() {
													for (idx2, &(x2, y2, _)) in matches[idx1+1..].iter().enumerate() {
														if x1 <= x2 && y1 <= y2 && x1 + pwidth > x2 && y1 + pheight > y2 {
															overlap.insert(idx1);
															overlap.insert(idx1 + 1 + idx2);
														}
													}
												}
												for (idx, &(x, y, wc)) in matches.iter().enumerate() {
													if !overlap.contains(&idx) {
														for (ry, row) in rep.rows.iter().enumerate() {
															for (rx, &rch) in row.iter().enumerate() {
																let ch = if rch == '\0' { wc } else { rch };
																if x + rx < field[y + ry].len() {
																	field[y + ry][x + rx] = ch;
																} else {
																	while x + rx - 1 > field[y + ry].len() {
																		field[y + ry].push(' ');
																	}
																	field[y + ry].push(ch);
																}
															}
														}
													}
												}
											}
										}
									}
								}
							}
						}
					},
					'>' => dir = Dir::E,
					'<' => dir = Dir::W,
					'v' => dir = Dir::S,
					'^' => dir = Dir::N,
					'!' => stack.clear(),
					_ => (),
				}
			}
		}
		for row in field {
			for c in row {
				print!("{}", c);
			}
			println!("");
		}
	} else {
		println!("kelxquoia [filename]")
	}
}

#[derive(Copy, Clone)]
enum Dir { E, N, S, W }

enum Cell {
	Grid(Grid),
	Row(Vec<char>),
}

#[derive(Default)]
struct Grid {
	pub cols: usize,
	pub rows: Vec<Vec<char>>,
}

impl Grid {
	fn add(&mut self, row: Vec<char>) {
		if row.len() > self.cols {
			self.cols = row.len();
		}
		self.rows.push(row);
	}
}
