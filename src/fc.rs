use crate::csp::*;
use std::collections::VecDeque;

impl CSP {

    pub fn forward_checking(
        &self,
        domains: &Domain,
        assignment: &Assignment,
        mut arc_queue: VecDeque<ConstraintArc>
    ) -> (bool, Domain) {
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

    fn fc_revise(&self, constraint_arc: ConstraintArc, inferred_domains: &mut Domain, assignment: &Assignment) -> (bool, bool, bool) {
        match constraint_arc.constraint {
            Constraint::NeighborBased(pole_xi, pole_xj) => {
                self.fc_revise_neighbor_constraint(constraint_arc.xi, constraint_arc.xj, pole_xi, pole_xj, inferred_domains, assignment)
            },
            Constraint::LimitBased(pole_xi, pole_xj) => {
                self.fc_revise_limit_constraint(constraint_arc.xi, constraint_arc.xj, pole_xi, pole_xj, inferred_domains, assignment)
            }
        }
    }

    // Revise the domains based on the neighboring cells and their signs. (two positives or two
    // negatives can't be next to each other.
    // returns: (feasible, xi_domain_changed, xj_domain_change)
    // feasible is false if any domain is reduced to zeo
    fn fc_revise_neighbor_constraint(&self, xi_index: VariableIndex, xj_index: VariableIndex, pole_xi: PoleNumber, pole_xj: PoleNumber, inferred_domains: &mut Domain, assignment: &Assignment) -> (bool, bool, bool) {
        if xi_index == xj_index || assignment[xj_index] != Value::Unassigned {
            return (false, false, false)
        }
        let xi_value = assignment[xi_index];
        let mut xj_domain_changed = false;
        let mut xi_domain_changed = false;

        match xi_value {
            Value::Pole1PositivePole2Negative => {
                if let Some(to_be_removed_value) = CSP::get_neighbor_pole_based_inconsistent_value(xi_value, pole_xi, pole_xj) {
                    xj_domain_changed = CSP::remove_value_from_domain(to_be_removed_value, &mut inferred_domains[xj_index]);
                }
            },
            Value::Pole2PositivePole1Negative => {
                if let Some(to_be_removed_value) = CSP::get_neighbor_pole_based_inconsistent_value(xi_value, pole_xi, pole_xj) {
                    xj_domain_changed = CSP::remove_value_from_domain(to_be_removed_value, &mut inferred_domains[xj_index]);
                }
            },
            Value::Unassigned => {
                // for each value in xi domain
                // if there are no values avalaible in xj's domain that are consistent with the
                // current value of xi, then delete the current value of xi
                let mut to_be_deleted: Vec<Value> = Vec::new();
                let mut constraint_count = 0;
                for xi_value in &inferred_domains[xi_index] {
                    if let Some(value) = CSP::get_neighbor_pole_based_inconsistent_value(*xi_value, pole_xi, pole_xj) {
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
                for value in to_be_deleted {
                    xi_domain_changed = CSP::remove_value_from_domain(value, &mut inferred_domains[xi_index]);
                }
            },
            _ => {}
        }

        if inferred_domains[xi_index].len() == 0 || inferred_domains[xj_index].len() == 0 {
            return (false, false, false)
        }
        (true, xi_domain_changed, xj_domain_changed)
    }

    fn fc_revise_limit_constraint(&self, xi_index: VariableIndex, xj_index: VariableIndex, pole_xi: PoleNumber, pole_xj: PoleNumber, inferred_domains: &mut Domain, assignment: &Assignment) -> (bool, bool, bool) {
        (true, true, true)
    }
 
}
