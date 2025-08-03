//! Bezier curve easing editor for Flash-style animation control

use egui::Vec2;
use serde::{Deserialize, Serialize};

/// A bezier curve used for easing animation properties
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BezierCurve {
    /// Control points defining the curve
    pub points: Vec<BezierPoint>,
}

/// A control point on a bezier curve with handles
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BezierPoint {
    /// Position in normalized space (0-1, 0-1)
    pub position: (f32, f32),
    /// Incoming tangent handle (relative to position)
    pub in_handle: (f32, f32),
    /// Outgoing tangent handle (relative to position)  
    pub out_handle: (f32, f32),
}

impl BezierPoint {
    /// Get position as Vec2
    pub fn position_vec2(&self) -> Vec2 {
        Vec2::new(self.position.0, self.position.1)
    }
    
    /// Get in handle as Vec2
    pub fn in_handle_vec2(&self) -> Vec2 {
        Vec2::new(self.in_handle.0, self.in_handle.1)
    }
    
    /// Get out handle as Vec2  
    pub fn out_handle_vec2(&self) -> Vec2 {
        Vec2::new(self.out_handle.0, self.out_handle.1)
    }
}

/// Common easing presets available in the Motion Editor
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EasingPreset {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    Custom(BezierCurve),
}

/// Property that can be animated with easing
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PropertyId {
    PositionX,
    PositionY,
    Rotation,
    ScaleX,
    ScaleY,
    Alpha,
    ColorR,
    ColorG,
    ColorB,
    Custom(String),
}

impl BezierCurve {
    /// Create a linear curve (0,0) to (1,1)
    pub fn linear() -> Self {
        Self {
            points: vec![
                BezierPoint {
                    position: (0.0, 0.0),
                    in_handle: (0.0, 0.0),
                    out_handle: (0.33, 0.0),
                },
                BezierPoint {
                    position: (1.0, 1.0),
                    in_handle: (-0.33, 0.0),
                    out_handle: (0.0, 0.0),
                },
            ],
        }
    }
    
    /// Create ease-in curve
    pub fn ease_in() -> Self {
        Self {
            points: vec![
                BezierPoint {
                    position: (0.0, 0.0),
                    in_handle: (0.0, 0.0),
                    out_handle: (0.2, 0.0),
                },
                BezierPoint {
                    position: (1.0, 1.0),
                    in_handle: (-0.2, 0.0),
                    out_handle: (0.0, 0.0),
                },
            ],
        }
    }
    
    /// Create ease-out curve
    pub fn ease_out() -> Self {
        Self {
            points: vec![
                BezierPoint {
                    position: (0.0, 0.0),
                    in_handle: (0.0, 0.0),
                    out_handle: (0.0, 0.4),
                },
                BezierPoint {
                    position: (1.0, 1.0),
                    in_handle: (0.0, -0.4),
                    out_handle: (0.0, 0.0),
                },
            ],
        }
    }
    
    /// Create ease-in-out curve
    pub fn ease_in_out() -> Self {
        Self {
            points: vec![
                BezierPoint {
                    position: (0.0, 0.0),
                    in_handle: (0.0, 0.0),
                    out_handle: (0.2, 0.0),
                },
                BezierPoint {
                    position: (1.0, 1.0),
                    in_handle: (-0.2, 0.0),
                    out_handle: (0.0, 0.0),
                },
            ],
        }
    }
    
    /// Evaluate the curve at time t (0.0 to 1.0)
    pub fn evaluate(&self, t: f32) -> f32 {
        if self.points.len() < 2 {
            return t; // Fallback to linear
        }
        
        let t = t.clamp(0.0, 1.0);
        
        // For simple 2-point curves, use cubic bezier evaluation
        if self.points.len() == 2 {
            let p0 = self.points[0].position_vec2();
            let p1 = self.points[0].position_vec2() + self.points[0].out_handle_vec2();
            let p2 = self.points[1].position_vec2() + self.points[1].in_handle_vec2();
            let p3 = self.points[1].position_vec2();
            
            // Cubic bezier: B(t) = (1-t)³P₀ + 3(1-t)²tP₁ + 3(1-t)t²P₂ + t³P₃
            let u = 1.0 - t;
            let u2 = u * u;
            let u3 = u2 * u;
            let t2 = t * t;
            let t3 = t2 * t;
            
            let y = u3 * p0.y + 3.0 * u2 * t * p1.y + 3.0 * u * t2 * p2.y + t3 * p3.y;
            y.clamp(0.0, 1.0)
        } else {
            // For multi-point curves, use linear interpolation between segments
            // TODO: Implement full bezier spline evaluation
            t
        }
    }
}

