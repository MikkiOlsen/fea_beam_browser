use nalgebra::{Matrix6, Vector2};

pub fn kebeam(p1: Vector2<f64>, p2: Vector2<f64>, e: f64, a: f64, i: f64) -> Matrix6<f64> {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let l = (dx * dx + dy * dy).sqrt();

    if l == 0.0 {
        return Matrix6::zeros();
    }

    let nx = dx / l;
    let ny = dy / l;
    let l2 = l * l;
    let l3 = l * l * l;

    #[rustfmt::skip]
    let kec = Matrix6::new(
         e*a/l ,  0.0          ,  0.0         , -e*a/l ,  0.0          ,  0.0,
         0.0   ,  12.0*e*i/l3  ,  6.0*e*i/l2  ,  0.0   , -12.0*e*i/l3  ,  6.0*e*i/l2,
         0.0   ,  6.0*e*i/l2   ,  4.0*e*i/l   ,  0.0   , -6.0*e*i/l2   ,  2.0*e*i/l,
        -e*a/l ,  0.0          ,  0.0         ,  e*a/l ,  0.0          ,  0.0,
         0.0   , -12.0*e*i/l3  , -6.0*e*i/l2  ,  0.0   ,  12.0*e*i/l3  , -6.0*e*i/l2,
         0.0   ,  6.0*e*i/l2   ,  2.0*e*i/l   ,  0.0   , -6.0*e*i/l2   ,  4.0*e*i/l,
    );

    #[rustfmt::skip]
    let ae = Matrix6::new(
         nx ,  ny , 0.0 , 0.0 , 0.0 , 0.0,
        -ny ,  nx , 0.0 , 0.0 , 0.0 , 0.0,
        0.0 , 0.0 , 1.0 , 0.0 , 0.0 , 0.0,
        0.0 , 0.0 , 0.0 ,  nx ,  ny , 0.0,
        0.0 , 0.0 , 0.0 , -ny ,  nx , 0.0,
        0.0 , 0.0 , 0.0 , 0.0 , 0.0 , 1.0,
    );

    ae.transpose() * kec * ae
}
