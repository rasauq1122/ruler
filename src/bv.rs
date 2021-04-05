use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::*;

#[derive(Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BV<const N: u32>(Inner);

type Inner = u32;
const INNER_N: u32 = 32;

impl<const N: u32> BV<N> {
    pub const ZERO: Self = Self(0);
    pub const ALL_ONES: Self = Self((!(0 as Inner)) >> (INNER_N - N));
    pub const NEG_ONE: Self = Self::ALL_ONES;
    pub const MIN: Self = Self(1 << (N - 1));
    pub const MAX: Self = Self(Self::ALL_ONES.0 >> 1);

    pub fn new(n: impl Into<Inner>) -> Self {
        Self(n.into() & Self::ALL_ONES.0)
    }

    pub fn wrapping_add(self, rhs: Self) -> Self {
        Self::new(self.0.wrapping_add(rhs.0))
    }

    pub fn wrapping_sub(self, rhs: Self) -> Self {
        Self::new(self.0.wrapping_sub(rhs.0))
    }

    pub fn wrapping_mul(self, rhs: Self) -> Self {
        Self::new(self.0.wrapping_mul(rhs.0))
    }

    pub fn wrapping_neg(self) -> Self {
        Self::new(self.0.wrapping_neg())
    }

    pub fn my_shl(self, rhs: Self) -> Self {
        if rhs.0 >= N {
            Self::ZERO
        } else {
            Self::new(self.0 << rhs.0)
        }
    }

    pub fn my_shr(self, rhs: Self) -> Self {
        if rhs.0 >= N {
            Self::ZERO
        } else {
            Self::new(self.0 >> rhs.0)
        }
    }
}

impl<const N: u32> Not for BV<N> {
    type Output = Self;
    fn not(self) -> Self {
        Self::new(self.0.not())
    }
}

impl<const N: u32> BitAnd for BV<N> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self::new(self.0.bitand(rhs.0))
    }
}

impl<const N: u32> BitOr for BV<N> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self::new(self.0.bitor(rhs.0))
    }
}

impl<const N: u32> BitXor for BV<N> {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self::new(self.0.bitxor(rhs.0))
    }
}

impl<const N: u32> fmt::Debug for BV<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<const N: u32> fmt::Display for BV<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<const N: u32> Distribution<BV<N>> for rand::distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BV<N> {
        let inner: Inner = rng.gen();
        inner.into()
    }
}

impl<const N: u32> std::str::FromStr for BV<N> {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(|inner: Inner| Self::new(inner))
    }
}

impl<const N: u32> From<Inner> for BV<N> {
    fn from(t: Inner) -> Self {
        Self::new(t)
    }
}

