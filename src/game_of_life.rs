use std::fmt::{Display, Formatter, Write};
use std::mem;
use std::num::NonZeroUsize;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CellValue {
    Dead = 0,
    Alive = 1,
}
impl CellValue {
    fn other(&self) -> Self {
        match self {
            CellValue::Dead => CellValue::Alive,
            CellValue::Alive => CellValue::Dead,
        }
    }
}

pub struct Field {
    width: NonZeroUsize,
    height: NonZeroUsize,
    cells: Vec<CellValue>,
    swap_cells: Vec<CellValue>,
}

impl Field {
    pub fn new(width: NonZeroUsize, height: NonZeroUsize) -> Self {
        let cell_count = width.get() * height.get();
        Self {
            width,
            height,
            cells: vec![CellValue::Dead; cell_count],
            swap_cells: vec![CellValue::Dead; cell_count],
        }
    }
    pub fn generate_by_fn(width: NonZeroUsize, height: NonZeroUsize, random_bool: impl Fn(usize) -> bool) -> Self {
        let cell_count = width.get() * height.get();

        let cells = (0..cell_count)
            .map(|i| {
                if random_bool(i) {
                    CellValue::Alive
                } else {
                    CellValue::Dead
                }
            })
            .collect();

        Self {
            width,
            height,
            cells,
            swap_cells: vec![CellValue::Dead; cell_count],
        }
    }
    pub fn get_height(&self) -> usize {
        self.height.get()
    }
    pub fn get_width(&self) -> usize {
        self.width.get()
    }
    fn get_by_coords(&self, row: usize, col: usize) -> Option<CellValue> {
        let index = self.coords_to_index_checked(row, col)?;
        Some(self.cells[index])
    }
    fn set_by_coords(&mut self, row: usize, col: usize, value: CellValue) -> Option<()> {
        let index = self.coords_to_index_checked(row, col)?;
        self.cells[index] = value;
        Some(())
    }
    pub fn toggle_by_coords(&mut self, row: usize, col: usize) -> Option<()> {
        let index = self.coords_to_index_checked(row, col)?;
        self.cells[index] = self.cells[index].other();
        Some(())
    }
    fn coords_to_index_checked(&self, row: usize, col: usize) -> Option<usize> {
        if row >= self.width.get() {
            return None;
        }
        if col >= self.height.get() {
            return None;
        }
        let index = self.coords_to_index_unchecked(row, col);
        Some(index)
    }
    fn coords_to_index_unchecked(&self, row: usize, col: usize) -> usize {
        (row * self.width.get()) + col
    }
    pub fn view(&self) -> &[CellValue] {
        &self.cells
    }
    pub fn view_old(&self) -> &[CellValue] {
        &self.swap_cells
    }
    pub fn rows(&self) -> impl Iterator<Item=&[CellValue]> + '_ {
        let width = self.width.get();
        self.cells.chunks(width)
    }
    pub fn rows_with_old(&self) -> impl Iterator<Item=(&[CellValue], &[CellValue])> + '_ {
        let width = self.width.get();
        self.cells.chunks(width).zip(self.swap_cells.chunks(width))
    }
    pub fn update(&mut self) -> bool {
        let max_col = self.width.get() - 1;
        let max_row = self.height.get() - 1;

        let width = self.width.get();
        let mut has_alive = false;
        for (row_no, row) in self.cells.chunks(width).enumerate() {
            for (col_no, &value) in row.iter().enumerate() {
                let live_neighbours = self.count_live_neighbours(row_no, col_no, max_row, max_col);
                let index = self.coords_to_index_unchecked(row_no, col_no);
                let new_value = Self::calc_new_value(value, live_neighbours);
                self.swap_cells[index] = new_value;
                has_alive = has_alive || (new_value == CellValue::Alive);
            }
        }
        mem::swap(&mut self.cells, &mut self.swap_cells);
        has_alive
    }
    fn count_live_neighbours(&self, row: usize, col: usize, max_row: usize, max_col: usize) -> u8 {
        let mut count = 0;
        let row_top = Self::prev_coord_wrapped(row, max_row);
        let row_bottom = Self::next_coord_wrapped(row, max_row);
        let col_left = Self::prev_coord_wrapped(col, max_col);
        let col_right = Self::next_coord_wrapped(col, max_col);
        count += self.cells[self.coords_to_index_unchecked(row_top, col_left)] as u8;
        count += self.cells[self.coords_to_index_unchecked(row_top, col)] as u8;
        count += self.cells[self.coords_to_index_unchecked(row_top, col_right)] as u8;
        count += self.cells[self.coords_to_index_unchecked(row, col_left)] as u8;
        count += self.cells[self.coords_to_index_unchecked(row, col_right)] as u8;
        count += self.cells[self.coords_to_index_unchecked(row_bottom, col_left)] as u8;
        count += self.cells[self.coords_to_index_unchecked(row_bottom, col)] as u8;
        count += self.cells[self.coords_to_index_unchecked(row_bottom, col_right)] as u8;
        count
    }
    fn next_coord_wrapped(value: usize, max_value: usize) -> usize {
        if value >= max_value {
            0
        } else {
            value + 1
        }
    }
    fn prev_coord_wrapped(value: usize, max_value: usize) -> usize {
        if value == 0 {
            max_value
        } else {
            value - 1
        }
    }
    fn count_live_neighbours_slow(&self, row: usize, col: usize) -> u8 {
        let width = self.width.get();
        let height = self.height.get();
        let mut count = 0;
        for delta_row in [height - 1, 0, 1] {
            for delta_col in [width - 1, 0, 1] {
                if (delta_row == 0) && (delta_col == 0) {
                    continue;
                }
                let check_row = (row + delta_row) % height;
                let check_col = (col + delta_col) % width;
                let check_index = self.coords_to_index_unchecked(check_row, check_col);
                count += self.cells[check_index] as u8;
            }
        }
        count
    }
    fn calc_new_value(old_value: CellValue, live_neighbours: u8) -> CellValue {
        match old_value {
            CellValue::Alive => if (live_neighbours == 2) || (live_neighbours == 3) {
                CellValue::Alive
            } else {
                CellValue::Dead
            },
            CellValue::Dead => if live_neighbours == 3 {
                CellValue::Alive
            } else {
                CellValue::Dead
            }
        }
    }
    pub fn from_str(str: &str) -> Result<Self, ParseError> {
        let str = str.trim();
        if str.is_empty() {
            return Err(ParseError::EmptyString);
        };
        let mut lines = Vec::new();
        let mut width = None;
        for str_line in str.lines() {
            let line = Self::from_str_line(str_line, width)?;
            let line_width = line.len();
            let expected_width = *width.get_or_insert(line_width);
            if line_width != expected_width {
                return Err(ParseError::WidthMismatch);
            }
            lines.push(line);
        }
        let height = lines.len();
        let cells = lines.concat();
        let cells_len = cells.len();
        let res = Self {
            width: width.unwrap().try_into().unwrap(),
            height: height.try_into().unwrap(),
            cells,
            swap_cells: vec![CellValue::Dead; cells_len],
        };
        Ok(res)
    }
    fn from_str_line(str: &str, expected_width: Option<usize>) -> Result<Vec<CellValue>, ParseError> {
        let mut vec = Vec::with_capacity(expected_width.unwrap_or(0));
        for char in str.trim().chars() {
            let val = if char == '#' {
                CellValue::Alive
            } else if char == '_' {
                CellValue::Dead
            } else {
                return Err(ParseError::UnknownChar);
            };
            vec.push(val);
        }
        Ok(vec)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParseError {
    EmptyString,
    UnknownChar,
    WidthMismatch,
}


impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = self.width.get();
        for row in self.cells.chunks(width) {
            for &value in row.iter() {
                let char = if value == CellValue::Alive { '#' } else { '_' };
                f.write_char(char)?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_string_parse() {
        let field_str =
"_____
_###_
#____
____#
__#__
";
        let field = Field::from_str(field_str).unwrap();
        assert_eq!(field_str, field.to_string());
        assert_eq!(CellValue::Dead, field.get_by_coords(0, 0).unwrap());
        assert_eq!(CellValue::Alive, field.get_by_coords(1, 1).unwrap());
    }
    #[test]
    fn test_glider() {
        let init_state = "
__#____
___#___
_###___
_______
_______
";
        let mut field = Field::from_str(init_state).unwrap();
        assert_eq!(init_state.trim(), field.to_string().trim());
        field.update();
        let expected_state = "
_______
_#_#___
__##___
__#____
_______
";
        assert_eq!(expected_state.trim(), field.to_string().trim());
        field.update();
        let expected_state = "
_______
___#___
_#_#___
__##___
_______
";
        assert_eq!(expected_state.trim(), field.to_string().trim());
        field.update();
        let expected_state = "
_______
__#____
___##__
__##___
_______
";
        assert_eq!(expected_state.trim(), field.to_string().trim());
        field.update();
        let expected_state = "
_______
___#___
____#__
__###__
_______
";
        assert_eq!(expected_state.trim(), field.to_string().trim());
    }
}

