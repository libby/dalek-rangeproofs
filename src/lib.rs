// -*- coding: utf-8; mode: rust; -*-
//
// To the extent possible under law, the authors have waived all
// copyright and related or neighboring rights to dalek-rangeproofs,
// using the Creative Commons "CC0" public domain dedication.  See
// <http://creativecommons.org/publicdomain/zero/.0/> for full
// details.
//
// Authors:
// - Isis Agora Lovecruft <isis@patternsinthevoid.net>
// - Henry de Valence <hdevalence@hdevalence.ca>

//! A pure-Rust implementation of the Back-Maxwell rangeproof scheme defined in
//! [_Confidential Assets_ (2017) by Poelstra, Back, Friedenbach, Maxwell,
//! Wuille](https://blockstream.com/bitcoin17-final41.pdf).
//!
//! The scheme is instantiated using the Decaf group on Curve25519, as
//! implemented in
//! [`curve25519-dalek`](https://github.com/isislovecruft/curve25519-dalek).
//! Note that since the Decaf implementation in `curve25519-dalek` is
//! currently **UNFINISHED, UNREVIEWED, AND EXPERIMENTAL**, so is this
//! library.
//!
//! This implementation hardcodes the ring size `m = 3`, as this is
//! the most efficient choice.  The number of rings `n` determines the
//! range `[0,3^n]`, as well as the size and speed of the proof.
//!
//! # Examples
//!
//! Suppose we want to prove that `134492616741` is within `[0,3^40]`.
//! Since 3^40 ≅ 0.66 (2^64 ), this is just slightly narrower than the
//! range of a `u64`.
//!
//! To construct a proof that `134492616741` is within `[0,3^40]`,
//! first choose orthogonal basepoints.  Usually the basepoints would
//! be set or distributed as system-wide parameters.  Here we choose
//! `G` to be the Decaf coset containing the ed25519 basepoint and
//! generate the second basepoint as `H = Hash(G)`.  (Technically, `G`
//! is actually a `DecafBasepointTable`, which implements
//! `Mul<&Scalar>` using a precomputed table).
//!
//! ```
//! # extern crate curve25519_dalek;
//! # extern crate sha2;
//! # fn main() {
//! use sha2::Sha256;
//!
//! use curve25519_dalek::constants as dalek_constants;
//! use curve25519_dalek::decaf::{DecafBasepointTable, DecafPoint};
//! use curve25519_dalek::scalar::Scalar;
//!
//! let G = &dalek_constants::DECAF_ED25519_BASEPOINT_TABLE;
//! let H = DecafPoint::hash_from_bytes::<Sha256>(G.basepoint().compress().as_bytes());
//! # }
//!
//! ```
//!
//! You'll also need your value and a CSPRNG:
//!
//! ```
//! # extern crate rand;
//! # fn main() {
//! use rand::OsRng;
//!
//! let mut csprng = OsRng::new().unwrap();
//! let value = 134492616741;
//! # }
//! ```
//!
//! We can now create the rangeproof, like so:
//!
//! ```
//! # extern crate dalek_rangeproofs;
//! # extern crate curve25519_dalek;
//! # extern crate sha2;
//! # extern crate rand;
//! # fn main() {
//! # use curve25519_dalek::constants as dalek_constants;
//! # use curve25519_dalek::decaf::{DecafBasepointTable, DecafPoint};
//! # use curve25519_dalek::scalar::Scalar;
//! # use rand::OsRng;
//! # use sha2::Sha256;
//! #
//! # let G = &dalek_constants::DECAF_ED25519_BASEPOINT_TABLE;
//! # let H = DecafPoint::hash_from_bytes::<Sha256>(G.basepoint().compress().as_bytes());
//! #
//! # let mut csprng = OsRng::new().unwrap();
//! # let value = 134492616741;
//! use dalek_rangeproofs::RangeProof;
//!
//! let (proof, commitment, blinding) =
//!     RangeProof::create(40, value, G, &H, &mut csprng).unwrap();
//! # }
//! ```
//!
//! The output is the proof `proof`, as well as `commitment =
//! blinding*G + value*H`.
//!
//! We can serialize the proof using [Serde](https://serde.rs).  Here, we use [CBOR](http://cbor.io).
//!
//! ```
//! # extern crate dalek_rangeproofs;
//! # extern crate curve25519_dalek;
//! # extern crate rand;
//! # extern crate sha2;
//! extern crate serde_cbor;
//! # fn main() {
//! # use curve25519_dalek::constants as dalek_constants;
//! # use curve25519_dalek::decaf::{DecafBasepointTable, DecafPoint};
//! # use curve25519_dalek::scalar::Scalar;
//! # use rand::OsRng;
//! # use sha2::Sha256;
//! #
//! # let G = &dalek_constants::DECAF_ED25519_BASEPOINT_TABLE;
//! # let H = DecafPoint::hash_from_bytes::<Sha256>(G.basepoint().compress().as_bytes());
//! # let mut csprng = OsRng::new().unwrap();
//! # let value = 134492616741;
//! # use dalek_rangeproofs::RangeProof;
//! # let (proof, commitment, blinding)
//! #     = RangeProof::create(40, value, G, &H, &mut csprng).unwrap();
//! 
//! let proof_bytes: Vec<u8> = serde_cbor::ser::to_vec_packed(&proof).unwrap();
//! assert_eq!(proof_bytes.len(), 4125);
//! # }
//! ```
//!
//! In this case, the `serde_cbor`-encoded proof statement is
//! approximately 6.5% larger than the optimal 3872 = 32(1+3*40)
//! bytes.  Another party can verify the proof statement from
//! `proof_bytes` by doing:
//!
//! ```
//! # extern crate dalek_rangeproofs;
//! # extern crate curve25519_dalek;
//! # extern crate rand;
//! # extern crate sha2;
//! extern crate serde_cbor;
//! # fn main() {
//! # use curve25519_dalek::constants as dalek_constants;
//! # use curve25519_dalek::decaf::{DecafBasepointTable, DecafPoint};
//! # use curve25519_dalek::scalar::Scalar;
//! # use rand::OsRng;
//! # use sha2::Sha256;
//! #
//! # let G = &dalek_constants::DECAF_ED25519_BASEPOINT_TABLE;
//! # let H = DecafPoint::hash_from_bytes::<Sha256>(G.basepoint().compress().as_bytes());
//! # let mut csprng = OsRng::new().unwrap();
//! # let value = 134492616741;
//! use dalek_rangeproofs::RangeProof;
//! # let (proof, commitment, blinding)
//! #     = RangeProof::create(40, value, G, &H, &mut csprng).unwrap();
//! # let proof_bytes: Vec<u8> = serde_cbor::to_vec(&proof).unwrap();
//! let proof: RangeProof = serde_cbor::from_slice(&proof_bytes).unwrap();
//!
//! let C_option = proof.verify(40, G, &H);
//! assert!(C_option.is_some());
//!
//! let C = C_option.unwrap();
//! # }
//! ```
//!
//! If the proof is well-formed, `verify` returns the commitment to
//! the value.  Since the commitment is the output of the
//! verification, the verifier is assured it opens to a value in the
//! range `[0,3^n]`. However, without knowing both `blinding` and the
//! actual `value`, the verifier cannot open this commitment, because
//! Pedersen commitments are computationally binding and perfectly
//! hiding (in addition to being additively homomorphic, a feature
//! used within this scheme).
//!
//! Later, to open this commitment, the prover could reveal the value
//! and the blinding, allowing others to check that `commitment =
//! blinding*G + value*H`.
//!
//! ```
//! # extern crate dalek_rangeproofs;
//! # extern crate curve25519_dalek;
//! # extern crate rand;
//! # extern crate sha2;
//! # fn main() {
//! # use curve25519_dalek::constants as dalek_constants;
//! # use curve25519_dalek::decaf::{DecafBasepointTable, DecafPoint};
//! # use curve25519_dalek::scalar::Scalar;
//! # use rand::OsRng;
//! # use sha2::Sha256;
//! #
//! # let G = &dalek_constants::DECAF_ED25519_BASEPOINT_TABLE;
//! # let H = DecafPoint::hash_from_bytes::<Sha256>(G.basepoint().compress().as_bytes());
//! # let mut csprng = OsRng::new().unwrap();
//! # let value = 134492616741;
//! # use dalek_rangeproofs::RangeProof;
//! # let (proof, commitment, blinding)
//! #     = RangeProof::create(40, value, G, &H, &mut csprng).unwrap();
//! # let C = proof.verify(40, G, &H).unwrap();
//! let C_hat = &(G * &blinding) + &(&H * &Scalar::from_u64(value));
//!
//! assert_eq!(C_hat, C);
//! # }
//! ```

