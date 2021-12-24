use std::{error::Error, option::Option, rc::Rc};

struct CSP {
    row_size: usize,
    col_size: usize,
    row_pos_poles: Vec<i32>,
    row_neg_poles: Vec<i32>,
    col_pos_poles: Vec<i32>,
    col_neg_poles: Vec<i32>,
    board: Vec<Vec<BoardCell>>,
    board_variable_association: Vec<Vec<usize>>,
    variables: Vec<Variable>,
}

#[derive(Debug, Clone)]
struct Point {
    row: usize,
    col: usize,
}
// A magnet slot
#[derive(Debug, Clone)]
struct Variable {
    index: usize,
    poles: Vec<Point>,
}
type Assignment = Vec<Value>;
type Domain = Vec<Vec<Value>>;

// A magnet slot can either be empty or be placed in one of the two directions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Value {
    Pole1PositivePole2Negative,
    Pole2PositivePole1Negative,
    Empty,
    Unassigned,
}

// Each single 1x1 cell in the board can have either one of these values.
#[derive(Debug, Clone, PartialEq, Eq)]
enum BoardCell {
    Positive,
    Negative,
    Empty,
}

impl CSP {
    fn new(
        row_size: usize,
        col_size: usize,
        row_pos_poles: Vec<i32>,
        row_neg_poles: Vec<i32>,
        col_pos_poles: Vec<i32>,
        col_neg_poles: Vec<i32>,
        mut raw_board: Vec<Vec<u8>>,
    ) -> CSP {
        let board = vec![vec![BoardCell::Empty; col_size]; row_size];
        let mut board_variable_association = vec![vec![0; col_size]; row_size];
        let mut variables: Vec<Variable> = Vec::new();
        let mut variable_index = 0;
        for i in 0..row_size {
            for j in 0..col_size {
                if raw_board[i][j] == 1 {
                    let down_i = i + 1;
                    if down_i >= row_size {
                        continue;
                    } else {
                        raw_board[i][j] = 2;
                        raw_board[down_i][j] = 2;
                        variables.push(Variable {
                            index: variable_index,
                            poles: vec![
                                Point { row: i, col: j },
                                Point {
                                    row: down_i,
                                    col: j,
                                },
                            ],
                        });
                        board_variable_association[i][j] = variable_index;
                        board_variable_association[down_i][j] = variable_index;
                        variable_index += 1;
                    }
                } else if raw_board[i][j] == 0 {
                    let right_j = j + 1;
                    if right_j >= col_size {
                        continue;
                    } else {
                        raw_board[i][j] = 2;
                        raw_board[i][right_j] = 2;
                        variables.push(Variable {
                            index: variable_index,
                            poles: vec![
                                Point { row: i, col: j },
                                Point {
                                    row: i,
                                    col: right_j,
                                },
                            ],
                        });
                        board_variable_association[i][j] = variable_index;
                        board_variable_association[i][right_j] = variable_index;
                        variable_index += 1;
                    }
                }
            }
        }
        CSP {
            row_size,
            col_size,
            row_pos_poles,
            row_neg_poles,
            col_pos_poles,
            col_neg_poles,
            board,
            board_variable_association,
            variables,
        }
    }

    fn solve(&mut self) -> Option<Assignment> {
        let initial_assignment: Assignment = vec![Value::Unassigned; self.variables.len()];
        let initial_domain: Domain = vec![
            vec![
                Value::Pole1PositivePole2Negative,
                Value::Pole2PositivePole1Negative,
                Value::Empty
            ];
            self.variables.len()
        ];
        self.backtrack(initial_domain, initial_assignment)
    }

    fn backtrack(&mut self, domains: Domain, mut assignment: Assignment) -> Option<Assignment> {
        if self.is_complete(&assignment) {
            return Some(assignment);
        }

        let var_index = self.select_unassigned_variable(&domains, &assignment);
        for value in self.order_domain_values(var_index, &domains, &assignment) {
            if self.assign(value, var_index, &mut assignment) {
                if self.is_consistent(&assignment) {
                    let (feasable, inferred_domains) =
                        self.inference(var_index, &domains, &assignment);
                    if feasable {
                        if let Some(result) = self.backtrack(inferred_domains, assignment.clone()) {
                            return Some(result);
                        }
                    }
                }
                self.unassign(var_index, &mut assignment);
            }
        }

        None
    }

    fn inference(
        &self,
        var_index: usize,
        domains: &Domain,
        assignment: &Assignment,
    ) -> (bool, Domain) {
        (true, vec![])
    }

