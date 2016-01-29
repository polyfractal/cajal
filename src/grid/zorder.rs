
// Credit to: https://fgiesen.wordpress.com/2009/12/13/decoding-morton-codes/

pub fn xy_to_z(x: u32, y: u32) -> u32 {
    (split_by_2(y) << 1) + split_by_2(x)
}

fn split_by_2(x: u32) -> u32  {
    let mut x = x & 0x0000ffff;
    x = (x ^ (x <<  8)) & 0x00ff00ff;
    x = (x ^ (x <<  4)) & 0x0f0f0f0f;
    x = (x ^ (x <<  2)) & 0x33333333;
    x = (x ^ (x <<  1)) & 0x55555555;
    x
}

pub fn z_to_xy(z: u32) -> (u32, u32) {
    (compact_by_2(z), compact_by_2(z >> 1))
}

fn compact_by_2(z:u32) -> u32 {
    let mut x = z & 0x55555555;
    x = (x ^ (x >>  1)) & 0x33333333;
    x = (x ^ (x >>  2)) & 0x0f0f0f0f;
    x = (x ^ (x >>  4)) & 0x00ff00ff;
    x = (x ^ (x >>  8)) & 0x0000ffff;
    x
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn simple_conversion() {
        for x in 0..100 {
            for y in 0..100 {
                let z = xy_to_z(x,y);
                let (x_r,y_r) = z_to_xy(z);
                assert!((x_r,y_r) == (x,y));
            }
        }
    }

    #[test]
    fn z_pattern() {
        /*
               0 1    2  3
               ___________
            0 |0, 1   4, 5
            1 |2, 3   6, 7
              |
            2 |8, 9   12,13
            3 |10,11  14,15
        */

        let z = xy_to_z(0,0);
        assert!(z == 0);
        let z = xy_to_z(1,0);
        assert!(z == 1);
        let z = xy_to_z(0,1);
        assert!(z == 2);
        let z = xy_to_z(1,1);
        assert!(z == 3);

        let z = xy_to_z(2,0);
        assert!(z == 4);
        let z = xy_to_z(3,0);
        assert!(z == 5);
        let z = xy_to_z(2,1);
        assert!(z == 6);
        let z = xy_to_z(3,1);
        assert!(z == 7);

        let z = xy_to_z(0,2);
        assert!(z == 8);
        let z = xy_to_z(1,2);
        assert!(z == 9);
        let z = xy_to_z(0,3);
        assert!(z == 10);
        let z = xy_to_z(1,3);
        assert!(z == 11);

        let z = xy_to_z(2,2);
        assert!(z == 12);
        let z = xy_to_z(3,2);
        assert!(z == 13);
        let z = xy_to_z(2,3);
        assert!(z == 14);
        let z = xy_to_z(3,3);
        assert!(z == 15);
    }


}
