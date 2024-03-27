use dep::std::collections::vec::Vec;
use dep::std;


fn main(x : [u32; 100], y: pub [u32; 100]) {
    let mut sum:u32 = 0;
    for i in 0..x.len() {
        let mut diff = 0;
        if(x[i] > y[i])
        {
            let diff = x[i] - y[i];
            sum += diff * diff;
        }
        else
        {
            let diff = y[i] - x[i];
            sum += diff * diff;
        }
        sum += diff * diff;
    }
    assert(sum <= 700);
}
