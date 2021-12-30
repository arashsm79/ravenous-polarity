use std::{collections::{HashSet, VecDeque}, option::Option};

pub struct CSP {
    pub row_size: usize,
    pub col_size: usize,
    pub row_pos_poles: Vec<i32>,
    pub row_neg_poles: Vec<i32>,
    pub col_pos_poles: Vec<i32>,
    pub col_neg_poles: Vec<i32>,
    pub board: Vec<Vec<BoardCell>>,
    pub board_variable_association: Vec<Vec<usize>>,
    pub variables: Vec<Variable>,
    pub inference_mode: InferenceMode,

    curr_row_pos_poles: Vec<i32>,
    curr_row_neg_poles: Vec<i32>,
    curr_col_pos_poles: Vec<i32>,
    curr_col_neg_poles: Vec<i32>,
}

#[derive(Debug, Clone)]
pub struct Point {
    pub row: usize,
    pub col: usize,
}
// A magnet slot
#[derive(Debug, Clone)]
pub struct Variable {
    pub index: usize,
    pub poles: Vec<Point>,
}
pub type Assignment = Vec<Value>;
pub type VariableIndex = usize;
pub type PoleNumber = u8;
pub type Domain = Vec<Vec<Value>>;

// A magnet slot can either be empty or be placed in one of the two directions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Pole1PositivePole2Negative,
    Pole2PositivePole1Negative,
    Empty,
    Unassigned,
}