    fn print_board(&self) {
        print!("{:8}", ' ');
        for i in &self.col_pos_poles {
            print!("{:3} ", i);
        }
        println!();
        print!("{:4}", ' ');
        for i in &self.col_neg_poles {
            print!("{:3} ", i);
        }
        println!();
        for i in 0..self.row_size {
            print!("{:4} ", self.row_pos_poles[i]);
            print!("{:4} ", self.row_neg_poles[i]);

            for cell in &self.board[i] {
                match cell {
                    BoardCell::Positive => {
                        print!(" + ");
                    }
                    BoardCell::Negative => {
                        print!(" - ");
                    }
                    BoardCell::Empty => {
                        print!("   ");
                    }
                }
            }
            println!();
        }
    }

    fn assign(&mut self, value: Value, var_index: usize, assignment: &mut Assignment) -> bool {
        let v = &self.variables[var_index];
        match value {
            Value::Pole1PositivePole2Negative => {
                if self.board[v.poles[0].row][v.poles[0].col] == BoardCell::Empty
                    && self.board[v.poles[1].row][v.poles[1].col] == BoardCell::Empty
                {
                    self.board[v.poles[0].row][v.poles[0].col] = BoardCell::Positive;
                    self.board[v.poles[1].row][v.poles[1].col] = BoardCell::Negative;
                } else {
                    return false;
                }
            }
            Value::Pole2PositivePole1Negative => {
                if self.board[v.poles[0].row][v.poles[0].col] == BoardCell::Empty
                    && self.board[v.poles[1].row][v.poles[1].col] == BoardCell::Empty
                {
                    self.board[v.poles[0].row][v.poles[0].col] = BoardCell::Negative;
                    self.board[v.poles[1].row][v.poles[1].col] = BoardCell::Positive;
                } else {
                    return false;
                }
            }
            Value::Empty => {
                self.board[v.poles[0].row][v.poles[0].col] = BoardCell::Empty;
                self.board[v.poles[1].row][v.poles[1].col] = BoardCell::Empty;
            }
            Value::Unassigned => return false,
        }
        assignment[var_index] = value;
        true
    }

    fn unassign(&mut self, var_index: usize, assignment: &mut Assignment) {
        let v = &self.variables[var_index];
        self.board[v.poles[0].row][v.poles[0].col] = BoardCell::Empty;
        self.board[v.poles[1].row][v.poles[1].col] = BoardCell::Empty;
    }

    // This function uses the MRV heuristic
    fn select_unassigned_variable(&self, domains: &Domain, assignment: &Assignment) -> usize {
        let mut mrv_index = 0;
        let mut mrv_value = std::usize::MAX;
        for i in 0..self.variables.len() {
            if assignment[i] == Value::Unassigned {
                if domains[i].len() < mrv_value {
                    mrv_value = domains[i].len();
                    mrv_index = i;
                }
            }
        }
        mrv_index
    }

    fn order_domain_values(
        &self,
        var_index: usize,
        domains: &Domain,
        assignment: &Assignment,
    ) -> Vec<Value> {
        let mut ordered_domain_values: Vec<(Value, i32)> = Vec::new();
        for value in &domains[var_index] {
            let mut constraint_score = 0;
            constraint_score += self
                .calculate_neighbor_based_constraint_score(*value, var_index, domains, assignment);
            // constraint_score +=
            //     self.calculate_limits_constraint_score(value, var_index, domains, assignment);
            ordered_domain_values.push((*value, constraint_score));
        }
        ordered_domain_values.sort_by(|a, b| a.1.cmp(&b.1));
        ordered_domain_values
            .iter()
            .map(|v| v.0)
            .collect::<Vec<Value>>()
    }

    fn calculate_limits_constraint_score(
        &self,
        value: Value,
        var_index: usize,
        domains: &Domain,
        assignment: &Assignment,
    ) -> i32 {
        0
    }