#![cfg_attr(feature = "bench", feature(test))]

#![allow(non_snake_case)]
#![deny(missing_docs)]

#[cfg(all(test, feature = "bench"))]
extern crate test;

extern crate curve25519_dalek;
extern crate sha2;

extern crate rand;

#[macro_use]
extern crate serde_derive;

use rand::Rng;

use sha2::Sha512;
use sha2::Digest;

// XXX rewrite curve25519_dalek to have nicer imports.
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::curve::{Identity};
use curve25519_dalek::decaf::{DecafPoint, DecafBasepointTable};
use curve25519_dalek::decaf::vartime;
use curve25519_dalek::subtle::CTAssignable;
use curve25519_dalek::subtle::bytes_equal_ct;
use curve25519_dalek::subtle::byte_is_nonzero;

/// A Back-Maxwell rangeproof, which proves in zero knowledge that a
/// number is in a range `[0,m^n]`.  We hardcode `m = 3` as this is
/// the most efficient.
///
/// The size of the proof and the cost of verification are
/// proportional to `n`.
#[derive(Serialize, Deserialize)]
pub struct RangeProof {
    e_0: Scalar,
    C: Vec<DecafPoint>,
    s_1: Vec<Scalar>,
    s_2: Vec<Scalar>,
}

/// The maximum allowed bound for the rangeproof.  Currently this is
/// set to 41, because we only implement conversion to base 3 digits
/// for `u64`s, and 3^41 is the least power of 3 greater than `2^64`.
pub const RANGEPROOF_MAX_N: usize = 41;

