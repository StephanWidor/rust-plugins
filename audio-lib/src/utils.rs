mod private {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

pub trait Float:
    'static
    + num_traits::Float
    + num_traits::ConstZero
    + num_traits::ConstOne
    + num_traits::FloatConst
    + std::ops::AddAssign
    + std::ops::MulAssign
    + std::ops::DivAssign
    + serde::Serialize
    + serde::de::DeserializeOwned
    + std::marker::Send
    + std::marker::Sync
    + private::Sealed
{
    const TWO: Self;
    const FOUR: Self;
    const TWO_PI: Self;
    const TEN: Self;
    const TWENTY: Self;

    const ONE_HALF: Self;
    const ONE_TWENTIETH: Self;

    fn from_float<T: Float>(f: T) -> Self {
        Self::from(f).unwrap()
    }
    fn from_integral<I: num::PrimInt>(i: I) -> Self {
        Self::from(i).unwrap()
    }
    fn cast<T: Float>(self) -> T {
        <T as num::NumCast>::from(self).unwrap()
    }
}
impl Float for f32 {
    const TWO: f32 = 2.0_f32;
    const FOUR: f32 = 4.0_f32;
    const TWO_PI: Self = std::f32::consts::TAU;
    const TEN: f32 = 10.0_f32;
    const TWENTY: f32 = 20.0_f32;

    const ONE_HALF: f32 = 0.5_f32;
    const ONE_TWENTIETH: f32 = 0.05_f32;
}
impl Float for f64 {
    const TWO: f64 = 2.0;
    const FOUR: f64 = 4.0;
    const TWO_PI: Self = std::f64::consts::TAU;
    const TEN: f64 = 10.0;
    const TWENTY: f64 = 20.0;

    const ONE_HALF: f64 = 0.5;
    const ONE_TWENTIETH: Self = 0.05;
}

pub fn get_length<F: Float>(range: &std::ops::RangeInclusive<F>) -> F {
    *range.end() - *range.start()
}

pub fn is_power_of_two(i: usize) -> bool {
    i != 0 && (i & (i - 1)) == 0
}

#[allow(type_alias_bounds)]
pub type PolynomRoots<F: Float> = smallvec::SmallVec<[num::Complex<F>; 2]>;

pub fn frequency_to_log<F: Float>(frequency: F) -> F {
    if frequency > F::ZERO {
        F::log10(frequency)
    } else {
        F::neg_infinity()
    }
}

pub fn log_to_frequency<F: Float>(log_frequency: F) -> F {
    if log_frequency == F::neg_infinity() {
        F::ZERO
    } else {
        F::TEN.powf(log_frequency)
    }
}

pub fn amplitude_to_db<F: Float>(amplitude: F) -> F {
    if amplitude > F::ZERO {
        F::TWENTY * F::log10(amplitude)
    } else {
        F::neg_infinity()
    }
}

pub fn db_to_amplitude<F: Float>(db: F) -> F {
    if db == F::neg_infinity() {
        F::ZERO
    } else {
        F::TEN.powf(db * F::ONE_TWENTIETH)
    }
}

pub fn omega<F: Float>(frequency: F, sample_rate: F) -> F {
    F::TWO_PI * (frequency / sample_rate)
}

pub fn make_impulse_response<F: Float>(
    process_function: &mut impl FnMut(F) -> F,
    rel_eps: F,
    release_time: F,
    max_time: F,
    sample_rate: F,
) -> Vec<F> {
    let mut response = Vec::new();
    let mut envelope = crate::envelope_follower::EnvelopeFollower::from_attack_and_release_time(
        F::ZERO,
        release_time,
        sample_rate,
    );
    let mut max = envelope
        .process(*response.push_mut(process_function(F::ONE)))
        .abs();
    for _ in 1..(max_time * sample_rate).to_usize().unwrap() {
        let r = envelope.process(response.push_mut(process_function(F::ZERO)).abs());
        if r <= rel_eps * max {
            break;
        }
        max = max.max(r);
    }
    response
}

pub fn make_gain_db_response<F: Float>(
    complex_frequency_response: impl Fn(F) -> num::Complex<F>,
) -> impl Fn(F) -> F {
    move |frequency| amplitude_to_db(complex_frequency_response(frequency).norm())
}

pub fn make_phase_response<F: Float>(
    complex_frequency_response: impl Fn(F) -> num::Complex<F>,
) -> impl Fn(F) -> F {
    move |frequency| complex_frequency_response(frequency).arg()
}

/// complex roots of polynom c2*x^2 + c1*x + c0
pub fn polynom_roots<F: Float>(c2: F, c1: F, c0: F) -> PolynomRoots<F> {
    if c2 == F::ZERO {
        if c1 == F::ZERO {
            if c0 == F::ZERO {
                // any x is a solution here, let's just return zero
                return PolynomRoots::from_elem(num::Complex::from(F::ZERO), 1);
            }
            return PolynomRoots::new();
        }
        return PolynomRoots::from_elem(num::Complex::from(-c0 / c1), 1);
    }
    let p = c1 / c2;
    let q = c0 / c2;
    let root_arg = p * p / F::FOUR - q;
    let p_half = num::Complex::from(p * F::ONE_HALF);
    if root_arg == F::ZERO {
        return PolynomRoots::from_elem(-p_half, 1);
    }

    let root_val = num::Complex::from(root_arg).sqrt();
    PolynomRoots::from_slice(&[-p_half - root_val, -p_half + root_val])
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use more_asserts::assert_le;

    use super::*;

    #[test]
    fn roundtrip_amplitude_db() {
        let n = 100;
        let step = 1.0 / (n as f64);
        for i in 0..(n + 1) {
            let amplitude = (i as f64) * step;
            let db = amplitude_to_db(amplitude);
            let amplitude_back = db_to_amplitude(db);
            assert_approx_eq!(amplitude, amplitude_back);
        }
    }

    #[test]
    fn check_explicit_polynom_roots() {
        let check_solutions = |solutions: PolynomRoots<f64>, expected: &Vec<num::Complex<f64>>| {
            assert_eq!(solutions.len(), expected.len());
            for solution in solutions.iter() {
                let find_index = expected
                    .into_iter()
                    .position(|x| (x - solution).norm() <= 1e-5);
                assert_ne!(find_index, None);
            }
        };

        check_solutions(polynom_roots(0.0, 0.0, 0.0), &vec![num::Complex::from(0.0)]);
        check_solutions(polynom_roots(0.0, 0.0, 0.1), &Vec::new());
        check_solutions(
            polynom_roots(0.0, 2.0, 6.0),
            &vec![num::Complex::from(-3.0)],
        );
        check_solutions(
            polynom_roots(1.6, 0.0, 0.4),
            &vec![
                num::Complex { re: 0.0, im: 0.5 },
                num::Complex { re: 0.0, im: -0.5 },
            ],
        );
        check_solutions(
            polynom_roots(1.0, 0.0, -4.0),
            &vec![num::Complex::from(-2.0), num::Complex::from(2.0)],
        );
    }

    #[test]
    fn validate_some_polynom_roots() {
        let check_solutions = |c2, c1, c0| {
            let solutions = polynom_roots(c2, c1, c0);
            for solution in solutions {
                let eval: num::Complex<f64> = solution * solution * c2 + solution * c1 + c0;
                let abs_eval = eval.norm();
                assert_le!(abs_eval, 1e-10);
            }
        };

        check_solutions(1.2, 2.3, 3.4);
        check_solutions(-17.2, 23.5, 0.004);
    }
}