// Each single 1x1 cell in the board can have either one of these values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardCell {
    Positive,
    Negative,
    Empty,
    Unassigned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    SignBased(PoleNumber, PoleNumber),
    LimitBased(PoleNumber, PoleNumber),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintArc {
    pub xi: VariableIndex,
    pub xj: VariableIndex,
    pub constraint: Constraint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferenceMode {
    FC,
    MAC,
}


impl CSP {
    pub fn new(
        row_size: usize,
        col_size: usize,
        row_pos_poles: Vec<i32>,
        row_neg_poles: Vec<i32>,
        col_pos_poles: Vec<i32>,
        col_neg_poles: Vec<i32>,
        mut raw_board: Vec<Vec<u8>>,
        inference_mode: InferenceMode
    ) -> CSP {
        let board = vec![vec![BoardCell::Unassigned; col_size]; row_size];
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
            curr_row_pos_poles: vec![0; row_pos_poles.len()],
            curr_row_neg_poles: vec![0; row_neg_poles.len()],
            curr_col_pos_poles: vec![0; col_pos_poles.len()],
            curr_col_neg_poles: vec![0; col_neg_poles.len()],
            row_size,
            col_size,
            row_pos_poles,
            row_neg_poles,
            col_pos_poles,
            col_neg_poles,
            board,
            board_variable_association,
            variables,
            inference_mode,
        }
    }

    pub fn solve(&mut self) -> Option<Assignment> {
        let mut initial_assignment: Assignment = vec![Value::Unassigned; self.variables.len()];
        let initial_domain: Domain = vec![
            vec![
                Value::Pole1PositivePole2Negative,
                Value::Pole2PositivePole1Negative,
                Value::Empty
            ];
            self.variables.len()
        ];
        self.backtrack(initial_domain, &mut initial_assignment)
    }

    fn backtrack(
        &mut self,
        domains: Domain,
        assignment: &mut Assignment,
    ) -> Option<Assignment> {

        if self.is_complete(&assignment) {
            return Some(assignment.clone());
        }

        if let Some(var_index) = self.select_unassigned_variable(&domains, &assignment) {
            for value in self.order_domain_values(var_index, &domains, assignment) {
                if self.assign(value, var_index, assignment) {
                    if self.is_consistent(var_index) {
                        let (feasible, inferred_domains) =
                            self.inference(var_index, &domains, &assignment);
                        if feasible {
                            if let Some(result) =
                                self.backtrack(inferred_domains, assignment)
                            {
                                return Some(result);
                            }
                        }
                    }
                    self.unassign(value, var_index, assignment);
                }
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

        let mut arc_queue: VecDeque<ConstraintArc> = VecDeque::new();

        self.generate_arc_constraints(var_index, assignment, &mut arc_queue, var_index);
        if self.inference_mode == InferenceMode::FC {
            self.forward_checking(domains, assignment, arc_queue)
        } else if self.inference_mode == InferenceMode::MAC {
            self.maintaining_arc_consistency(domains, assignment, arc_queue)
        } else {
            (false, domains.clone())
        }
    }

    pub fn remove_value_from_domain(value: Value, domain: &mut Vec<Value>) -> bool {
        if domain.contains(&value) {
            if let Some(pos) = domain.iter().position(|x| *x == value) {
                domain.swap_remove(pos);
                return true;
            }
        }
        false
    }

    // Given the value of xi, this function retuns the value that xj cant be based on the sign of
    // the poles and their possitions
    pub fn get_neighbor_pole_based_inconsistent_value(xi_value: Value, xi_pole_index: PoleNumber, xj_pole_index: PoleNumber) -> Option<Value> {
        match xi_value {
            Value::Pole1PositivePole2Negative => {
                if xi_pole_index == 0 && xj_pole_index == 0 {
                    Some(Value::Pole1PositivePole2Negative)
                } else if xi_pole_index == 0 && xj_pole_index == 1 {
                    Some(Value::Pole2PositivePole1Negative)
                } else if xi_pole_index == 1 && xj_pole_index == 0 {
                    Some(Value::Pole2PositivePole1Negative)
                } else if xi_pole_index == 1 && xj_pole_index == 1 {
                    Some(Value::Pole1PositivePole2Negative)
                } else {
                    None
                }
            },
            Value::Pole2PositivePole1Negative => {
                if xi_pole_index == 0 && xj_pole_index == 0 {
                    Some(Value::Pole2PositivePole1Negative)
                } else if xi_pole_index == 0 && xj_pole_index == 1 {
                    Some(Value::Pole1PositivePole2Negative)
                } else if xi_pole_index == 1 && xj_pole_index == 0 {
                    Some(Value::Pole1PositivePole2Negative)
                } else if xi_pole_index == 1 && xj_pole_index == 1 {
                    Some(Value::Pole2PositivePole1Negative)
                } else {
                    None
                }
            },
            _ => { None }
        }
    }

    pub fn revise(&self, constraint_arc: &ConstraintArc, inferred_domains: &mut Domain, assignment: &Assignment) -> (bool, bool) {
        let (xi_pole_index, xj_pole_index) = match constraint_arc.constraint {
            Constraint::SignBased(xi_pole_index, xj_pole_index) => {
                (xi_pole_index, xj_pole_index)
            },
            Constraint::LimitBased(xi_pole_index, xj_pole_index) => {
                (xi_pole_index, xj_pole_index)
            }
        };

        let xi_index = constraint_arc.xi;
        let xj_index = constraint_arc.xj;

        if xi_index == xj_index {
            return (false, false)
        }
        let xi_value = assignment[xi_index];
        let mut revised = false;

        if xi_value == Value::Unassigned {
                // for each value in xi domain
                // if there are no values avalaible in xj's domain that are consistent with the
                // current value of xi, then delete the current value of xi
                let mut to_be_deleted: Vec<Value> = Vec::new();
                let mut constraint_count = 0;
                for xi_value in &inferred_domains[xi_index] {
                    let value_unwrapped = match constraint_arc.constraint {
                        Constraint::SignBased(_, _) => {
                            CSP::get_neighbor_pole_based_inconsistent_value(*xi_value, xi_pole_index, xj_pole_index)
                        },
                        Constraint::LimitBased(_, _) => {
                            self.get_neighbor_limit_based_inconsistent_value(xi_index, xj_index, *xi_value, xi_pole_index, xj_pole_index, assignment)
                        }
                    };
                    if let Some(value) = value_unwrapped{
                        if assignment[xj_index] != Value::Unassigned && assignment[xj_index] == value {
                                to_be_deleted.push(*xi_value);
                        } else if inferred_domains[xj_index].contains(&value) {
                            constraint_count += 1;
                        }
                    }
                    if constraint_count == inferred_domains[xj_index].len() {
                        to_be_deleted.push(*xi_value);
                    }
                }
                revised = !to_be_deleted.is_empty();
                for value in to_be_deleted {
                    CSP::remove_value_from_domain(value, &mut inferred_domains[xi_index]);
                }
        }

        if inferred_domains[xi_index].len() == 0 {
            return (false, false)
        }
        (true, revised)
    }

    // Given the value of xi, this function retuns the value that xj cant be based on the limits of
    // positive and negatives signs in each row
    pub fn get_neighbor_limit_based_inconsistent_value(&self, xi_index: VariableIndex, xj_index: VariableIndex, xi_value: Value, xi_pole_index: PoleNumber, xj_pole_index: PoleNumber, assignment: &Assignment) -> Option<Value> {

        // self.print_board();
        // println!("{:?}", assignment);
        // println!("xi {} xj {}", xi_index, xj_index);
        // println!("xi {:?} xj {:?}", self.variables[xi_index], self.variables[xj_index]);
        // println!("xi_pole_i {:?} xj_pole_i {:?}", xi_pole_index, xj_pole_index);
        // println!(" xi vlaue {:?}", xi_value);

        let xi = &self.variables[xi_index];
        let xj = &self.variables[xj_index];

        let xi_pole = &xi.poles[xi_pole_index as usize];
        let xj_pole = &xj.poles[xj_pole_index as usize];
        // println!("xi_pole {:?} xj_pole {:?}", xi_pole, xj_pole);

        // if the constrained poles of xi and xj are on the same row:
        if xi_pole.row == xj_pole.row {
            let mut board_row_pos_sum = self.curr_row_pos_poles[xi_pole.row];
            let mut board_row_neg_sum = self.curr_row_neg_poles[xi_pole.row];
            // dont count the poels of xi and xj
            if self.board[xi_pole.row][xi_pole.col] == BoardCell::Positive {
                board_row_pos_sum -= 1;
            }
            if self.board[xj_pole.row][xj_pole.col] == BoardCell::Positive {
                board_row_pos_sum -= 1;
            }
            if self.board[xi_pole.row][xi_pole.col] == BoardCell::Negative {
                board_row_neg_sum -= 1;
            }
            if self.board[xj_pole.row][xj_pole.col] == BoardCell::Negative {
                board_row_neg_sum -= 1;
            }
            // println!("brps {:?}", board_row_pos_sum);
            // println!("brns {:?}", board_row_neg_sum);

            // the  curr row sum (without the considering the values of poles of xi and xj) is one
            // less than the limit. Thus if the pole of si is positive then the pole of xj cant be
            // positive
            if board_row_pos_sum == self.row_pos_poles[xi_pole.row] - 1 {
                match xi_value {
                    Value::Pole1PositivePole2Negative => {
                        if xi_pole_index == 0 && xj_pole_index == 0 {
                            Some(Value::Pole1PositivePole2Negative)
                        } else if xi_pole_index == 0 && xj_pole_index == 1 {
                            Some(Value::Pole2PositivePole1Negative)
                        } else { None }
                    },
                    Value::Pole2PositivePole1Negative => {
                        if xi_pole_index == 1 && xj_pole_index == 0 {
                            Some(Value::Pole1PositivePole2Negative)
                        } else if xi_pole_index == 1 && xj_pole_index == 1 {
                            Some(Value::Pole2PositivePole1Negative)
                        } else { None }
                    },
                    _ => { None }
                }
            } else if board_row_neg_sum == self.row_neg_poles[xi_pole.row] - 1 {
                match xi_value {
                    Value::Pole1PositivePole2Negative => {
                        if xi_pole_index == 1 && xj_pole_index == 0 {
                            Some(Value::Pole2PositivePole1Negative)
                        } else if xi_pole_index == 1 && xj_pole_index == 1 {
                            Some(Value::Pole1PositivePole2Negative)
                        } else { None }
                    },
                    Value::Pole2PositivePole1Negative => {
                        if xi_pole_index == 0 && xj_pole_index == 0 {
                            Some(Value::Pole2PositivePole1Negative)
                        } else if xi_pole_index == 0 && xj_pole_index == 1 {
                            Some(Value::Pole1PositivePole2Negative)
                        } else { None }
                    },
                    _ => { None }
                }
                // xj cant be empty if it is the last unassigned variable in a row and the row
                // constraint has not been met
            }else if board_row_pos_sum == self.row_pos_poles[xi_pole.row] - 2 {
                let mut unassigned_vars_in_row: HashSet<VariableIndex> = HashSet::new();
                for i in 0..self.col_size {
                    let curr_var_index = self.board_variable_association[xi_pole.row][i];
                    if curr_var_index != xi_index && curr_var_index != xj_index && assignment[curr_var_index] == Value::Unassigned {
                        unassigned_vars_in_row.insert(curr_var_index);
                    }
                }
                if unassigned_vars_in_row.len() == 0 {
                    match xi_value {
                        Value::Pole1PositivePole2Negative => {
                            if xi_pole_index == 0  {
                                Some(Value::Empty)
                            } else if xi_pole_index == 0 {
                                Some(Value::Empty)
                            } else { None }
                        },
                        Value::Pole2PositivePole1Negative => {
                            if xi_pole_index == 1 {
                                Some(Value::Empty)
                            } else if xi_pole_index == 1 {
                                Some(Value::Empty)
                            } else { None }
                        },
                        _ => { None }
                    }
                } else { None }

            } else if board_row_neg_sum == self.row_neg_poles[xi_pole.row] - 2 {
                let mut unassigned_vars_in_row: HashSet<VariableIndex> = HashSet::new();
                for i in 0..self.col_size {
                    let curr_var_index = self.board_variable_association[xi_pole.row][i];
                    if curr_var_index != xi_index && curr_var_index != xj_index && assignment[curr_var_index] == Value::Unassigned {
                        unassigned_vars_in_row.insert(curr_var_index);
                    }
                }
                if unassigned_vars_in_row.len() == 0 {
                    match xi_value {
                        Value::Pole1PositivePole2Negative => {
                            if xi_pole_index == 1  {
                                Some(Value::Empty)
                            } else if xi_pole_index == 1 {
                                Some(Value::Empty)
                            } else { None }
                        },
                        Value::Pole2PositivePole1Negative => {
                            if xi_pole_index == 0 {
                                Some(Value::Empty)
                            } else if xi_pole_index == 0 {
                                Some(Value::Empty)
                            } else { None }
                        },
                        _ => { None }
                    }
                } else { None }
            } else { None }

        // if the constrained poles of xi and xj are on the same col:
        } else if xi_pole.col == xj_pole.col {
            let mut board_col_pos_sum = self.curr_col_pos_poles[xi_pole.col];
            let mut board_col_neg_sum = self.curr_col_neg_poles[xi_pole.col];
            // dont count the poles of xi and xj
            if self.board[xi_pole.row][xi_pole.col] == BoardCell::Positive {
                board_col_pos_sum -= 1;
            }
            if self.board[xj_pole.row][xj_pole.col] == BoardCell::Positive {
                board_col_pos_sum -= 1;
            }
            if self.board[xi_pole.row][xi_pole.col] == BoardCell::Negative {
                board_col_neg_sum -= 1;
            }
            if self.board[xj_pole.row][xj_pole.col] == BoardCell::Negative {
                board_col_neg_sum -= 1;
            }
            // println!("bcps {:?}", board_col_pos_sum);
            // println!("bcns {:?}", board_col_neg_sum);

            if board_col_pos_sum == self.col_pos_poles[xi_pole.col] - 1 {
                match xi_value {
                    Value::Pole1PositivePole2Negative => {
                        if xi_pole_index == 0 && xj_pole_index == 0 {
                            Some(Value::Pole1PositivePole2Negative)
                        } else if xi_pole_index == 0 && xj_pole_index == 1 {
                            Some(Value::Pole2PositivePole1Negative)
                        } else { None }
                    },
                    Value::Pole2PositivePole1Negative => {
                        if xi_pole_index == 1 && xj_pole_index == 0 {
                            Some(Value::Pole1PositivePole2Negative)
                        } else if xi_pole_index == 1 && xj_pole_index == 1 {
                            Some(Value::Pole2PositivePole1Negative)
                        } else { None }
                    },
                    _ => { None }
                }
            } else if board_col_neg_sum == self.col_neg_poles[xi_pole.col] - 1 {
                match xi_value {
                    Value::Pole1PositivePole2Negative => {
                        if xi_pole_index == 1 && xj_pole_index == 0 {
                            Some(Value::Pole2PositivePole1Negative)
                        } else if xi_pole_index == 1 && xj_pole_index == 1 {
                            Some(Value::Pole1PositivePole2Negative)
                        } else { None }
                    },
                    Value::Pole2PositivePole1Negative => {
                        if xi_pole_index == 0 && xj_pole_index == 0 {
                            Some(Value::Pole2PositivePole1Negative)
                        } else if xi_pole_index == 0 && xj_pole_index == 1 {
                            Some(Value::Pole1PositivePole2Negative)
                        } else { None }
                    },
                    _ => { None }
                }
            } else if board_col_pos_sum == self.col_pos_poles[xi_pole.col] - 2 {
                // xj cant be empty if it is the last unassigned variable in a col and the col
                // constraint has not been met
                let mut unassigned_vars_in_col: HashSet<VariableIndex> = HashSet::new();
                for i in 0..self.row_size {
                    let curr_var_index = self.board_variable_association[i][xi_pole.col];
                    if curr_var_index != xi_index && curr_var_index != xj_index && assignment[curr_var_index] == Value::Unassigned {
                        unassigned_vars_in_col.insert(curr_var_index);
                    }
                }
                if unassigned_vars_in_col.len() == 0 {
                    match xi_value {
                        Value::Pole1PositivePole2Negative => {
                            if xi_pole_index == 0  {
                                Some(Value::Empty)
                            } else if xi_pole_index == 0 {
                                Some(Value::Empty)
                            } else { None }
                        },
                        Value::Pole2PositivePole1Negative => {
                            if xi_pole_index == 1 {
                                Some(Value::Empty)
                            } else if xi_pole_index == 1 {
                                Some(Value::Empty)
                            } else { None }
                        },
                        _ => { None }
                    }
                } else { None }
            } else if board_col_neg_sum == self.col_neg_poles[xi_pole.col] - 2 {
                let mut unassigned_vars_in_col: HashSet<VariableIndex> = HashSet::new();
                for i in 0..self.row_size {
                    let curr_var_index = self.board_variable_association[i][xi_pole.col];
                    if curr_var_index != xi_index && curr_var_index != xj_index && assignment[curr_var_index] == Value::Unassigned {
                        unassigned_vars_in_col.insert(curr_var_index);
                    }
                }
                if unassigned_vars_in_col.len() == 0 {
                    match xi_value {
                        Value::Pole1PositivePole2Negative => {
                            if xi_pole_index == 1  {
                                Some(Value::Empty)
                            } else if xi_pole_index == 1 {
                                Some(Value::Empty)
                            } else { None }
                        },
                        Value::Pole2PositivePole1Negative => {
                            if xi_pole_index == 0 {
                                Some(Value::Empty)
                            } else if xi_pole_index == 0 {
                                Some(Value::Empty)
                            } else { None }
                        },
                        _ => { None }
                    }
                } else { None }
            } else { None }
        } else { None }
    }

    // Generates all the constraints of the given value with respect to its neighbors
    // returns a list of binary arc constrains except for the given neighbor
    // generating arcs for xi results in all arcs (xj, xi) where xj is a neighbor of xi
    pub fn generate_arc_constraints(
        &self,
        var_index: usize,
        assignment: &Assignment,
        arc_queue: &mut VecDeque<ConstraintArc>,
        except_neighbor: VariableIndex
    )  {
        let variable = &self.variables[var_index];
        // generate arcs for each pole
        for pole_number in 0..2 {
            let pole = &variable.poles[pole_number];
            let other_pole = &variable.poles[(pole_number + 1) % 2];

            let neighboring_cells = self.get_neighboring_cells(pole, other_pole);
            for neighbor_cell in neighboring_cells {
                let neighbor_index =
                    self.board_variable_association[neighbor_cell.row][neighbor_cell.col];
                if neighbor_index == var_index || assignment[neighbor_index] != Value::Unassigned || neighbor_index == except_neighbor {
                    continue;
                }
                let neighbor = &self.variables[neighbor_index];
                let neighbor_pole_number = CSP::get_pole_number(neighbor, &neighbor_cell);
                arc_queue.push_back(ConstraintArc {
                    xi: neighbor_index,
                    xj: var_index,
                    constraint: Constraint::SignBased(neighbor_pole_number, pole_number as u8),
                });
            }

            // Limit based constraints
            let neighboring_cells = self.get_limiting_cells(pole, other_pole);
            for neighbor_cell in neighboring_cells {
                let neighbor_index =
                    self.board_variable_association[neighbor_cell.row][neighbor_cell.col];
                if neighbor_index == var_index || assignment[neighbor_index] != Value::Unassigned  || neighbor_index == except_neighbor {
                    continue;
                }
                let neighbor = &self.variables[neighbor_index];
                let neighbor_pole_number = CSP::get_pole_number(neighbor, &neighbor_cell);
                arc_queue.push_back(ConstraintArc {
                    xi: neighbor_index,
                    xj: var_index,
                    constraint: Constraint::LimitBased(neighbor_pole_number, pole_number as u8),
                });
            }
        }
    }

    pub fn print_board(&self) {
        print!("{:8}", ' ');
        for i in &self.col_pos_poles {
            print!("{:4}", i);
        }
        println!();
        print!("{:8}", ' ');
        for i in &self.col_neg_poles {
            print!("{:4}", i);
        }
        println!();
        for i in 0..self.row_size {
            print!("{:4}", self.row_pos_poles[i]);
            print!("{:4}", self.row_neg_poles[i]);

            for cell in &self.board[i] {
                match cell {
                    BoardCell::Positive => {
                        print!("   {}", '+');
                    }
                    BoardCell::Negative => {
                        print!("   {}", '-');
                    }
                    BoardCell::Empty => {
                        print!("   {}", ' ');
                    }
                    BoardCell::Unassigned => {
                        print!("   {}", '*');
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
                if self.board[v.poles[0].row][v.poles[0].col] == BoardCell::Unassigned
                    && self.board[v.poles[1].row][v.poles[1].col] == BoardCell::Unassigned
                {
                    self.board[v.poles[0].row][v.poles[0].col] = BoardCell::Positive;
                    self.board[v.poles[1].row][v.poles[1].col] = BoardCell::Negative;
                    self.curr_row_pos_poles[v.poles[0].row] += 1;
                    self.curr_col_pos_poles[v.poles[0].col] += 1;
                    self.curr_row_neg_poles[v.poles[1].row] += 1;
                    self.curr_col_neg_poles[v.poles[1].col] += 1;
                } else {
                    return false;
                }
            }
            Value::Pole2PositivePole1Negative => {
                if self.board[v.poles[0].row][v.poles[0].col] == BoardCell::Unassigned
                    && self.board[v.poles[1].row][v.poles[1].col] == BoardCell::Unassigned
                {
                    self.board[v.poles[0].row][v.poles[0].col] = BoardCell::Negative;
                    self.board[v.poles[1].row][v.poles[1].col] = BoardCell::Positive;
                    self.curr_row_neg_poles[v.poles[0].row] += 1;
                    self.curr_col_neg_poles[v.poles[0].col] += 1;
                    self.curr_row_pos_poles[v.poles[1].row] += 1;
                    self.curr_col_pos_poles[v.poles[1].col] += 1;
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

    fn unassign(&mut self, value: Value, var_index: usize, assignment: &mut Assignment) {
        let v = &self.variables[var_index];
        self.board[v.poles[0].row][v.poles[0].col] = BoardCell::Unassigned;
        self.board[v.poles[1].row][v.poles[1].col] = BoardCell::Unassigned;
        match value {
            Value::Pole1PositivePole2Negative => {
                    self.curr_row_pos_poles[v.poles[0].row] -= 1;
                    self.curr_col_pos_poles[v.poles[0].col] -= 1;
                    self.curr_row_neg_poles[v.poles[1].row] -= 1;
                    self.curr_col_neg_poles[v.poles[1].col] -= 1;
            }
            Value::Pole2PositivePole1Negative => {
                    self.curr_row_neg_poles[v.poles[0].row] -= 1;
                    self.curr_col_neg_poles[v.poles[0].col] -= 1;
                    self.curr_row_pos_poles[v.poles[1].row] -= 1;
                    self.curr_col_pos_poles[v.poles[1].col] -= 1;
            }
            _ => {},
        }
        assignment[var_index] = Value::Unassigned;
    }

    // This function uses the MRV heuristic
    fn select_unassigned_variable(
        &self,
        domains: &Domain,
        assignment: &Assignment,
    ) -> Option<usize> {
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

        if assignment[mrv_index] == Value::Unassigned {
            Some(mrv_index)
        } else {
            None
        }
    }

    // LCV 
    fn order_domain_values(
        &self,
        var_index: usize,
        domains: &Domain,
        assignment: &Assignment
    ) -> Vec<Value> {
        // Turn of LCV
        // return domains[var_index].clone();
        let mut ordered_domain_values: Vec<(Value, i32)> = Vec::new();
        for value in &domains[var_index] {
            let mut constraint_score = 0;
            constraint_score += self.calculate_constraint_score(*value, var_index, domains, assignment);
            ordered_domain_values.push((*value, constraint_score));
        }
        ordered_domain_values.sort_by(|a, b| a.1.cmp(&b.1));
        ordered_domain_values
            .iter()
            .map(|v| v.0)
            .collect::<Vec<Value>>()
    }


    pub fn get_pole_number(variable: &Variable, cell: &Point) -> u8 {
        if cell.row == variable.poles[0].row && cell.col == variable.poles[0].col {
            0
        } else {
            1
        }
    }

    // Returns the constraint score, given the value of variable
    fn calculate_constraint_score(
        &self,
        value: Value,
        var_index: usize,
        domains: &Domain,
        assignment: &Assignment
    ) -> i32 {
        let mut arc_queue: VecDeque<ConstraintArc> = VecDeque::new();
        let mut constraint_score = 0;
        self.generate_arc_constraints(var_index, assignment, &mut arc_queue, var_index);
        while !arc_queue.is_empty() {
            if let Some(constraint_arc) = arc_queue.pop_front() {
                let (xi_pole_index, xj_pole_index) = match constraint_arc.constraint {
                    Constraint::SignBased(xi_pole_index, xj_pole_index) => {
                        (xi_pole_index, xj_pole_index)
                    },
                    Constraint::LimitBased(xi_pole_index, xj_pole_index) => {
                        (xi_pole_index, xj_pole_index)
                    }
                };

                let xi_index = constraint_arc.xi;
                let xj_index = constraint_arc.xj;

                if xi_index == xj_index {
                    continue;
                }
                let xi_value = assignment[xi_index];
                let xj_value = value;

                if xi_value == Value::Unassigned {
                    // for each value in xi domain
                    // if the values of xj is inconsistent with the
                    // current value of xi, then increase constraint score
                    let mut to_be_deleted: Vec<Value> = Vec::new();
                    for xi_value in &domains[xi_index] {
                        let value_unwrapped = match constraint_arc.constraint {
                            Constraint::SignBased(_, _) => {
                                CSP::get_neighbor_pole_based_inconsistent_value(*xi_value, xi_pole_index, xj_pole_index)
                            },
                            Constraint::LimitBased(_, _) => {
                                self.get_neighbor_limit_based_inconsistent_value(xi_index, xj_index, *xi_value, xi_pole_index, xj_pole_index, assignment)
                            }
                        };
                        if let Some(value) = value_unwrapped{
                            if xj_value == value {
                                to_be_deleted.push(*xi_value);
                            }
                        }
                    }
                    for _ in &to_be_deleted {
                        constraint_score += 1;
                    }
                    // If this value results in a 0 domain, then increase the constraint more
                    if to_be_deleted.len() != 0 && domains[xi_index].len() == 1 {
                        constraint_score += 5;
                    }
                }
            }
        }
        constraint_score
    }

    // Returns the four neighbors of a cell
    pub fn get_neighboring_cells(&self, cell: &Point, same_variable_cell: &Point) -> Vec<Point> {
        let mut neighboring_cells: Vec<Point> = Vec::new();
        if cell.row + 1 < self.row_size
        && cell.row + 1 != same_variable_cell.row
        && cell.col != same_variable_cell.col {
            neighboring_cells.push(Point {
                row: cell.row + 1,
                col: cell.col,
            });
        }
        if cell.row as i64 - 1 >= 0
        && cell.row as i64 - 1 != same_variable_cell.row as i64
        && cell.col != same_variable_cell.col {
            neighboring_cells.push(Point {
                row: cell.row - 1,
                col: cell.col,
            });
        }
        if cell.col + 1 < self.col_size
        && cell.row != same_variable_cell.row
        && cell.col + 1 != same_variable_cell.col {
            neighboring_cells.push(Point {
                row: cell.row,
                col: cell.col + 1,
            });
        }
        if cell.col as i64 - 1 >= 0
        && cell.row != same_variable_cell.row
        && cell.col as i64 - 1 != same_variable_cell.col as i64 {
            neighboring_cells.push(Point {
                row: cell.row,
                col: cell.col - 1,
            });
        }
        neighboring_cells
    }

    // Returns cells that are on the same row and col as the given cell.
    pub fn get_limiting_cells(&self, cell: &Point, same_variable_cell: &Point) -> Vec<Point> {
        let mut neighboring_cells: Vec<Point> = Vec::new();
        for i in 0..self.row_size {
            if i == cell.row {
                continue;
            }
            if i != same_variable_cell.row
            && cell.col != same_variable_cell.col {
                neighboring_cells.push(Point { row: i, col: cell.col });
            }
        }

        for j in 0..self.col_size {
            if j == cell.col {
                continue;
            }
            if cell.row != same_variable_cell.row
            && j != same_variable_cell.col {
                neighboring_cells.push(Point { row: cell.row, col: j });
            }
        }
        neighboring_cells
    }

    fn is_complete(&self, assignment: &Assignment) -> bool {
        if !assignment
            .iter()
            .fold(true, |acc, v| acc & (*v != Value::Unassigned))
        {
            return false;
        }

        true
    }

    fn check_neighbors_pole_sign_constraint(&self, cell: &Point) -> bool {
        let value = &self.board[cell.row][cell.col];
        match value {
            BoardCell::Positive => {
                if cell.row + 1 < self.row_size {
                    if self.board[cell.row + 1][cell.col] == BoardCell::Positive {
                        return false;
                    }
                }
                if cell.row as i64 - 1 >= 0 {
                    if self.board[cell.row - 1][cell.col] == BoardCell::Positive {
                        return false;
                    }
                }
                if cell.col + 1 < self.col_size {
                    if self.board[cell.row][cell.col + 1] == BoardCell::Positive {
                        return false;
                    }
                }
                if cell.col as i64 - 1 >= 0 {
                    if self.board[cell.row][cell.col - 1] == BoardCell::Positive {
                        return false;
                    }
                }
            },
            BoardCell::Negative => {
                if cell.row + 1 < self.row_size {
                    if self.board[cell.row + 1][cell.col] == BoardCell::Negative {
                        return false;
                    }
                }
                if cell.row as i64 - 1 >= 0 {
                    if self.board[cell.row - 1][cell.col] == BoardCell::Negative {
                        return false;
                    }
                }
                if cell.col + 1 < self.col_size {
                    if self.board[cell.row][cell.col + 1] == BoardCell::Negative {
                        return false;
                    }
                }
                if cell.col as i64 - 1 >= 0 {
                    if self.board[cell.row][cell.col - 1] == BoardCell::Negative {
                        return false;
                    }
                }
            },
            _ => {}
        }
        true
    }

    fn is_consistent(&self, var_index: VariableIndex) -> bool {
        let var = &self.variables[var_index];
        // pole sign based cinssitency
        for pole in &var.poles {
            if !self.check_neighbors_pole_sign_constraint(pole) {
                return false
            }
        }

        // limit based consistency
        // if this is a horizontal magnet
        if var.poles[0].row == var.poles[1].row {
            let poles_row = var.poles[0].row;

            // if all the cells in this row are assigned
            // then the curr limit of this row has to be equal to the total limit of this row
            let mut poles_row_all_assigned = true;
            for j in 0..self.col_size {
                poles_row_all_assigned &= self.board[poles_row][j] != BoardCell::Unassigned;
            }
            if poles_row_all_assigned {
                if self.curr_row_pos_poles[poles_row] != self.row_pos_poles[poles_row]
                || self.curr_row_neg_poles[poles_row] != self.row_neg_poles[poles_row] {
                    return false
                }
            }

            // if there are some unassigned cells left then the curr limit has to be lower than the
            // total limit for that row
            if self.curr_row_pos_poles[poles_row] > self.row_pos_poles[poles_row]
                || self.curr_row_neg_poles[poles_row] > self.row_neg_poles[poles_row] {
                return false
            }

            let pole1_col = var.poles[0].col;
            let pole2_col = var.poles[1].col;

            let mut pole1_col_all_assigned = true;
            for i in 0..self.row_size {
                pole1_col_all_assigned &= self.board[i][pole1_col] != BoardCell::Unassigned;
            }
            if pole1_col_all_assigned {
                if self.curr_col_pos_poles[pole1_col] != self.col_pos_poles[pole1_col]
                || self.curr_col_neg_poles[pole1_col] != self.col_neg_poles[pole1_col] {
                    return false
                }
            }
            let mut pole2_col_all_assigned = true;
            for i in 0..self.row_size {
                pole2_col_all_assigned &= self.board[i][pole2_col] != BoardCell::Unassigned;
            }
            if pole2_col_all_assigned {
                if self.curr_col_pos_poles[pole2_col] != self.col_pos_poles[pole2_col]
                || self.curr_col_neg_poles[pole2_col] != self.col_neg_poles[pole2_col] {
                    return false
                }
            }

            if self.curr_col_pos_poles[pole1_col] > self.col_pos_poles[pole1_col]
                || self.curr_col_neg_poles[pole1_col] > self.col_neg_poles[pole1_col] {
                return false
            }
            if self.curr_col_pos_poles[pole2_col] > self.col_pos_poles[pole2_col]
                || self.curr_col_neg_poles[pole2_col] > self.col_neg_poles[pole2_col] {
                return false
            }
        // if this is a vertical magnet
        } else if var.poles[0].col == var.poles[1].col {
            let pole1_row = var.poles[0].row;
            let pole2_row = var.poles[1].row;
            let mut pole1_row_all_assigned = true;
            for j in 0..self.col_size {
                pole1_row_all_assigned &= self.board[pole1_row][j] != BoardCell::Unassigned;
            }
            if pole1_row_all_assigned {
                if self.curr_row_pos_poles[pole1_row] != self.row_pos_poles[pole1_row]
                || self.curr_row_neg_poles[pole1_row] != self.row_neg_poles[pole1_row] {
                    return false
                }
            }
            let mut pole2_row_all_assigned = true;
            for j in 0..self.col_size {
                pole2_row_all_assigned &= self.board[pole2_row][j] != BoardCell::Unassigned;
            }
            if pole2_row_all_assigned {
                if self.curr_row_pos_poles[pole2_row] != self.row_pos_poles[pole2_row]
                || self.curr_row_neg_poles[pole2_row] != self.row_neg_poles[pole2_row] {
                    return false
                }
            }
            if self.curr_row_pos_poles[pole1_row] > self.row_pos_poles[pole1_row]
                || self.curr_row_neg_poles[pole1_row] > self.row_neg_poles[pole1_row] {
                return false
            }
            if self.curr_row_pos_poles[pole2_row] > self.row_pos_poles[pole2_row]
                || self.curr_row_neg_poles[pole2_row] > self.row_neg_poles[pole2_row] {
                return false
            }
            let poles_col = var.poles[0].col;
            let mut poles_col_all_assigned = false;
            for i in 0..self.row_size {
                poles_col_all_assigned &= self.board[i][poles_col] != BoardCell::Unassigned;
            }
            if poles_col_all_assigned {
                if self.curr_col_pos_poles[poles_col] != self.col_pos_poles[poles_col]
                || self.curr_col_neg_poles[poles_col] != self.col_neg_poles[poles_col] {
                    return false
                }
            }
            if self.curr_col_pos_poles[poles_col] > self.col_pos_poles[poles_col]
                || self.curr_col_neg_poles[poles_col] > self.col_neg_poles[poles_col] {
                return false
            }
        }
        true
    }
}
