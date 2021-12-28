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
}
