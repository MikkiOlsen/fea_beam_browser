use crate::assem;
use crate::types::{Element, Node};
use nalgebra::{DMatrix, DVector};

pub struct FeManager {
    pub stiffness_matrix: DMatrix<f64>,
    pub force_vector: DVector<f64>,
    pub displacements: DVector<f64>,
    pub solver_status: String,
}

impl FeManager {
    pub fn new() -> Self {
        Self {
            stiffness_matrix: DMatrix::zeros(0, 0),
            force_vector: DVector::zeros(0),
            displacements: DVector::zeros(0),
            solver_status: String::new(),
        }
    }

    pub fn build_and_solve(&mut self, nodes: &[Node], elements: &[Element]) {
        let total_dofs = nodes.len() * 3;
        if total_dofs == 0 {
            return;
        }

        self.stiffness_matrix = assem::assemble_global_stiffness(nodes, elements);
        assem::apply_boundary_conditions(&mut self.stiffness_matrix, nodes);

        self.force_vector = DVector::zeros(total_dofs);
        self.displacements = DVector::zeros(total_dofs);

        for (i, node) in nodes.iter().enumerate() {
            self.force_vector[i * 3] += node.fx;
            self.force_vector[i * 3 + 1] += node.fy;
            self.force_vector[i * 3 + 2] += node.m;
        }

        for el in elements {
            if el.w != 0.0 {
                let n1_idx = nodes.iter().position(|n| n.id == el.n1_id).unwrap();
                let n2_idx = nodes.iter().position(|n| n.id == el.n2_id).unwrap();
                let p1 = nodes[n1_idx].pos;
                let p2 = nodes[n2_idx].pos;

                let dx = (p2.x - p1.x) as f64;
                let dy = (p2.y - p1.y) as f64;
                let l = (dx * dx + dy * dy).sqrt();

                if l > 0.0 {
                    let nx = dx / l;
                    let ny = dy / l;
                    let v = el.w * l / 2.0;
                    let mom = el.w * l * l / 12.0;
                    let f_local = nalgebra::Vector6::new(0.0, v, mom, 0.0, v, -mom);

                    #[rustfmt::skip]
                    let ae_t = nalgebra::Matrix6::new(
                        nx, -ny, 0.0, 0.0, 0.0, 0.0,
                        ny,  nx, 0.0, 0.0, 0.0, 0.0,
                        0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
                        0.0, 0.0, 0.0, nx, -ny, 0.0,
                        0.0, 0.0, 0.0, ny,  nx, 0.0,
                        0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                    );

                    let f_global = ae_t * f_local;
                    let ig = [
                        n1_idx * 3,
                        n1_idx * 3 + 1,
                        n1_idx * 3 + 2,
                        n2_idx * 3,
                        n2_idx * 3 + 1,
                        n2_idx * 3 + 2,
                    ];
                    for i in 0..6 {
                        self.force_vector[ig[i]] += f_global[i];
                    }
                }
            }
        }

        for (i, node) in nodes.iter().enumerate() {
            let (fix_x, fix_y, fix_rot) = match node.support {
                crate::types::SupportType::Free => (false, false, false),
                crate::types::SupportType::Pin => (true, true, false),
                crate::types::SupportType::RollerH => (false, true, false),
                crate::types::SupportType::RollerV => (true, false, false),
                crate::types::SupportType::Fixed => (true, true, true),
            };
            if fix_x {
                self.force_vector[i * 3] = 0.0;
            }
            if fix_y {
                self.force_vector[i * 3 + 1] = 0.0;
            }
            if fix_rot {
                self.force_vector[i * 3 + 2] = 0.0;
            }
        }

        let k_decomp = self.stiffness_matrix.clone().lu();

        if let Some(u) = k_decomp.solve(&self.force_vector) {
            self.displacements = u;
            self.solver_status = "SUCCESS".to_string();
        } else {
            self.displacements = DVector::zeros(0);
            self.solver_status = "ERROR: Singular Matrix! Structure is unstable.".to_string();
        }
    }
}