impl RangeProof {
    /// Verify the rangeproof, returning a Pedersen commitment to the
    /// in-range value if successful.
    pub fn verify(
        &self,
        n: usize,
        G: &DecafBasepointTable,
        H: &DecafPoint,
    ) -> Option<DecafPoint> {
        // Calling verify with n out of bounds is a programming error.
        if n > RANGEPROOF_MAX_N {
            panic!("Error: called create_vartime with too large bound 3^n, n = {}", n);
        }
        
        // If the lengths of any of the arrays don't match, the proof
        // is malformed.
        if n != self.C.len() {
            return None;
        } else if n != self.s_1.len() {
            return None;
        } else if n != self.s_2.len() {
            return None;
        }
        
        let mut e_0_hash = Sha512::default();
        let mut C = DecafPoint::identity();
        // mi_H = m^i * H = 3^i * H in the loop below
        let mut mi_H = *H;

        for i in 0..n {
            let mi2_H = &mi_H + &mi_H;

            let Ci_minus_miH = &self.C[i] - &mi_H;
            let P = vartime::k_fold_scalar_mult(&[self.s_1[i], -&self.e_0],
                                                &[G.basepoint(), Ci_minus_miH]);
            let ei_1 = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());

            let Ci_minus_2miH = &self.C[i] - &mi2_H;
            let P = vartime::k_fold_scalar_mult(&[self.s_2[i], -&ei_1],
                                                &[G.basepoint(), Ci_minus_2miH]);
            let ei_2 = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());

            let Ri = &self.C[i] * &ei_2;
            e_0_hash.input(Ri.compress().as_bytes());
            C = &C + &self.C[i];

