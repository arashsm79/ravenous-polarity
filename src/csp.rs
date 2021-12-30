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
    pub inference_mode: InferenceMode
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    NeighborBased(PoleNumber, PoleNumber),
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
            inference_mode
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
        self.backtrack(&initial_domain, &mut initial_assignment, 100)
    }

    fn backtrack(
        &mut self,
        domains: &Domain,
        assignment: &mut Assignment,
        depth: u32,
    ) -> Option<Assignment> {

        // println!("{:?}", assignment);
        // self.print_board();

        if self.is_complete(&assignment) {
            return Some(assignment.clone());
        }

        if let Some(var_index) = self.select_unassigned_variable(&domains, &assignment) {
            for value in self.order_domain_values(var_index, &domains) {
                if self.assign(value, var_index, assignment) {
                    if self.is_consistent(var_index) {
                        // let (feasible, inferred_domains) =
                        //     self.inference(var_index, &domains, &assignment);
                        let feasible = true;
                        if feasible {
                            if let Some(result) =
                                self.backtrack(domains, assignment, depth - 1)
                            {
                                return Some(result);
                            }
                        }
                    }
                    self.unassign(var_index, assignment);
                }
            }
        }
        None
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
        match constraint_arc.constraint {
            Constraint::NeighborBased(pole_xi, pole_xj) => {
                self.revise_pole_constraint(constraint_arc.xi, constraint_arc.xj, pole_xi, pole_xj, inferred_domains, assignment)
            },
            Constraint::LimitBased(pole_xi, pole_xj) => {
                self.revise_limit_constraint(constraint_arc.xi, constraint_arc.xj, pole_xi, pole_xj, inferred_domains, assignment)
                // (true, false)
            }
        }
    }

    // Revise the domains based on the neighboring cells and their signs. (two positives or two
    // negatives can't be next to each other.
    // returns: (feasible, revised)
    // feasible is false if any domain is reduced to zero
    fn revise_pole_constraint(&self, xi_index: VariableIndex, xj_index: VariableIndex, xi_pole_index: PoleNumber, xj_pole_index: PoleNumber, inferred_domains: &mut Domain, assignment: &Assignment) -> (bool, bool) {
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
                    if let Some(value) = CSP::get_neighbor_pole_based_inconsistent_value(*xi_value, xi_pole_index, xj_pole_index) {
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

    fn revise_limit_constraint(&self, xi_index: VariableIndex, xj_index: VariableIndex, xi_pole_index: PoleNumber, xj_pole_index: PoleNumber, inferred_domains: &mut Domain, assignment: &Assignment) -> (bool, bool) {
        let xi_value = assignment[xi_index];
        let mut revised = false;

        if xi_value == Value::Unassigned {
                // for each value in xi domain
                // if there are no values avalaible in xj's domain that are consistent with the
                // current value of xi, then delete the current value of xi
                let mut to_be_deleted: Vec<Value> = Vec::new();
                let mut constraint_count = 0;
                for xi_value in &inferred_domains[xi_index] {
                    if let Some(value) = self.get_neighbor_limit_based_inconsistent_value(xi_index, xj_index, *xi_value, xi_pole_index, xj_pole_index, assignment) {
                        // println!("inconsistent value: {:?}", value);
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
            let mut board_row_pos_sum = 0;
            let mut board_row_neg_sum = 0;
            for i in 0..self.col_size {
                // dont count the poels of xi and xj
                if self.board_variable_association[xi_pole.row][i] == xi_index || self.board_variable_association[xi_pole.row][i] == xj_index {
                    continue;
                }
                if self.board[xi_pole.row][i] == BoardCell::Positive {
                    board_row_pos_sum += 1;
                } else if self.board[xi_pole.row][i] == BoardCell::Negative {
                    board_row_neg_sum += 1;
                }
            }
            // println!("brps {:?}", board_row_pos_sum);
            // println!("brns {:?}", board_row_neg_sum);

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
            let mut board_col_pos_sum = 0;
            let mut board_col_neg_sum = 0;
            for i in 0..self.row_size {
                // dont count the poels of xi and xj
                if self.board_variable_association[i][xi_pole.col] == xi_index || self.board_variable_association[i][xi_pole.col] == xj_index {
                    continue;
                }
                if self.board[i][xi_pole.col] == BoardCell::Positive {
                    board_col_pos_sum += 1;
                } else if self.board[i][xi_pole.col] == BoardCell::Negative {
                    board_col_neg_sum += 1;
                }
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
    pub fn generate_arc_constraints(
        &self,
        var_index: usize,
        assignment: &Assignment,
        arc_queue: &mut VecDeque<ConstraintArc>,
        except_neighbor: VariableIndex
    )  {
        let variable = &self.variables[var_index];
        // generate arcs for each pole
        for pole in &variable.poles {
            let pole_number = CSP::get_pole_number(variable, pole);

            // Neighbor based constraints
            let neighboring_cells = self.get_neighboring_cells(pole);
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
                    constraint: Constraint::NeighborBased(neighbor_pole_number, pole_number),
                });
            }

            // Limit based constraints
            let neighboring_cells = self.get_limiting_cells(pole);
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
                    constraint: Constraint::LimitBased(neighbor_pole_number, pole_number),
                });
            }
        }
    }

    fn inference(
        &self,
        var_index: usize,
        domains: &Domain,
        assignment: &Assignment,
    ) -> (bool, Domain) {

        return (true, domains.clone());
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

    fn order_domain_values(
        &self,
        var_index: usize,
        domains: &Domain,
    ) -> Vec<Value> {
        return domains[var_index].clone();
        let mut ordered_domain_values: Vec<(Value, i32)> = Vec::new();
        for value in &domains[var_index] {
            let mut constraint_score = 0;
            constraint_score += self
                .calculate_neighbor_based_constraint_score(*value, var_index, domains);
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

    // fn calculate_limits_constraint_score(
    //     &self,
    //     value: Value,
    //     var_index: usize,
    //     domains: &Domain,
    //     assignment: &Assignment,
    // ) -> i32 {
    //     0
    // }

    pub fn get_pole_number(variable: &Variable, cell: &Point) -> u8 {
        if cell.row == variable.poles[0].row && cell.col == variable.poles[0].col {
            0
        } else {
            1
        }
    }

    fn calculate_neighbor_based_constraint_score(
        &self,
        value: Value,
        var_index: usize,
        domains: &Domain,
    ) -> i32 {
        let mut constraint_score = 0;
        let variable = &self.variables[var_index];
        for pole in &variable.poles {
            // returns the cells around the given pole.
            let neighboring_cells = self.get_neighboring_cells(pole);
            let pole_number = CSP::get_pole_number(variable, pole);
            for neighbor_cell in neighboring_cells {
                let neighbor_index =
                    self.board_variable_association[neighbor_cell.row][neighbor_cell.col];
                if neighbor_index == var_index {
                    continue;
                }
                let neighbor = &self.variables[neighbor_index];
                let neighbor_pole_number = CSP::get_pole_number(neighbor, &neighbor_cell);
                let mut increase_constraint_score = false;

                if value == Value::Pole1PositivePole2Negative {
                    if pole_number == 0 && neighbor_pole_number == 0 {
                        if domains[neighbor_index].contains(&Value::Pole1PositivePole2Negative) {
                            increase_constraint_score = true;
                        }
                    } else if pole_number == 0 && neighbor_pole_number == 1 {
                        if domains[neighbor_index].contains(&Value::Pole2PositivePole1Negative) {
                            increase_constraint_score = true;
                        }
                    } else if pole_number == 1 && neighbor_pole_number == 0 {
                        if domains[neighbor_index].contains(&Value::Pole2PositivePole1Negative) {
                            increase_constraint_score = true;
                        }
                    } else if pole_number == 1 && neighbor_pole_number == 1 {
                        if domains[neighbor_index].contains(&Value::Pole1PositivePole2Negative) {
                            increase_constraint_score = true;
                        }
                    }
                } else if value == Value::Pole2PositivePole1Negative {
                    if pole_number == 0 && neighbor_pole_number == 0 {
                        if domains[neighbor_index].contains(&Value::Pole2PositivePole1Negative) {
                            increase_constraint_score = true;
                        }
                    } else if pole_number == 0 && neighbor_pole_number == 1 {
                        if domains[neighbor_index].contains(&Value::Pole1PositivePole2Negative) {
                            increase_constraint_score = true;
                        }
                    } else if pole_number == 1 && neighbor_pole_number == 0 {
                        if domains[neighbor_index].contains(&Value::Pole1PositivePole2Negative) {
                            increase_constraint_score = true;
                        }
                    } else if pole_number == 1 && neighbor_pole_number == 1 {
                        if domains[neighbor_index].contains(&Value::Pole2PositivePole1Negative) {
                            increase_constraint_score = true;
                        }
                    }
                }
                if increase_constraint_score {
                    constraint_score += 1;
                    if domains[neighbor_index].len() == 1 {
                        constraint_score += 5;
                    }
                }
            }
        }
        constraint_score
    }

    // Returns the four neighbors of a cell
    pub fn get_neighboring_cells(&self, cell: &Point) -> Vec<Point> {
        let mut neighboring_cells: Vec<Point> = Vec::new();
        if cell.row + 1 < self.row_size {
            neighboring_cells.push(Point {
                row: cell.row + 1,
                col: cell.col,
            });
        }
        if cell.row as i64 - 1 >= 0 {
            neighboring_cells.push(Point {
                row: cell.row - 1,
                col: cell.col,
            });
        }
        if cell.col + 1 < self.col_size {
            neighboring_cells.push(Point {
                row: cell.row,
                col: cell.col + 1,
            });
        }
        if cell.col as i64 - 1 >= 0 {
            neighboring_cells.push(Point {
                row: cell.row,
                col: cell.col - 1,
            });
        }
        neighboring_cells
    }

    // Returns cells that are on the same row and col as the given cell.
    pub fn get_limiting_cells(&self, cell: &Point) -> Vec<Point> {
        let mut neighboring_cells: Vec<Point> = Vec::new();
        for i in 0..self.row_size {
            if i == cell.row {
                continue;
            }
            neighboring_cells.push(Point { row: i, col: cell.col });
        }

        for j in 0..self.col_size {
            if j == cell.col {
                continue;
            }
            neighboring_cells.push(Point { row: cell.row, col: j });
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

    fn is_consistent(&self, var_index: VariableIndex) -> bool {
        let var = &self.variables[var_index];
        // if this is a horizontal magnet
        if var.poles[0].row == var.poles[1].row {
            let poles_row = var.poles[0].row;
            // check row limits for pos and neg
            let mut count_pos = 0;
            let mut count_neg = 0;
            for j in 0..self.col_size {
                if self.board[poles_row][j] == BoardCell::Positive {
                    count_pos += 1;
                } else if self.board[poles_row][j] == BoardCell::Negative {
                    count_neg += 1;
                }
            }
            if count_pos > self.row_pos_poles[poles_row] || count_neg > self.row_neg_poles[poles_row] {
                return false;
            }

            let poles_col = var.poles[0].col;
            // check pole 1 col limits for pos and neg
            count_pos = 0;
            count_neg = 0;
            for i in 0..self.row_size {
                if self.board[i][poles_col] == BoardCell::Positive {
                    count_pos += 1;
                } else if self.board[i][poles_col] == BoardCell::Negative {
                    count_neg += 1;
                }
            }
            if count_pos > self.col_pos_poles[poles_col] || count_neg > self.col_neg_poles[poles_col] {
                return false;
            }

            let poles_col = var.poles[1].col;
            // check pole 2 col limits for pos and neg
            count_pos = 0;
            count_neg = 0;
            for i in 0..self.row_size {
                if self.board[i][poles_col] == BoardCell::Positive {
                    count_pos += 1;
                } else if self.board[i][poles_col] == BoardCell::Negative {
                    count_neg += 1;
                }
            }
            if count_pos > self.col_pos_poles[poles_col] || count_neg > self.col_neg_poles[poles_col] {
                return false;
            }
        // if this is a vertical magnet
        } else if var.poles[0].col == var.poles[1].col {
            let poles_row = var.poles[0].row;
            // check pole 1 row limits for pos and neg
            let mut count_pos = 0;
            let mut count_neg = 0;
            for j in 0..self.col_size {
                if self.board[poles_row][j] == BoardCell::Positive {
                    count_pos += 1;
                } else if self.board[poles_row][j] == BoardCell::Negative {
                    count_neg += 1;
                }
            }
            if count_pos > self.row_pos_poles[poles_row] || count_neg > self.row_neg_poles[poles_row] {
                return false;
            }

            let poles_row = var.poles[1].row;
            // check pole 2 row limits for pos and neg
            count_pos = 0;
            count_neg = 0;
            for j in 0..self.col_size {
                if self.board[poles_row][j] == BoardCell::Positive {
                    count_pos += 1;
                } else if self.board[poles_row][j] == BoardCell::Negative {
                    count_neg += 1;
                }
            }
            if count_pos > self.row_pos_poles[poles_row] || count_neg > self.row_neg_poles[poles_row] {
                return false;
            }

            let poles_col = var.poles[0].col;
            // check col limits for pos and neg
            count_pos = 0;
            count_neg = 0;
            for i in 0..self.row_size {
                if self.board[i][poles_col] == BoardCell::Positive {
                    count_pos += 1;
                } else if self.board[i][poles_col] == BoardCell::Negative {
                    count_neg += 1;
                }
            }
            if count_pos > self.col_pos_poles[poles_col] || count_neg > self.col_neg_poles[poles_col] {
                return false;
            }
        }
        true
    }
}
