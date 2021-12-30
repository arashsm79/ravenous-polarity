# Ravenous Polarity
Solving the Magnet Puzzle using constraint satisfaction problem heuristics such as MRV and LCV and backtracking.
## The Model
The main object in the program is the CSP struct. After parsing the input, the fields of this struct are filled accordingly. The board field is the grid of all the cells in the game. The `board_variable_association` is a mapping from the cells of the board to the variables (magnets) and finally the `variables` array is the list of variables (magnets) present in the game.
``` rust
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
```
We have two more important objects: 
* `Variable`
``` rust
pub struct Variable {
    pub index: usize,
    pub poles: Vec<Point>,
}
pub enum Value {
    Pole1PositivePole2Negative,
    Pole2PositivePole1Negative,
    Empty,
    Unassigned,
}
pub enum BoardCell {
    Positive,
    Negative,
    Empty,
    Unassigned,
}
```
The Variable struct corresponds to a Magnet in the game. It has an index which is the position and the unique ID of this variable in the `assignment` and `domains` and `variables` arrays. Each magnet (variable) has two poles that are specified in the poles field.
* `ConstraintArc`
``` rust
pub struct ConstraintArc {
    pub xi: VariableIndex,
    pub xj: VariableIndex,
    pub constraint: Constraint,
}
pub enum Constraint {
    SignBased(PoleNumber, PoleNumber),
        LimitBased(PoleNumber, PoleNumber),
}
```
This represents a binary constraint between two variables. The constraint fields specifies the kind of constraint that can be either a `SignBased` constraint (the given poles of these to magnets can't have the same sign) or `LimitBased` constraint (constraints based on the limits of each row and column between the poles of these two variables). Since each magnet (variable) hast two poles it is specified in the constraint field that which of these two poles are present in the constraint.

# Backtracking
The solve functions creates the initial `assignment` and `domains` array and calls the `backtrack` function.
The `assignment` array holds the value given to a variable (it is indexed by the variable ID) and the `domains` array is a 2d array that contains the domain of each variable.
``` rust
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

    fn backtrack( &mut self, domains: Domain, assignment: &mut Assignment) -> Option<Assignment> {

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
                            if let Some(result) = self.backtrack(inferred_domains, assignment) {
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
```
* The `is_complete` function returns the current assignment if all of the variables have been assigned a proper value. 
* Minimum Remaining Values (MRV) is used to select the next unassigned variable. It select the variable that has the least number of value available in its domain.
* Least Constraining Value (LCV) is used to assign a penalty to each value of the chosen variable. The penalty is chosen based on the amount of constraint that the given value imposes on its neighbors. The value if the least penalty is chosen first.
* The is consistent functions checks the following:
    * The assigned value should not make the poles of the variable have the same sign as the poles of its neighbors
    * If all of the row and column cells of each of the poles of the variable are assigned, then the number of positive and negative poles in that row and column must be equal to the specified limit in the problem
    * If all of the row and column cells of each of the poles of the variable are not assigned, then the number of positive and negative poles in that row and column must less than to the specified limit in the problem. If on of the these conditions does not hold then it will not accept the value.
* `inference` uses either forward checking or maintaining arc consistency (which uses ac3) to reduce the domains of the variables. For the given value xi it first generates all the binary constraints (xj, xi) where xj is an unassigned neighbor of xi and then passes this list to the appropriate functions.
``` rust
    fn inference( &self, var_index: usize, domains: &Domain, assignment: &Assignment) -> (bool, Domain) {

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
```
* `forward checking` uses `revise` to reduce the domain of xi in each (xi, xj) constraints.
``` rust
    pub fn forward_checking( &self, domains: &Domain, assignment: &Assignment,
                             mut arc_queue: VecDeque<ConstraintArc>) -> (bool, Domain) {
        let mut inferred_domains = domains.clone();
        while !arc_queue.is_empty() {
            if let Some(constraint_arc) = arc_queue.pop_front() {
                let (feasible, _) = self.revise(&constraint_arc, &mut inferred_domains, assignment);
                if !feasible {
                    return (false, vec![]);
                }
            }
        }
        (true, inferred_domains)
    }
```
* `maintaining_arc_consistency` is like `forward_checking` with only one difference; when ever the domain of xi is changed in a constraint like (xi, xj), then it adds all the unassigned neighbors of xi to the queue.
``` rust
    pub fn maintaining_arc_consistency( &self, domains: &Domain, assignment: &Assignment,
                                        mut arc_queue: VecDeque<ConstraintArc>) -> (bool, Domain) {
        let mut inferred_domains = domains.clone();
        while !arc_queue.is_empty() {
            if let Some(constraint_arc) = arc_queue.pop_front() {
                let (feasible, revised) = self.revise(&constraint_arc, &mut inferred_domains, assignment);
                if !feasible {
                    return (false, vec![]);
                }
                if revised {
                    self.generate_arc_constraints(constraint_arc.xi, assignment, &mut arc_queue, constraint_arc.xj);
                }

            }
        }
        (true, inferred_domains)
    }
```
# Results
For small inputs the overhead of arc consistency is high and thus, backtracking without inference is generally faster. But as the size of input gets larger arc consistency and inference start to shine.
### Test case 1
Input: 
```
6 6
1 2 3 1 2 1
1 2 1 3 1 2
2 1 2 2 2 1
2 1 2 2 1 2
1 0 0 1 0 0
1 0 0 1 0 0
1 0 0 0 0 1
1 1 0 0 1 1
1 1 0 0 1 1
1 0 0 0 0 1
```
Output:
```
           2   1   2   2   2   1
           2   1   2   2   1   2
   1   1       -   +
   2   2       +   -       +   -
   3   1   +           +   -   +
   1   3   -       +   -       -
   2   1   +       -   +
   1   2   -           -   +
```

### Test case 2
Input:
```
10 9
3 4 4 4 0 3 3 3 4 3
3 4 4 3 2 2 2 3 4 4
4 2 3 4 3 3 4 5 3
4 2 4 3 5 1 4 4 4
1 1 1 0 0 1 1 0 0
1 1 1 1 1 1 1 0 0
0 0 1 1 1 1 0 0 1
1 1 1 0 0 1 0 0 1
1 1 1 1 0 0 1 1 1
0 0 1 1 0 0 1 1 1
0 0 0 0 0 0 0 0 1
1 1 0 0 0 0 0 0 1
1 1 1 0 0 0 0 1 1
0 0 1 0 0 0 0 1 1
```
Output:
```
           4   2   3   4   3   3   4   5   3
           4   2   4   3   5   1   4   4   4
   3   3   -   +   -   +   -       +
   4   4   +   -   +   -   +       -   +   -
   4   4   -   +   -   +   -       +   -   +
   4   3   +       +   -   +       -   +   -
   0   2   -                           -
   3   2   +   -           -   +       +
   3   2           -   +           +   -   +
   3   3   +               -   +   -   +   -
   4   4   -       +   -   +   -   +   -   +
   3   4           -   +   -   +   -   +   -

```

|Method|Backtracking without Inference| Forward Checking| AC3 |
|---|---|---|---|
|Input 1|11 ms| 75 ms |  110 ms |
|Input 2|14 ms| 80 ms|  126 ms|



