// Modified version of https://github.com/elliotekj/DeltaE

use lab::Lab;
use std::f32;
use std::f32::consts::PI;

pub struct DE2000;

pub struct KSubArgs {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

#[allow(clippy::needless_doctest_main)]
impl DE2000 {
    /// Returns the difference between two `Lab` colors.
    ///
    /// ### Example
    ///
    /// ```ignore
    /// extern crate delta_e;
    /// extern crate lab;
    ///
    /// use delta_e::DE2000;
    /// use lab::Lab;
    ///
    /// fn main() {
    ///     let color_1 = Lab {
    ///         l: 38.972,
    ///         a: 58.991,
    ///         b: 37.138,
    ///     };
    ///
    ///     let color_2 = Lab {
    ///         l: 54.528,
    ///         a: 42.416,
    ///         b: 54.497,
    ///     };
    ///
    ///     let delta_e = DE2000::new(color_1, color_2);
    ///     println!("The color difference is: {}", delta_e);
    /// }
    /// ```

    #[allow(clippy::new_ret_no_self)]
    pub fn new(color_1: Lab, color_2: Lab, ksub: KSubArgs) -> f32 {
        let delta_l_prime = color_2.l - color_1.l;

        let l_bar = (color_1.l + color_2.l) / 2.0;

        let c1 = (color_1.a.powi(2) + color_1.b.powi(2)).sqrt();
        let c2 = (color_2.a.powi(2) + color_2.b.powi(2)).sqrt();

        let (a_prime_1, a_prime_2) = {
            let c_bar = (c1 + c2) / 2.0;

            let tmp = 1.0 - (c_bar.powi(7) / (c_bar.powi(7) + 25f32.powi(7))).sqrt();
            (
                color_1.a + (color_1.a / 2.0) * tmp,
                color_2.a + (color_2.a / 2.0) * tmp,
            )
        };

        let c_prime_1 = (a_prime_1.powi(2) + color_1.b.powi(2)).sqrt();
        let c_prime_2 = (a_prime_2.powi(2) + color_2.b.powi(2)).sqrt();

        let c_bar_prime = (c_prime_1 + c_prime_2) / 2.0;

        let delta_c_prime = c_prime_2 - c_prime_1;

        let s_sub_l =
            1.0 + ((0.015 * (l_bar - 50.0).powi(2)) / (20.0 + (l_bar - 50.0).powi(2)).sqrt());

        let s_sub_c = 1.0 + 0.045 * c_bar_prime;

        let h_prime_1 = get_h_prime_fn(color_1.b, a_prime_1);
        let h_prime_2 = get_h_prime_fn(color_2.b, a_prime_2);

        let delta_h_prime = get_delta_h_prime(c1, c2, h_prime_1, h_prime_2);

        let delta_upcase_h_prime =
            2.0 * (c_prime_1 * c_prime_2).sqrt() * ((delta_h_prime) / 2.0).sin();

        let upcase_h_bar_prime = get_upcase_h_bar_prime(h_prime_1, h_prime_2);

        let upcase_t = get_upcase_t(upcase_h_bar_prime);

        let s_sub_upcase_h = 1.0 + 0.015 * c_bar_prime * upcase_t;

        let r_sub_t = get_r_sub_t(c_bar_prime, upcase_h_bar_prime);

        let lightness: f32 = delta_l_prime / (ksub.l * s_sub_l);

        let chroma: f32 = delta_c_prime / (ksub.c * s_sub_c);

        let hue: f32 = delta_upcase_h_prime / (ksub.h * s_sub_upcase_h);

        (lightness.powi(2) + chroma.powi(2) + hue.powi(2) + r_sub_t * chroma * hue).sqrt()
    }
}

fn get_h_prime_fn(x: f32, y: f32) -> f32 {
    let mut hue_angle;

    if x == 0.0 && y == 0.0 {
        return 0.0;
    }

    hue_angle = x.atan2(y);

    if hue_angle < 0.0 {
        hue_angle += 2. * PI;
    }

    hue_angle
}

fn get_delta_h_prime(c1: f32, c2: f32, h_prime_1: f32, h_prime_2: f32) -> f32 {
    if 0.0 == c1 || 0.0 == c2 {
        return 0.0;
    }

    if (h_prime_1 - h_prime_2).abs() <= PI {
        return h_prime_2 - h_prime_1;
    }

    if h_prime_2 <= h_prime_1 {
        h_prime_2 - h_prime_1 + 2. * PI
    } else {
        h_prime_2 - h_prime_1 - 2. * PI
    }
}

fn get_upcase_h_bar_prime(h_prime_1: f32, h_prime_2: f32) -> f32 {
    if (h_prime_1 - h_prime_2).abs() > PI {
        return (h_prime_1 + h_prime_2 + 2.0 * PI) / 2.0;
    }

    (h_prime_1 + h_prime_2) / 2.0
}

fn get_upcase_t(upcase_h_bar_prime: f32) -> f32 {
    1.0 - 0.17 * (upcase_h_bar_prime - PI / 6.0).cos()
        + 0.24 * (2.0 * upcase_h_bar_prime).cos()
        + 0.32 * (3.0 * upcase_h_bar_prime + PI / 30.0).cos()
        - 0.20 * (4.0 * upcase_h_bar_prime - 7.0 * PI / 20.0).cos()
}

fn get_r_sub_t(c_bar_prime: f32, upcase_h_bar_prime: f32) -> f32 {
    let degrees = (radians_to_degrees(upcase_h_bar_prime) - 275.0) * (1.0 / 25.0);
    -2.0 * (c_bar_prime.powi(7) / (c_bar_prime.powi(7) + 25f32.powi(7))).sqrt()
        * (degrees_to_radians(60.0 * (-(degrees.powi(2))).exp())).sin()
}

fn radians_to_degrees(radians: f32) -> f32 {
    radians * (180.0 / f32::consts::PI)
}

fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * (f32::consts::PI / 180.0)
}