            // Set mi_H <-- 3*m_iH, so that mi_H is always 3^i * H in the loop
            mi_H = &mi_H + &mi2_H;
        }

        let e_0_hat = Scalar::from_hash(e_0_hash);

        if e_0_hat == self.e_0 {
            return Some(C);
        } else {
            return None;
        }
    }

    /// Construct a rangeproof for `value`, in variable time.
    ///
    /// # Inputs
    ///
    /// * `n`, so that the range is `[0,3^n]` with `n < RANGEPROOF_MAX_N`;
    /// * The `value` to prove within range `[0,3^n]`;
    /// * `csprng`, an implementation of `rand::Rng`, which should be
    /// cryptographically secure.
    ///
    /// # Returns
    ///
    /// If `value` is not in the range `[0,3^n]`, return None.
    /// 
    /// Otherwise, returns `Some((proof, commitment, blinding))`, where:
    /// `proof` is the rangeproof, and `commitment = blinding*G + value*H`.
    ///
    /// Only the `RangeProof` should be sent to the verifier.  The
    /// commitment and blinding are for the use of the prover.
    pub fn create_vartime<T: Rng>(
        n: usize,
        value: u64,
        G: &DecafBasepointTable,
        H: &DecafPoint,
        mut csprng: &mut T,
    ) -> Option<(RangeProof, DecafPoint, Scalar)> {
        // Calling verify with n out of bounds is a programming error.
        if n > RANGEPROOF_MAX_N {
            panic!("Error: called create_vartime with too large bound 3^n, n = {}", n);
        }

        // Check that value is in range: all digits above n should be 0
        let v = base3_digits(value);
        for i in n..41 {
            if v[i] != 0 { return None; }
        }

        let mut R = vec![DecafPoint::identity(); n];
        let mut C = vec![DecafPoint::identity(); n];
        let mut k   = vec![Scalar::zero(); n];
        let mut r   = vec![Scalar::zero(); n];
        let mut s_1 = vec![Scalar::zero(); n];
        let mut s_2 = vec![Scalar::zero(); n];
        let mut e_1 = vec![Scalar::zero(); n];
        let mut e_2 = vec![Scalar::zero(); n];

        let mut mi_H = *H;
        for i in 0..n {
            let mi2_H = &mi_H + &mi_H;
            k[i] = Scalar::random(&mut csprng);

            if v[i] == 0 {
                R[i] = G * &k[i];
            } else if v[i] == 1 {
                // Commitment to i-th digit is r^i G + 1 * m^i H
                r[i] = Scalar::random(&mut csprng);
                C[i] = &(G * &r[i]) + &mi_H;
                // Begin at index 1 in the ring, choosing random e_1
                let P = G * &k[i];
                e_1[i] = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());
                // Choose random scalar for s_2
                s_2[i] = Scalar::random(&mut csprng);
                // Compute e_2 = Hash(s_2^i G - e_1^i (C^i - 2m^i H) )
                let Ci_minus_mi2H = &C[i] - &mi2_H;
                let P = vartime::k_fold_scalar_mult(&[s_2[i],       -&e_1[i]],
                                                    &[G.basepoint(), Ci_minus_mi2H]);
                e_2[i] = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());

                R[i] = &C[i] * &e_2[i];
            } else if v[i] == 2 {
                // Commitment to i-th digit is r^i G + 2 * m^i H
                r[i] = Scalar::random(&mut csprng);
                C[i] = &(G * &r[i]) + &mi2_H;
                // Begin at index 2 in the ring, choosing random e_2
                let P = G * &k[i];
                e_2[i] = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());

                R[i] = &C[i] * &e_2[i];
            } else {
                panic!("Invalid digit {}", v[i]);
            }

            // Set mi_H <- 3 * mi_H so that mi_H = m^i H in the loop
            mi_H = &mi2_H + &mi_H;
        }

        // Compute e_0 = Hash( R^0 || ... || R^{n-1} )
        let mut e_0_hash = Sha512::default();
        for i in 0..n {
            e_0_hash.input(R[i].compress().as_bytes());
        }
        let e_0 = Scalar::from_hash(e_0_hash);

        let mut mi_H = *H;
        for i in 0..n {
            let mi2_H = &mi_H + &mi_H;
            if v[i] == 0 {
                let k_1 = Scalar::random(&mut csprng);
                let P = vartime::k_fold_scalar_mult(&[k_1, e_0], &[G.basepoint(), mi_H]);
                e_1[i] = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());

                let k_2 = Scalar::random(&mut csprng);
                let P = vartime::k_fold_scalar_mult(&[k_2, e_1[i]], &[G.basepoint(), mi2_H]);
                e_2[i] = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());

                let e_2_inv = e_2[i].invert();
                r[i] = &e_2_inv * &k[i];
                C[i] = G * &r[i];

                s_1[i] = &k_1 + &(&e_0    * &(&k[i] * &e_2_inv));
                s_2[i] = &k_2 + &(&e_1[i] * &(&k[i] * &e_2_inv));
            } else if v[i] == 1 {
                s_1[i] = Scalar::multiply_add(&e_0, &r[i], &k[i]);
            } else if v[i] == 2 {
                s_1[i] = Scalar::random(&mut csprng);
                // Compute e_1^i = Hash(s_1^i G - e_0^i (C^i - 1 m^i H) )
                let Ci_minus_miH = &C[i] - &mi_H;
                let P = vartime::k_fold_scalar_mult(&[s_1[i],        -&e_0],
                                                    &[G.basepoint(), Ci_minus_miH]);
                e_1[i] = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());
                s_2[i] = Scalar::multiply_add(&e_1[i], &r[i], &k[i]);
            }
            // Set mi_H <-- 3*m_iH, so that mi_H is always 3^i * H in the loop
            mi_H = &mi_H + &mi2_H;
        }

        let mut blinding = Scalar::zero();
        let mut commitment = DecafPoint::identity();
        for i in 0..n {
            blinding += &r[i];
            // XXX implement AddAssign for ExtendedPoint
            commitment = &commitment + &C[i];
        }

        Some((
            RangeProof{e_0: e_0, C: C, s_1: s_1, s_2: s_2},
            commitment,
            blinding,
        ))
    }

    /// Construct a rangeproof for `value`, in constant time.
    ///
    /// This function is roughly three times slower (since `m = 3`) than the
    /// variable time version, for all values of `n`.
    ///
    /// # Inputs
    ///
    /// * `n`, so that the range is `[0,3^n]` with `n < RANGEPROOF_MAX_N`;
    /// * The `value` to prove within range `[0,3^n]`;
    /// * `csprng`, an implementation of `rand::Rng`, which should be
    /// cryptographically secure.
    ///
    /// # Returns
    ///
    /// If `value` is not in the range `[0,3^n]`, return None.
    ///
    /// Note that this function is designed to execute in constant
    /// time for all *valid* inputs.  Passing an out-of-range `value`
    /// will cause it to return `None` early.
    /// 
    /// Otherwise, returns `Some((proof, commitment, blinding))`, where:
    /// `proof` is the rangeproof, and `commitment = blinding*G + value*H`.
    ///
    /// Only the `RangeProof` should be sent to the verifier.  The
    /// commitment and blinding are for the use of the prover.
    ///
    /// # Note
    ///
    /// Even when passing a deterministic CSPRNG generated with identical seeds,
    /// e.g. two instances of `rand::chacha::ChaChaRng::new_unseeded()`, and
    /// seeking to prove the same `value`, one cannot expect the `RangeProofs`
    /// generated with `RangeProof::create_vartime()` and `RangeProof::create()`
    /// to be identical.  The values in the eventual proofs will differ, since
    /// this constant time version makes additional calls to the `csprng` which
    /// are thrown away in some conditions.
    pub fn create<T: Rng>(
        n: usize,
        value: u64,
        G: &DecafBasepointTable,
        H: &DecafPoint,
        mut csprng: &mut T,
    ) -> Option<(RangeProof, DecafPoint, Scalar)> {
        // Calling verify with n out of bounds is a programming error.
        if n > RANGEPROOF_MAX_N {
            panic!("Error: called create_vartime with too large bound 3^n, n = {}", n);
        }

        // Check that value is in range: all digits above N should be 0
        let v = base3_digits(value);
        for i in n..41 {
            if v[i] != 0 { return None; }
        }

        let mut R = vec![DecafPoint::identity(); n];
        let mut C = vec![DecafPoint::identity(); n];
        let mut k   = vec![Scalar::zero(); n];
        let mut r   = vec![Scalar::zero(); n];
        let mut s_1 = vec![Scalar::zero(); n];
        let mut s_2 = vec![Scalar::zero(); n];
        let mut e_1 = vec![Scalar::zero(); n];
        let mut e_2 = vec![Scalar::zero(); n];

        let mut mi_H = *H;
        let mut P: DecafPoint;

        for i in 0..n {
            debug_assert!(v[i] == 0 || v[i] == 1 || v[i] == 2);

            let mi2_H: DecafPoint = &mi_H + &mi_H;

            k[i] = Scalar::random(&mut csprng);

            // Commitment to i-th digit is r^i G + (v^1 * m^i H)
            let maybe_ri: Scalar = Scalar::random(&mut csprng);
            r[i].conditional_assign(&maybe_ri, byte_is_nonzero(v[i]));

            let mut which_mi_H: DecafPoint = mi_H;  // is a copy
            which_mi_H.conditional_assign(&mi2_H, bytes_equal_ct(v[i], 2u8));

            let maybe_Ci: DecafPoint = &(G * &r[i]) + &which_mi_H;
            C[i].conditional_assign(&maybe_Ci, byte_is_nonzero(v[i]));

            P = &k[i] * G;

            // Begin at index 1 in the ring, choosing random e_{v^i}
            let mut maybe_ei = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());
            e_1[i].conditional_assign(&maybe_ei, bytes_equal_ct(v[i], 1u8));
            e_2[i].conditional_assign(&maybe_ei, bytes_equal_ct(v[i], 2u8));

            // Choose random scalar for s_2
            let maybe_s2: Scalar = Scalar::random(&mut csprng);
            s_2[i].conditional_assign(&maybe_s2, bytes_equal_ct(v[i], 1u8));

            // Compute e_2 = Hash(s_2^i G - e_1^i (C^i - 2m^i H) )
            P = &(&s_2[i] * G) - &(&e_1[i] * &(&C[i] - &mi2_H));
            maybe_ei = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());
            e_2[i].conditional_assign(&maybe_ei, bytes_equal_ct(v[i], 1u8));

            // Compute R^i = k^i G            iff  v^i == 0, otherwise
            //         R^i = e_2^i * C^i
            R[i] = &k[i] * G;

            let maybe_Ri: DecafPoint = &e_2[i] * &C[i];
            R[i].conditional_assign(&maybe_Ri, byte_is_nonzero(v[i]));

            // Multiply mi_H by m (a.k.a. m == 3)
            mi_H = &mi2_H + &mi_H;
        }

        // Compute e_0 = Hash( R^0 || ... || R^{n-1} )
        let mut e_0_hash = Sha512::default();
        for i in 0..n {
            e_0_hash.input(R[i].compress().as_bytes());  // XXX new digest API for 0.5.x
        }
        let e_0 = Scalar::from_hash(e_0_hash);

        let mut mi_H = *H;

        for i in 0..n {
            debug_assert!(v[i] == 0 || v[i] == 1 || v[i] == 2);

            let mi2_H = &mi_H + &mi_H;

            let mut k_1 = Scalar::zero();
            let maybe_k1: Scalar = Scalar::random(&mut csprng);
            k_1.conditional_assign(&maybe_k1, bytes_equal_ct(v[i], 0u8));

            P = &(&k_1 * G) + &(&e_0 * &mi_H);
            let maybe_e_1 = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());
            e_1[i].conditional_assign(&maybe_e_1, bytes_equal_ct(v[i], 0u8));

            let mut k_2 = Scalar::zero();
            let maybe_k2: Scalar = Scalar::random(&mut csprng);
            k_2.conditional_assign(&maybe_k2, bytes_equal_ct(v[i], 0u8));

            P = &(&k_2 * &G.basepoint()) + &(&e_1[i] * &mi2_H);
            let maybe_e_2 = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes()); // XXX API
            e_2[i].conditional_assign(&maybe_e_2, bytes_equal_ct(v[i], 0u8));

            let e_2_inv = e_2[i].invert();  // XXX only used in v[i]==0, check what the optimiser is doing
            let maybe_r_i = &e_2_inv * &k[i];
            r[i].conditional_assign(&maybe_r_i, bytes_equal_ct(v[i], 0u8));

            let maybe_C_i = G * &r[i];
            C[i].conditional_assign(&maybe_C_i, bytes_equal_ct(v[i], 0u8));

            let mut maybe_s_1 = &k_1 + &(&e_0 * &(&k[i] * &e_2_inv));  // XXX reuse k[i] * e_2_inv
            s_1[i].conditional_assign(&maybe_s_1, bytes_equal_ct(v[i], 0u8));
            maybe_s_1 = Scalar::multiply_add(&e_0, &r[i], &k[i]);
            s_1[i].conditional_assign(&maybe_s_1, bytes_equal_ct(v[i], 1u8));
            maybe_s_1 = Scalar::random(&mut csprng);
            s_1[i].conditional_assign(&maybe_s_1, bytes_equal_ct(v[i], 2u8));

            // Compute e_1^i = Hash(s_1^i G - e_0^i (C^i - 1 m^i H) )
            let Ci_minus_miH = &C[i] - &mi_H;  // XXX only used in v[i]==2, check optimiser

            P = &(&s_1[i] * &G.basepoint()) - &(&e_0 * &Ci_minus_miH);
            let maybe_e_1 = Scalar::hash_from_bytes::<Sha512>(P.compress().as_bytes());
            e_1[i].conditional_assign(&maybe_e_1, bytes_equal_ct(v[i], 2u8));

            let mut maybe_s_2 = &k_2 + &(&e_1[i] * &(&k[i] * &e_2_inv));  // XXX reuse k[i] * e_2_inv
            s_2[i].conditional_assign(&maybe_s_2, bytes_equal_ct(v[i], 0u8));
            maybe_s_2 = Scalar::multiply_add(&e_1[i], &r[i], &k[i]);
            s_2[i].conditional_assign(&maybe_s_2, bytes_equal_ct(v[i], 2u8));

            // Set mi_H <-- 3*m_iH, so that mi_H is always 3^i * H in the loop
            mi_H = &mi_H + &mi2_H;
        }

        let mut blinding = Scalar::zero();
        let mut commitment = DecafPoint::identity();
        for i in 0..n {
            blinding += &r[i];
            // XXX implement AddAssign for ExtendedPoint
            commitment = &commitment + &C[i];
        }

        Some((
            RangeProof{ e_0: e_0, C: C, s_1: s_1, s_2: s_2},
            commitment,
            blinding,
        ))
    }
}

