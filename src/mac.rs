use crate::csp::*;
use std::collections::VecDeque;

impl CSP {

    pub fn maintaining_arc_consistency(
        &self,
        domains: &Domain,
        assignment: &Assignment,
        mut arc_queue: VecDeque<ConstraintArc>
    ) -> (bool, Domain) {
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

}
