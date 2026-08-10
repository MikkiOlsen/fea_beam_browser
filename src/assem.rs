use nalgebra::{DMatrix, Vector2};
use crate::types::{Node, Element};
use crate::kbeam::kebeam;

pub fn assemble_global_stiffness(nodes: &[Node], elements: &[Element]) -> DMatrix<f64> {
    let dof_per_node = 3;
    let total_dof = nodes.len() * dof_per_node;
    
    let mut k_global = DMatrix::<f64>::zeros(total_dof, total_dof);

    for el in elements {
        let n1_idx = nodes.iter().position(|n| n.id == el.n1_id).unwrap();
        let n2_idx = nodes.iter().position(|n| n.id == el.n2_id).unwrap();

        let p1 = Vector2::new(nodes[n1_idx].pos.x as f64, nodes[n1_idx].pos.y as f64);
        let p2 = Vector2::new(nodes[n2_idx].pos.x as f64, nodes[n2_idx].pos.y as f64);

        let ke = kebeam(p1, p2, el.e, el.a, el.i);

        let ig = [
            n1_idx * 3, n1_idx * 3 + 1, n1_idx * 3 + 2,
            n2_idx * 3, n2_idx * 3 + 1, n2_idx * 3 + 2,
        ];

        for i in 0..6 {
            for j in 0..6 {
                k_global[(ig[i], ig[j])] += ke[(i, j)];
            }
        }
    }

    k_global
}

pub fn apply_boundary_conditions(k_global: &mut DMatrix<f64>, nodes: &[Node]) {
    let total_dof = nodes.len() * 3;

    for (i, node) in nodes.iter().enumerate() {
        let dof_x = i * 3;
        let dof_y = i * 3 + 1;
        let dof_rot = i * 3 + 2;

        let (fix_x, fix_y, fix_rot) = match node.support {
            crate::types::SupportType::Free => (false, false, false),
            crate::types::SupportType::Pin => (true, true, false),
            crate::types::SupportType::RollerH => (false, true, false), // Y is fixed
            crate::types::SupportType::RollerV => (true, false, false), // X is fixed
            crate::types::SupportType::Fixed => (true, true, true),     // Clamped
        };

        let dofs_to_fix = [
            (fix_x, dof_x),
            (fix_y, dof_y),
            (fix_rot, dof_rot),
        ];

        for (is_fixed, dof) in dofs_to_fix.iter() {
            if *is_fixed {
                for j in 0..total_dof {
                    k_global[(*dof, j)] = 0.0;
                    k_global[(j, *dof)] = 0.0;
                }
                k_global[(*dof, *dof)] = 1.0;
            }
        }
    }
}