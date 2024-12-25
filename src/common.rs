macro_rules! bit {
    ($a: expr, $n: expr) => {
        if ($a & (1 << $n)) != 0 {
            1
        } else {
            0
        }
    };
}

pub(crate) use bit;

macro_rules! bit_set {
    ($a: expr, $n: expr, $on: expr) => {{
        if $on != 0 {
            $a |= 1 << $n; 
        } else {
            $a &= !(1 << $n); 
        }
        $a
    }};
}


pub(crate) use bit_set;

macro_rules! between {
    ($a: expr, $b: expr, $c: expr) => {
        ($a >= $b && $a <= $c)
    };
}

pub fn delay(ms: u32) {}