#[macro_export]
macro_rules! impl_bv {
    ($n:literal) => {
        use egg::*;
        use $crate::*;

        use std::ops::*;

        use rand_pcg::Pcg64;

        use rand::prelude::*;
        use serde::{Deserialize, Serialize};
        use std::fmt;

        pub type BV = $crate::BV::<$n>;

        egg::define_language! {
            pub enum Math {
                "+" = Add([Id; 2]),
                "--" = Sub([Id; 2]),
                "*" = Mul([Id; 2]),
                "-" = Neg(Id),
                "~" = Not(Id),
                "<<" = Shl([Id; 2]),
                ">>" = Shr([Id; 2]),
                "&" = And([Id; 2]),
                "|" = Or([Id; 2]),
                "^" = Xor([Id; 2]),
                Num(BV),
                Var(egg::Symbol),
            }
        }

        impl SynthLanguage for Math {
            type Constant = BV;

            fn convert_parse(s: &str) -> RecExpr<Self> {
                let s = s
                    .replace("bvadd", "+")
                    .replace("bvsub", "--")
                    .replace("bvmul", "*")
                    .replace("bvand", "&")
                    .replace("bvor", "|")
                    .replace("bvneg", "-")
                    .replace("bvnot", "~")
                    .replace("bvlshr", ">>")
                    .replace("bvshl", "<<")
                    .replace("#b0000", "0")
                    .replace("#b0111", "7")
                    .replace("#b1000", "8")
                    .replace("and", "&")
                    .replace("xor", "^")
                    .replace("or", "|")
                    .replace("not", "~");
                assert!(!s.contains('#'));
                s.parse().unwrap()
            }

            fn eval<'a, F>(&'a self, cvec_len: usize, mut v: F) -> CVec<Self>
            where
                F: FnMut(&'a Id) -> &'a CVec<Self>,
            {
                match self {
                    Math::Neg(a) => map!(v, a => Some(a.wrapping_neg())),
                    Math::Not(a) => map!(v, a => Some(a.not())),

                    Math::Add([a, b]) => map!(v, a, b => Some(a.wrapping_add(*b))),
                    Math::Sub([a, b]) => map!(v, a, b => Some(a.wrapping_sub(*b))),
                    Math::Mul([a, b]) => map!(v, a, b => Some(a.wrapping_mul(*b))),

                    Math::Shl([a, b]) => map!(v, a, b => Some(a.my_shl(*b))),
                    Math::Shr([a, b]) => map!(v, a, b => Some(a.my_shr(*b))),

                    Math::And([a, b]) => map!(v, a, b => Some(*a & *b)),
                    Math::Or([a, b]) => map!(v, a, b => Some(*a | *b)),
                    Math::Xor([a, b]) => map!(v, a, b => Some(*a ^ *b)),

                    Math::Num(n) => vec![Some(n.clone()); cvec_len],
                    Math::Var(_) => vec![],
                }
            }

            fn to_var(&self) -> Option<Symbol> {
                if let Math::Var(sym) = self {
                    Some(*sym)
                } else {
                    None
                }
            }

            fn mk_var(sym: Symbol) -> Self {
                Math::Var(sym)
            }

            fn to_constant(&self) -> Option<&Self::Constant> {
                if let Math::Num(n) = self {
                    Some(n)
                } else {
                    None
                }
            }

            fn mk_constant(c: Self::Constant) -> Self {
                Math::Num(c)
            }

            fn init_synth(synth: &mut Synthesizer<Self>) {
                let consts: Vec<Option<BV>> = (0..1 << 4).map(|i| Some(i.into())).collect();

                let consts = self_product(&consts, synth.params.variables);
                println!("cvec len: {}", consts[0].len());

                let mut egraph = EGraph::new(SynthAnalysis {
                    cvec_len: consts[0].len(),
                });

                egraph.add(Math::Num(0.into()));
                egraph.add(Math::Num(0x7.into()));
                egraph.add(Math::Num(0x8.into()));

                for i in 0..synth.params.variables {
                    let var = Symbol::from(letter(i));
                    let id = egraph.add(Math::Var(var));
                    egraph[id].data.cvec = consts[i].clone();
                }

                synth.egraph = egraph;
            }

            fn make_layer(synth: &Synthesizer<Self>, iter: usize) -> Vec<Self> {
                let mut extract = Extractor::new(&synth.egraph, NumberOfOps);

                // maps ids to n_ops
                let ids: HashMap<Id, usize> = synth
                    .ids()
                    .map(|id| (id, extract.find_best_cost(id)))
                    .collect();

                let mut to_add = vec![];
                for i in synth.ids() {
                    for j in synth.ids() {
                        if ids[&i] + ids[&j] + 1 != iter {
                            continue;
                        }

                        to_add.push(Math::Add([i, j]));
                        to_add.push(Math::Sub([i, j]));
                        to_add.push(Math::Mul([i, j]));

                        if !synth.params.no_shift {
                            to_add.push(Math::Shl([i, j]));
                            to_add.push(Math::Shr([i, j]));
                        }

                        to_add.push(Math::And([i, j]));
                        to_add.push(Math::Or([i, j]));
                        // if !synth.params.no_xor {
                        //     to_add.push(Math::Xor([i, j]));
                        // }
                    }
                    if ids[&i] + 1 != iter {
                        continue;
                    }

                    to_add.push(Math::Not(i));
                    to_add.push(Math::Neg(i));
                }

                log::info!("Made a layer of {} enodes", to_add.len());
                to_add
            }

            fn is_valid(_rng: &mut Pcg64, _lhs: &Pattern<Self>, _rhs: &Pattern<Self>) -> bool {
                true
            }
        }

    };
}

#[cfg(test)]
pub mod tests {
    use super::*;

    type BV4 = BV<4>;

    #[test]
    fn test_bv() {
        assert_eq!(BV4::ALL_ONES.0, 0b1111);
        assert_eq!(BV4::MAX.0, 0b0111);
        assert_eq!(BV4::MIN.0, 0b1000);

        let one = BV4::from(1);

        assert_eq!(BV4::MAX.wrapping_add(one), BV::MIN);
        assert_eq!(BV4::NEG_ONE.wrapping_neg(), one);
        assert_eq!(BV4::MIN.wrapping_mul(BV::NEG_ONE), BV::MIN);
        assert_eq!(BV4::MIN.wrapping_neg(), BV::MIN);
    }
}
