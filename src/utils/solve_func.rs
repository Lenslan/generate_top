
pub trait SolveFunc {
    // 直接输入增广矩阵
    fn solve(&mut self) -> Option<Vec<usize>>;
}

impl SolveFunc for Vec<Vec<i64>> {
    fn solve(&mut self,) -> Option<Vec<usize>> {
        let num_eqs = self.len();
        let num_vars = self.get(0).map_or(0, |v| v.len() - 1);

        let mut pivot_val = 1;
        for k in 0..num_vars {
            let pivot_row_idx = (k..num_eqs).find(|&i| self[i][k] != 0);

            if let Some(p_idx) = pivot_row_idx {
                self.swap(k, p_idx);
            } else {
                return None;
            }

            let current_pivot = self[k][k];

            for i in (k+1)..num_eqs {
                for j in (k+1)..=num_vars {
                    let term1 = self[i][j] * current_pivot;
                    let term2 = self[i][k] * self[k][j];
                    let numerator = term1 - term2;
                    self[i][j] = numerator / pivot_val;
                }
                self[i][k] = 0;
            }
            pivot_val = current_pivot;
        }

        for i in num_vars..num_eqs {
            if self[i][num_vars] != 0 {
                return None;  // 无解
            }
        }

        let mut x = vec![0i64; num_vars];
        for i in (0..num_vars).rev() {
            let mut sum_ax = 0;
            for j in (i+1)..num_vars {
                let term = self[i][j] * x[j];
                sum_ax = sum_ax + term;
            }

            let rhs = self[i][num_vars] - sum_ax;
            let divisor = self[i][i];

            if rhs % divisor != 0 {
                return None;  // 存在小数
            }
            x[i] = rhs / divisor;
        }
        Some(x.iter().map(|&v| v as usize).collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_solve() {
        let mut a = vec![
            vec![1,1,0,3],
            vec![0,1,1,5],
            vec![1,0,1,4],
            vec![2,2,0,6]
        ];
        let b = a.solve().unwrap();
        println!("{:?}", b);
    }
}