fn base3_digits(mut x: u64) -> [u8; 41] {
    let mut digits = [0u8; 41];
    for i in 0..41 {
        let rem = x % 3;
        digits[i] = rem as u8;
        x = x / 3;
    }
    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::OsRng;
    use sha2::Sha256;

    use curve25519_dalek::constants as dalek_constants;

    #[test]
    fn base3_digits_vs_sage() {
        let values: [u64; 10] = [10352669767914021650,  7804842618637096123,
                                  7334633556203117754,  8160423201521470302,
                                 17232767106382697250,  8845500362072010910,
                                  9696550650556789001,   769845413554321661,
                                  3398590720602317514, 14390516357262902374];
        let digits_sage: [[u8;41]; 10] = [
            [2, 2, 0, 2, 1, 2, 2, 2, 1, 1, 2, 2, 1, 1, 1, 2, 1, 2, 0, 1, 0, 2, 2, 1, 0, 1, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 1, 1, 2, 0],
            [1, 1, 2, 2, 1, 0, 1, 2, 1, 0, 0, 1, 0, 2, 2, 1, 1, 1, 2, 0, 1, 0, 0, 1, 1, 0, 2, 1, 2, 1, 2, 2, 2, 2, 2, 2, 0, 2, 2, 1, 0],
            [0, 1, 1, 0, 2, 2, 1, 2, 0, 0, 0, 2, 0, 2, 1, 1, 0, 2, 1, 0, 0, 0, 2, 0, 0, 1, 2, 1, 1, 2, 1, 0, 1, 2, 1, 2, 0, 1, 2, 1, 0],
            [0, 2, 0, 1, 2, 2, 0, 0, 2, 2, 2, 2, 0, 2, 2, 0, 1, 1, 1, 1, 0, 1, 0, 2, 0, 2, 1, 2, 2, 1, 1, 2, 2, 0, 0, 1, 0, 0, 0, 2, 0],
            [0, 1, 2, 1, 2, 2, 0, 2, 0, 0, 2, 2, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 2, 1, 2, 2, 2, 0, 1, 1, 2, 2, 0, 1, 2, 0, 2, 0, 1, 1],
            [1, 2, 0, 2, 2, 0, 2, 1, 0, 1, 2, 1, 2, 0, 0, 0, 0, 2, 2, 0, 0, 2, 2, 1, 1, 0, 2, 0, 0, 0, 2, 1, 0, 1, 2, 2, 1, 1, 0, 2, 0],
            [2, 0, 1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 1, 2, 0, 0, 1, 2, 2, 0, 0, 2, 2, 1, 0, 1, 0, 2, 1, 1, 1, 2, 0, 1, 2, 1, 1, 0, 1, 2, 0],
            [2, 2, 2, 1, 2, 2, 1, 2, 0, 2, 1, 0, 2, 1, 1, 2, 2, 2, 2, 0, 2, 1, 0, 1, 2, 0, 1, 2, 0, 0, 1, 1, 1, 0, 1, 0, 2, 1, 0, 0, 0],
            [0, 1, 1, 2, 0, 1, 1, 1, 2, 1, 0, 0, 2, 2, 2, 0, 0, 1, 1, 1, 2, 2, 0, 0, 0, 2, 2, 1, 0, 2, 0, 0, 1, 2, 2, 1, 1, 1, 2, 0, 0],
            [1, 2, 0, 0, 1, 1, 0, 1, 2, 1, 0, 1, 0, 1, 2, 2, 0, 0, 2, 1, 0, 1, 2, 2, 1, 2, 2, 0, 1, 2, 2, 2, 1, 2, 1, 2, 2, 1, 1, 0, 1],
        ];

        for i in 0..10 {
            let digits = base3_digits(values[i]);
            for j in 0..41 {
                assert_eq!(digits[j], digits_sage[i][j]);
            }
        }
    }

    #[test]
    fn prove_and_verify_vartime() {
        let G = &dalek_constants::DECAF_ED25519_BASEPOINT_TABLE;
        let H = DecafPoint::hash_from_bytes::<Sha256>(G.basepoint().compress().as_bytes());

        let mut csprng = OsRng::new().unwrap();

        let n = 16;
        let value = 13449261;
        let (proof, commitment, blinding) =
            RangeProof::create_vartime(n, value, G, &H, &mut csprng).unwrap();

        let C_option = proof.verify(n, G, &H);
        assert!(C_option.is_some());

        assert!(proof.verify(2, G, &H).is_none());

        let C = C_option.unwrap();
        let C_hat = &(G * &blinding) + &(&H * &Scalar::from_u64(value));

        assert_eq!(C.compress(), C_hat.compress());
        assert_eq!(commitment.compress(), C_hat.compress());
    }

    #[test]
    fn prove_and_verify_ct() {
        let G = &dalek_constants::DECAF_ED25519_BASEPOINT_TABLE;
        let H = DecafPoint::hash_from_bytes::<Sha256>(G.basepoint().compress().as_bytes());

        let mut csprng = OsRng::new().unwrap();

        let n = 16;
        let value = 13449261;
        let (proof, commitment, blinding) =
            RangeProof::create(n, value, G, &H, &mut csprng).unwrap();

        let C_option = proof.verify(n, G, &H);
        assert!(C_option.is_some());

        assert!(proof.verify(2, G, &H).is_none());

        let C = C_option.unwrap();
        let C_hat = &(G * &blinding) + &(&H * &Scalar::from_u64(value));

        assert_eq!(C.compress(), C_hat.compress());
        assert_eq!(commitment.compress(), C_hat.compress());
    }
}

