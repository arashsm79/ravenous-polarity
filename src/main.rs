use std::error::Error;

struct CSP {
    row_size: usize,
    col_size: usize,
    row_pos_poles: Vec<i32>,
    row_neg_poles: Vec<i32>,
    col_pos_poles: Vec<i32>,
    col_neg_poles: Vec<i32>,
    board: Vec<Vec<BoardCell>>,
    variables: Vec<Variable>,
}

// A magnet slot
#[derive(Debug, Clone)]
struct Variable {
    pole1_row: usize,
    pole1_col: usize,
    pole2_row: usize,
    pole2_col: usize,
    value: Value,
}

// A magnet slot can either be empty or be placed in one of the two directions
#[derive(Debug, Clone)]
enum Value {
    Pole1PositivePole2Negative,
    Pole2PositivePole1Negative,
    Empty,
}

// Each single 1x1 cell in the board can have either one of these values.
#[derive(Debug, Clone)]
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
        let mut variables: Vec<Variable> = Vec::new();
        for i in 0..row_size {
            for j in 0..col_size {
                if raw_board[i][j] == 1 {
                    let down_i = i + 1;
                    if down_i >= row_size {
                        continue;
                    } else {
                        raw_board[i][j] = 2;
                        raw_board[down_i][j] = 2;
                        variables.push(Variable { pole1_row: i, pole1_col: j, pole2_row: down_i, pole2_col: j, value: Value::Empty });
                    }
                } else if raw_board[i][j] == 0 {
                    let right_j = j + 1;
                    if right_j >= col_size {
                        continue;
                    } else {
                        raw_board[i][j] = 2;
                        raw_board[i][right_j] = 2;
                        variables.push(Variable { pole1_row: i, pole1_col: j, pole2_row: i, pole2_col: right_j, value: Value::Empty });
                    }
                }
            }
        }
        CSP { row_size, col_size, row_pos_poles, row_neg_poles, col_pos_poles, col_neg_poles, board, variables}
    }
    fn backtrack(self) -> () {}
}

fn main() -> Result<(), Box<dyn Error>> {
    let test_case_path = std::env::args()
        .nth(1)
        .expect("Please provide a test case path as command line argument.");

    let csp = init_problem(test_case_path).expect("Couldn't parse input");
    println!("{:#?}", csp.variables);
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