    fn calculate_neighbor_based_constraint_score(
        &self,
        value: Value,
        var_index: usize,
        domains: &Domain,
        assignment: &Assignment,
    ) -> i32 {
        let mut constraint_score = 0;
        // returns the cells around the given pole. In this case pole1
        let neighboring_cells = self.get_neighboring_cells_around_pole(var_index, 1);
        for neighbor_cell in neighboring_cells {
            let neighbor_index = self.board_variable_association[neighbor_cell.0][neighbor_cell.1];
            let neighbor = &self.variables[neighbor_index];
            match value {
                //pole1 is positive, thus the neighboring cells cant be positive
                Value::Pole1PositivePole2Negative => {
                    // If this neighboring cell is the pole1 of the neighbor variable
                    if neighbor_cell.0 == neighbor.pole1_row
                        && neighbor_cell.1 == neighbor.pole1_col
                    {
                        if domains[neighbor_index].contains(&Value::Pole1PositivePole2Negative) {
                            constraint_score += 1;
                            // if this is the only possible value
                            // increase constraint_score
                            if domains[neighbor_index].len() == 1 {
                                constraint_score += 5;
                            }
                        }
                    // If this neighboring cell is the pole2 of the neighbor variable
                    } else if neighbor_cell.0 == neighbor.pole2_row
                        && neighbor_cell.1 == neighbor.pole2_col
                    {
                        if domains[neighbor_index].contains(&Value::Pole2PositivePole1Negative) {
                            constraint_score += 1;
                            // if this is the only possible value
                            // increase constraint_score
                            if domains[neighbor_index].len() == 1 {
                                constraint_score += 5;
                            }
                        }
                    }
                }
                //pole1 is negative, thus the neighboring cells cant be negative
                Value::Pole2PositivePole1Negative => {
                    // If this neighboring cell is the pole1 of the neighbor variable
                    if neighbor_cell.0 == neighbor.pole1_row
                        && neighbor_cell.1 == neighbor.pole1_col
                    {
                        if domains[neighbor_index].contains(&Value::Pole2PositivePole1Negative) {
                            constraint_score += 1;
                            // if this is the only possible value
                            // increase constraint_score
                            if domains[neighbor_index].len() == 1 {
                                constraint_score += 5;
                            }
                        }
                    // If this neighboring cell is the pole2 of the neighbor variable
                    } else if neighbor_cell.0 == neighbor.pole2_row
                        && neighbor_cell.1 == neighbor.pole2_col
                    {
                        if domains[neighbor_index].contains(&Value::Pole1PositivePole2Negative) {
                            constraint_score += 1;
                            // if this is the only possible value
                            // increase constraint_score
                            if domains[neighbor_index].len() == 1 {
                                constraint_score += 5;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // returns the cells around the given pole. In this case pole2
        let neighboring_cells = self.get_neighboring_cells_around_pole(var_index, 2);
        for neighbor_cell in neighboring_cells {
            let neighbor_index = self.board_variable_association[neighbor_cell.0][neighbor_cell.1];
            let neighbor = &self.variables[neighbor_index];
            match value {
                //pole2 is negative, thus the neighboring cells cant be negative
                Value::Pole1PositivePole2Negative => {
                    // If this neighboring cell is the pole1 of the neighbor variable
                    if neighbor_cell.0 == neighbor.pole1_row
                        && neighbor_cell.1 == neighbor.pole1_col
                    {
                        if domains[neighbor_index].contains(&Value::Pole2PositivePole1Negative) {
                            constraint_score += 1;
                            // if this is the only possible value
                            // increase constraint_score
                            if domains[neighbor_index].len() == 1 {
                                constraint_score += 5;
                            }
                        }
                    // If this neighboring cell is the pole2 of the neighbor variable
                    } else if neighbor_cell.0 == neighbor.pole2_row
                        && neighbor_cell.1 == neighbor.pole2_col
                    {
                        if domains[neighbor_index].contains(&Value::Pole1PositivePole2Negative) {
                            constraint_score += 1;
                            // if this is the only possible value
                            // increase constraint_score
                            if domains[neighbor_index].len() == 1 {
                                constraint_score += 5;
                            }
                        }
                    }
                }
                //pole2 is positive, thus the neighboring cells cant be positive
                Value::Pole2PositivePole1Negative => {
                    // If this neighboring cell is the pole1 of the neighbor variable
                    if neighbor_cell.0 == neighbor.pole1_row
                        && neighbor_cell.1 == neighbor.pole1_col
                    {
                        if domains[neighbor_index].contains(&Value::Pole1PositivePole2Negative) {
                            constraint_score += 1;
                            // if this is the only possible value
                            // increase constraint_score
                            if domains[neighbor_index].len() == 1 {
                                constraint_score += 5;
                            }
                        }
                    // If this neighboring cell is the pole2 of the neighbor variable
                    } else if neighbor_cell.0 == neighbor.pole2_row
                        && neighbor_cell.1 == neighbor.pole2_col
                    {
                        if domains[neighbor_index].contains(&Value::Pole2PositivePole1Negative) {
                            constraint_score += 1;
                            // if this is the only possible value
                            // increase constraint_score
                            if domains[neighbor_index].len() == 1 {
                                constraint_score += 5;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        constraint_score
    }

    fn get_neighboring_cells_around_pole(
        &self,
        var_index: usize,
        pole_number: u8,
    ) -> Vec<(usize, usize)> {
        let mut neighboring_cells: Vec<(usize, usize)> = Vec::new();
        let v = &self.variables[var_index];

        if pole_number == 1 {
            if v.poles[0].row + 1 < self.row_size && v.poles[0].row != v.poles[1].row {
                neighboring_cells.push((v.poles[0].row + 1, v.poles[0].col));
            }
            if v.poles[0].row - 1 >= 0 && v.poles[0].row != v.poles[1].row {
                neighboring_cells.push((v.poles[0].row - 1, v.poles[0].col));
            }
            if v.poles[0].col + 1 < self.col_size && v.poles[0].col != v.poles[1].col {
                neighboring_cells.push((v.poles[0].row, v.poles[0].col + 1));
            }
            if v.poles[0].col - 1 >= 0 && v.poles[0].col != v.poles[1].col {
                neighboring_cells.push((v.poles[0].row, v.poles[0].col - 1));
            }
        } else if pole_number == 2 {
            if v.poles[1].row + 1 < self.row_size && v.poles[0].row != v.poles[1].row {
                neighboring_cells.push((v.poles[1].row + 1, v.poles[1].col));
            }
            if v.poles[1].row - 1 >= 0 && v.poles[0].row != v.poles[1].row {
                neighboring_cells.push((v.poles[1].row - 1, v.poles[1].col));
            }

            if v.poles[1].col + 1 < self.col_size && v.poles[0].col != v.poles[1].col {
                neighboring_cells.push((v.poles[1].row, v.poles[1].col + 1));
            }
            if v.poles[1].col - 1 >= 0 && v.poles[0].col != v.poles[1].col {
                neighboring_cells.push((v.poles[1].row, v.poles[1].col - 1));
            }
        }
        neighboring_cells
    }

    fn is_complete(&self, assignment: &Assignment) -> bool {
        assignment
            .iter()
            .fold(true, |acc, v| acc & (*v != Value::Unassigned))
    }

    fn is_consistent(&self, assignment: &Assignment) -> bool {
        // check rows limits for pos and neg
        for i in 0..self.row_size {
            let mut count_pos = 0;
            let mut count_neg = 0;
            for j in 0..self.col_size {
                if self.board[i][j] == BoardCell::Positive {
                    count_pos += 1;
                } else if self.board[i][j] == BoardCell::Negative {
                    count_neg += 1;
                }
            }
            if count_pos != self.row_pos_poles[i] || count_neg != self.row_neg_poles[i] {
                return false;
            }
        }
        // check column limits for pos and neg
        for j in 0..self.col_size {
            let mut count_pos = 0;
            let mut count_neg = 0;
            for i in 0..self.row_size {
                if self.board[i][j] == BoardCell::Positive {
                    count_pos += 1;
                } else if self.board[i][j] == BoardCell::Negative {
                    count_neg += 1;
                }
            }
            if count_pos != self.col_pos_poles[j] || count_neg != self.col_neg_poles[j] {
                return false;
            }
        }
        true
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let test_case_path = std::env::args()
        .nth(1)
        .expect("Please provide a test case path as command line argument.");

    let mut csp = init_problem(test_case_path).expect("Couldn't parse input");
    if let Some(_) = csp.solve() {
        csp.print_board();
    }
    Ok(())
}

fn init_problem(test_case_path: String) -> Result<CSP, Box<dyn Error>> {
    let test_case_lines: Vec<String> = std::fs::read_to_string(test_case_path)?
        .lines()
        .map(|l| l.parse::<String>().unwrap())
        .collect();

    let board_size: Vec<usize> = test_case_lines
        .get(0)
        .expect("Wrong input format. First line must be the size of the board")
        .split(" ")
        .map(|tok| tok.parse::<usize>().unwrap())
        .collect();
    let row_size = board_size[0];
    let col_size = board_size[1];

    let row_pos_poles: Vec<i32> = test_case_lines
        .get(1)
        .expect("Wrong input format. Second line must be the number of positive poles per row")
        .split(" ")
        .map(|tok| tok.parse::<i32>().unwrap())
        .collect();

    let row_neg_poles: Vec<i32> = test_case_lines
        .get(2)
        .expect("Wrong input format. Third line must be the number of negative poles per row")
        .split(" ")
        .map(|tok| tok.parse::<i32>().unwrap())
        .collect();

    let col_pos_poles: Vec<i32> = test_case_lines
        .get(3)
        .expect("Wrong input format. Forth line must be the number of positive poles per column")
        .split(" ")
        .map(|tok| tok.parse::<i32>().unwrap())
        .collect();

    let col_neg_poles: Vec<i32> = test_case_lines
        .get(4)
        .expect("Wrong input format. Fifth line must be the number of negative poles per column")
        .split(" ")
        .map(|tok| tok.parse::<i32>().unwrap())
        .collect();

    let raw_board: Vec<Vec<u8>> = test_case_lines
        .get(5..(5 + row_size) as usize)
        .expect("Wrong input format. Not enough rows specified")
        .iter()
        .map(|line| {
            line.split(" ")
                .map(|tok| tok.parse::<u8>().unwrap())
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<Vec<u8>>>();
    Ok(CSP::new(
        row_size,
        col_size,
        row_pos_poles,
        row_neg_poles,
        col_pos_poles,
        col_neg_poles,
        raw_board,
    ))
}