#[cfg(all(test, feature = "bench"))]
mod bench {
    use test::Bencher;
    use super::*;

    use rand::OsRng;
    use sha2::Sha256;
    use curve25519_dalek::constants as dalek_constants;

    #[bench]
    fn verify(b: &mut Bencher) {
        let G = &dalek_constants::DECAF_ED25519_BASEPOINT_TABLE;
        let H = DecafPoint::hash_from_bytes::<Sha256>(G.basepoint().compress().as_bytes());

        let mut csprng = OsRng::new().unwrap();

        let value = 1666;
        let (proof, _, _) = RangeProof::create_vartime(
            RANGEPROOF_MAX_N, value, G, &H, &mut csprng).unwrap();

        b.iter(|| proof.verify(RANGEPROOF_MAX_N, G, &H));
    }

    #[bench]
    fn prove_vartime(b: &mut Bencher) {
        let G = &dalek_constants::DECAF_ED25519_BASEPOINT_TABLE;
        let H = DecafPoint::hash_from_bytes::<Sha256>(G.basepoint().compress().as_bytes());

        let mut csprng = OsRng::new().unwrap();

        let value = 1666;
        b.iter(|| RangeProof::create_vartime(RANGEPROOF_MAX_N, value, G, &H, &mut csprng));
    }

    #[bench]
    fn prove_ct(b: &mut Bencher) {
        let G = &dalek_constants::DECAF_ED25519_BASEPOINT_TABLE;
        let H = DecafPoint::hash_from_bytes::<Sha256>(G.basepoint().compress().as_bytes());

        let mut csprng = OsRng::new().unwrap();

        let value = 1666;
        b.iter(|| RangeProof::create(RANGEPROOF_MAX_N, value, G, &H, &mut csprng));
    }

    #[bench]
    fn bench_base3_digits(b: &mut Bencher) {
        b.iter(|| base3_digits(10352669767914021650) );
    }
}
