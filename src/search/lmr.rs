pub const LM_REDUCTIONS_TABLE: [[i64; 64]; 64] = init_lmr_table();

const PRIMES: [usize; 18] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61,
];

const PRIME_LNS: [f64; 18] = [
    0.6931471805599453, // 2
    1.0986122886681098, // 3
    1.6094379124341003, // 5
    1.9459101490553132, // 7
    2.3978952727983707, // 11
    2.5649493574615367, // 13
    2.833213344056216,  // 17
    2.9444389791664403, // 19
    3.1354942159291497, // 23
    3.367295829986474,  // 29
    3.4339872044851463, // 31
    3.6109179126442243, // 37
    3.713572066704308,  // 41
    3.7612001156935624, // 43
    3.8501476017100584, // 47
    3.970291913552122,  // 53
    4.07753744390572,   // 59,
    4.110873864173311,  // 61
];

// Any sane person would just store the logs of every number from 1 to 63.
// Unfortunately, I have a maths degree so I want to do this the funny way and only store the logs of primes.
// I like the fundamental theorem of arithmetic :)
const fn const_ln(mut n: usize) -> f64 {
    if n == 0 {
        return f64::NEG_INFINITY;
    }
    if n == 1 {
        return 0.0;
    }

    let mut ln_n = 0.0;
    let mut i = 0;

    while i < 18 {
        let p = PRIMES[i];
        let ln_p = PRIME_LNS[i];

        while n % p == 0 {
            n /= p;
            ln_n += ln_p;
        }
        i += 1
    }

    ln_n
}

const fn init_lmr_table() -> [[i64; 64]; 64] {
    let mut table = [[0; 64]; 64];
    let mut d = 1;
    while d < 64 {
        let mut m = 1;
        while m < 64 {
            let ln_d = const_ln(d);
            let ln_m = const_ln(m);
            // Ethereal's LMR formula.
            let r = 0.7844 + (ln_d * ln_m) / 2.4696;
            let adjusted_r = if d < 3 { 0 } else { r as i64 }; // if depth < 3, don't reduce at all.
            table[d][m] = adjusted_r;
            m += 1;
        }

        d += 1;
    }
    table
}