impl EasingPreset {
    /// Get all available presets
    pub fn all_presets() -> Vec<EasingPreset> {
        vec![
            EasingPreset::Linear,
            EasingPreset::EaseIn,
            EasingPreset::EaseOut,
            EasingPreset::EaseInOut,
            EasingPreset::EaseInQuad,
            EasingPreset::EaseOutQuad,
            EasingPreset::EaseInOutQuad,
            EasingPreset::EaseInCubic,
            EasingPreset::EaseOutCubic,
            EasingPreset::EaseInOutCubic,
            EasingPreset::EaseInElastic,
            EasingPreset::EaseOutElastic,
            EasingPreset::EaseInOutElastic,
            EasingPreset::EaseInBounce,
            EasingPreset::EaseOutBounce,
            EasingPreset::EaseInOutBounce,
        ]
    }
    
    /// Get the display name for the preset
    pub fn name(&self) -> &str {
        match self {
            EasingPreset::Linear => "Linear",
            EasingPreset::EaseIn => "Ease In",
            EasingPreset::EaseOut => "Ease Out",
            EasingPreset::EaseInOut => "Ease In-Out",
            EasingPreset::EaseInQuad => "Ease In Quad",
            EasingPreset::EaseOutQuad => "Ease Out Quad",
            EasingPreset::EaseInOutQuad => "Ease In-Out Quad",
            EasingPreset::EaseInCubic => "Ease In Cubic",
            EasingPreset::EaseOutCubic => "Ease Out Cubic",
            EasingPreset::EaseInOutCubic => "Ease In-Out Cubic",
            EasingPreset::EaseInElastic => "Ease In Elastic",
            EasingPreset::EaseOutElastic => "Ease Out Elastic",
            EasingPreset::EaseInOutElastic => "Ease In-Out Elastic",
            EasingPreset::EaseInBounce => "Ease In Bounce",
            EasingPreset::EaseOutBounce => "Ease Out Bounce",
            EasingPreset::EaseInOutBounce => "Ease In-Out Bounce",
            EasingPreset::Custom(_) => "Custom",
        }
    }
    
    /// Convert preset to bezier curve
    pub fn to_curve(&self) -> BezierCurve {
        match self {
            EasingPreset::Linear => BezierCurve::linear(),
            EasingPreset::EaseIn => BezierCurve::ease_in(),
            EasingPreset::EaseOut => BezierCurve::ease_out(),
            EasingPreset::EaseInOut => BezierCurve::ease_in_out(),
            EasingPreset::Custom(curve) => curve.clone(),
            // TODO: Implement other preset curves
            _ => BezierCurve::linear(),
        }
    }
}

impl PropertyId {
    /// Get display name for the property
    pub fn name(&self) -> &str {
        match self {
            PropertyId::PositionX => "Position X",
            PropertyId::PositionY => "Position Y",
            PropertyId::Rotation => "Rotation",
            PropertyId::ScaleX => "Scale X",
            PropertyId::ScaleY => "Scale Y",
            PropertyId::Alpha => "Alpha",
            PropertyId::ColorR => "Color Red",
            PropertyId::ColorG => "Color Green",
            PropertyId::ColorB => "Color Blue",
            PropertyId::Custom(name) => name,
        }
    }
    
    /// Get all animatable properties
    pub fn all_properties() -> Vec<PropertyId> {
        vec![
            PropertyId::PositionX,
            PropertyId::PositionY,
            PropertyId::Rotation,
            PropertyId::ScaleX,
            PropertyId::ScaleY,
            PropertyId::Alpha,
            PropertyId::ColorR,
            PropertyId::ColorG,
            PropertyId::ColorB,
        ]
    }
}

impl Default for BezierCurve {
    fn default() -> Self {
        Self::linear()
    }
}

impl Default for EasingPreset {
    fn default() -> Self {
        EasingPreset::Linear
    }
}