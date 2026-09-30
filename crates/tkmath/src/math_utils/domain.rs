/// Scalar parameter interval; `Domain1D::new(0.0,1.0)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Domain1D {
    /// Lower endpoint; `domain.min`.
    pub min: f64,
    /// Upper endpoint; `domain.max`.
    pub max: f64,
}
impl Domain1D {
    /// Retains endpoints without normalization; `Domain1D::new(0.0,1.0)`.
    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }
    /// Returns max-min; `domain.length()`.
    pub const fn length(&self) -> f64 {
        self.max - self.min
    }
    /// Requires finite ordered bounds; `domain.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.min.is_finite() && self.max.is_finite() && self.min <= self.max
    }
    /// Retains the native sum before halving; `domain.mid()`.
    pub const fn mid(&self) -> f64 {
        (self.min + self.max) * 0.5
    }
    /// Checks parameter and tolerance finiteness; `domain.contains(u,0.0)`.
    pub fn contains(&self, u: f64, tolerance: f64) -> bool {
        self.is_valid()
            && u.is_finite()
            && tolerance.is_finite()
            && tolerance >= 0.0
            && u >= self.min - tolerance
            && u <= self.max + tolerance
    }
    /// Applies native endpoint comparisons; `domain.clamp(u)`.
    pub fn clamp(&self, u: f64) -> f64 {
        super::clamp(u, self.min, self.max)
    }
    /// Checks native domain length threshold; `domain.is_large(1000.0)`.
    pub fn is_large(&self, threshold: f64) -> bool {
        self.length() > threshold
    }
    /// Checks coverage of a period with tolerance; `domain.is_full_period(period,1e-10)`.
    pub fn is_full_period(&self, period: f64, tolerance: f64) -> bool {
        self.length() >= period - tolerance
    }
    /// Interpolates without clamping; `domain.lerp(0.5)`.
    pub const fn lerp(&self, t: f64) -> f64 {
        self.min + t * (self.max - self.min)
    }
    /// Returns 0.5 for nonpositive length; `domain.normalize(u)`.
    pub fn normalize(&self, u: f64) -> f64 {
        let length: f64 = self.length();
        if length > 0.0 {
            (u - self.min) / length
        } else {
            0.5
        }
    }
    /// Checks strict effective-infinity limits; `domain.is_finite(1e100)`.
    pub fn is_finite(&self, limit: f64) -> bool {
        self.is_valid() && limit.is_finite() && limit > 0.0 && self.min > -limit && self.max < limit
    }
    /// Compares endpoints without extra validation; `domain.is_equal(&other,1e-10)`.
    pub fn is_equal(&self, other: &Self, tolerance: f64) -> bool {
        (self.min - other.min).abs() <= tolerance && (self.max - other.max).abs() <= tolerance
    }
}
/// Rectangular parameter interval; `Domain2D::new(0.0,1.0,0.0,1.0)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Domain2D {
    /// Lower U endpoint; `domain.u_min`.
    pub u_min: f64,
    /// Upper U endpoint; `domain.u_max`.
    pub u_max: f64,
    /// Lower V endpoint; `domain.v_min`.
    pub v_min: f64,
    /// Upper V endpoint; `domain.v_max`.
    pub v_max: f64,
}
impl Domain2D {
    /// Retains all endpoints; `Domain2D::new(0.0,1.0,0.0,1.0)`.
    pub const fn new(u_min: f64, u_max: f64, v_min: f64, v_max: f64) -> Self {
        Self {
            u_min,
            u_max,
            v_min,
            v_max,
        }
    }
    /// Combines scalar domains; `Domain2D::from_domains(u,v)`.
    pub const fn from_domains(u: Domain1D, v: Domain1D) -> Self {
        Self::new(u.min, u.max, v.min, v.max)
    }
    /// Copies the U subdomain; `domain.u()`.
    pub const fn u(&self) -> Domain1D {
        Domain1D::new(self.u_min, self.u_max)
    }
    /// Copies the V subdomain; `domain.v()`.
    pub const fn v(&self) -> Domain1D {
        Domain1D::new(self.v_min, self.v_max)
    }
    /// Returns U extent; `domain.u_length()`.
    pub const fn u_length(&self) -> f64 {
        self.u_max - self.u_min
    }
    /// Returns V extent; `domain.v_length()`.
    pub const fn v_length(&self) -> f64 {
        self.v_max - self.v_min
    }
    /// Validates both subdomains; `domain.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.u().is_valid() && self.v().is_valid()
    }
    /// Returns native U midpoint; `domain.u_mid()`.
    pub const fn u_mid(&self) -> f64 {
        (self.u_min + self.u_max) * 0.5
    }
    /// Returns native V midpoint; `domain.v_mid()`.
    pub const fn v_mid(&self) -> f64 {
        (self.v_min + self.v_max) * 0.5
    }
    /// Tests a UV pair with finite tolerance; `domain.contains(u,v,0.0)`.
    pub fn contains(&self, u: f64, v: f64, tolerance: f64) -> bool {
        self.is_valid() && self.u().contains(u, tolerance) && self.v().contains(v, tolerance)
    }
    /// Modifies both coordinates with native comparisons; `domain.clamp(&mut u,&mut v)`.
    pub fn clamp(&self, u: &mut f64, v: &mut f64) {
        *u = self.u().clamp(*u);
        *v = self.v().clamp(*v);
    }
    /// Requires both lengths above the threshold; `domain.is_large(1000.0)`.
    pub fn is_large(&self, threshold: f64) -> bool {
        self.u_length() > threshold && self.v_length() > threshold
    }
    /// Checks U periodic coverage; `domain.is_u_full_period(period,1e-10)`.
    pub fn is_u_full_period(&self, period: f64, tolerance: f64) -> bool {
        self.u_length() >= period - tolerance
    }
    /// Checks V periodic coverage; `domain.is_v_full_period(period,1e-10)`.
    pub fn is_v_full_period(&self, period: f64, tolerance: f64) -> bool {
        self.v_length() >= period - tolerance
    }
    /// Requires finite subdomains within the effective limit; `domain.is_finite(1e100)`.
    pub fn is_finite(&self, limit: f64) -> bool {
        self.is_valid() && self.u().is_finite(limit) && self.v().is_finite(limit)
    }
}
