#[rustfmt::skip]
macro_rules! impl_macro_n_times {
    ($macro:ident) => {
        $macro!(
            (),
            (P1),
            (P1, P2),
            (P1, P2, P3),
            (P1, P2, P3, P4),
            (P1, P2, P3, P4, P5),
            (P1, P2, P3, P4, P5, P6),
            (P1, P2, P3, P4, P5, P6, P7),
            (P1, P2, P3, P4, P5, P6, P7, P8),
            (P1, P2, P3, P4, P5, P6, P7, P8, P9),
            (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10),
            (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11),
            (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12),
            (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13),
            (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14),
            (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15),
            (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15, P16)
        );
    };
}

use super::into::macros::*;
use super::params::macros::*;

impl_macro_n_times!(impl_system_param_tuple);
impl_macro_n_times!(impl_into_system);
impl_macro_n_times!(impl_into_system_condition);
