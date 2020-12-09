use fxhash::FxHashSet;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::usize;

fn main() {
	if let Some(a) = env::args().nth(1) {
		print!("{}", run(fs::File::open(&a).expect("Failed to open file")));
	} else {
		println!("kelxquoia [filename]")
	}
}

fn run<R>(f: R) -> String
where
	R: std::io::Read,
{
	let f = BufReader::new(f);
	let mut field = Vec::new();
	let mut xy = (usize::MAX, usize::MAX);
	for (y, line) in f.lines().enumerate() {
		if let Ok(line) = line {
			for (x, c) in line.chars().enumerate() {
				if c == '$' {
					if xy != (usize::MAX, usize::MAX) {
						return String::from("Duplicate $\n");
					}
					xy = (x, y);
				}
			}
			field.push(line.chars().collect::<Vec<char>>());
		}
	}
	let mut width = 0;
	for row in field.iter() {
		if row.len() > width {
			width = row.len()
		}
	}
	if xy == (usize::MAX, usize::MAX) {
		return String::from("No $\n");
	}
	let mut stack = Vec::new();
	let mut dir = Dir::E;
	loop {
		let quote = match dir {
			Dir::E => {
				xy.0 += 1;
				if xy.0 == field[xy.1].len() {
					break;
				}
				xy.1 + 1 < field.len()
					&& xy.0 < field[xy.1 + 1].len()
					&& field[xy.1 + 1][xy.0] == '\''
			}
			Dir::W => {
				if xy.0 == 0 {
					break;
				}
				xy.0 -= 1;
				xy.1 > 0 && xy.0 < field[xy.1 - 1].len() && field[xy.1 - 1][xy.0] == '\''
			}
			Dir::S => {
				xy.1 += 1;
				if xy.1 == field.len() {
					break;
				}
				xy.0 > 0 && field[xy.1][xy.0 - 1] == '\''
			}
			Dir::N => {
				if xy.1 == 0 {
					break;
				}
				xy.1 -= 1;
				xy.0 + 1 < field[xy.1].len() && field[xy.1][xy.0 + 1] == '\''
			}
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
					if len >= 2
						&& match stack[len - 1] {
							Cell::Row(_) => match stack[len - 2] {
								Cell::Grid(_) => true,
								_ => false,
							},
							_ => false,
						} {
						if let Some(Cell::Row(row)) = stack.pop() {
							if let Some(&mut Cell::Grid(ref mut grid)) = stack.last_mut() {
								grid.push(row);
							}
						}
					}
				}
				'?' => {
					if let Some(&mut Cell::Row(ref mut row)) = stack.last_mut() {
						row.push('\0');
					}
				}
				'/' => {
					let len = stack.len();
					if len >= 2
						&& match stack[len - 1] {
							Cell::Grid(_) => match stack[len - 2] {
								Cell::Grid(_) => true,
								_ => false,
							},
							_ => false,
						} {
						if let Some(Cell::Grid(rep)) = stack.pop() {
							if let Some(Cell::Grid(pat)) = stack.pop() {
								if rep.len() <= pat.len() {
									let repcols = rep.iter().map(|v| v.len()).max().unwrap_or(0);
									let patcols = pat.iter().map(|v| v.len()).max().unwrap_or(0);
									if repcols <= patcols {
										let mut patwild = 0;
										for row in pat.iter() {
											for &c in row.iter() {
												if c == '\0' {
													patwild += 1;
												}
											}
										}
										if patwild < 2 {
											let mut repwild = 0;
											for row in rep.iter() {
												for &c in row.iter() {
													if c == '\0' {
														repwild += 1;
													}
												}
											}
											if repwild <= patwild {
												let mut matches = Vec::new();
												let mut wildch = '\0';
												for my in 0..field.len() {
													'nextmatch: for mx in 0..width {
														for (py, row) in pat.iter().enumerate() {
															for (px, &pch) in row.iter().enumerate()
															{
																let mch = *field[my + py]
																	.get(mx + px)
																	.unwrap_or(&' ');
																if pch == '\0' {
																	wildch = mch;
																	continue;
																}
																if pch != ' '
																	&& (mx + px == width
																		|| my + py == field.len()
																		|| mch != pch)
																{
																	continue 'nextmatch;
																}
															}
														}
														matches.push((mx, my, wildch));
													}
												}
												let pwidth = patcols;
												let pheight = pat.len();
												let mut overlap = FxHashSet::default();
												for (idx1, &(x1, y1, _)) in
													matches.iter().enumerate()
												{
													for (idx2, &(x2, y2, _)) in
														matches[idx1 + 1..].iter().enumerate()
													{
														if x1 <= x2
															&& y1 <= y2 && x1 + pwidth > x2 && y1
															+ pheight
															> y2
														{
															overlap.insert(idx1);
															overlap.insert(idx1 + 1 + idx2);
														}
													}
												}
												for (idx, &(x, y, wc)) in matches.iter().enumerate()
												{
													if !overlap.contains(&idx) {
														for (ry, row) in rep.iter().enumerate() {
															if y + ry == field.len() {
																field.push(Vec::with_capacity(
																	x + row.len(),
																));
															}
															while x >= field[y + ry].len() {
																field[y + ry].push(' ');
															}
															for (rx, &rch) in row.iter().enumerate()
															{
																let ch = if rch == '\0' {
																	wc
																} else {
																	rch
																};
																if x + rx < field[y + ry].len() {
																	field[y + ry][x + rx] = ch;
																} else {
																	field[y + ry].push(ch);
																	if field[y + ry].len() > width {
																		width += 1;
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
						}
					}
				}
				'>' => dir = Dir::E,
				'<' => dir = Dir::W,
				'v' => dir = Dir::S,
				'^' => dir = Dir::N,
				'!' => stack.clear(),
				_ => (),
			}
		}
	}
	let mut ret = String::new();
	for row in field {
		for c in row {
			ret.push(c);
		}
		ret.push('\n');
	}
	ret
}

#[derive(Copy, Clone)]
enum Dir {
	E,
	N,
	S,
	W,
}

enum Cell {
	Grid(Vec<Vec<char>>),
	Row(Vec<char>),
}

#[cfg(test)]
mod tests {
	#[test]
	fn bob() {
		let input = br#"$+-W*-P*+-B*-M*/
   '  '   '  '
WOW
POP"#;
		assert_eq!(
			super::run(&input[..]),
			"$               \n   \'  \'   \'  \'\nBOB\nMOM\n"
		);
	}

	#[test]
	fn rrrr() {
		let input = br#" +-0 0*+-1*/+-?*-R*- *+-?*-R*-?*/
 RRRRRRRRRRRRRRRRRRR RRRRRRRRRRRR
$+-0 0*+-1*/+-?*-R*- *+-?*-R*-?*/
   ' '   '       '  '      '   

 00 00 00 00"#;
		assert_eq!(super::run(&input[..]), " +-0 0*+-1*/+-?*-R*- *+-?*-R*-?*/\n RRRRRRRRRRRRRRRRRRR RRRRRRRRRRRR\n$                                \n   \' \'   \'       \'  \'      \'   \n\n 10 10 10 10\n");
	}
}
