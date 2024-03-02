//! # `bevy_tween`

// #![warn(missing_docs)]

use bevy::{app::PluginGroupBuilder, prelude::*};

mod utils;

pub mod interpolation;
pub mod lenses;
pub mod tween;
pub mod tween_player;

#[cfg(feature = "span_tween")]
pub mod span_tween;

/// Commonly used items
pub mod prelude {
    pub use crate::interpolation::EaseFunction;
    pub use crate::lenses::{self, TweenLens};
    pub use crate::tween_player::{Repeat, RepeatStyle};
    pub use crate::DefaultTweenPlugins;

    #[cfg(all(feature = "bevy_asset", feature = "tween_unboxed"))]
    pub use crate::tween::AssetTween;
    #[cfg(feature = "tween_unboxed")]
    pub use crate::tween::ComponentTween;
    #[cfg(feature = "tween_unboxed")]
    pub use crate::tween::ResourceTween;

    #[cfg(all(feature = "tween_boxed", feature = "bevy_asset"))]
    pub use crate::tween::AssetTweenBoxed;
    #[cfg(feature = "tween_boxed")]
    pub use crate::tween::ComponentTweenBoxed;
    #[cfg(feature = "tween_boxed")]
    pub use crate::tween::ResourceTweenBoxed;

    #[cfg(feature = "span_tween")]
    pub use crate::span_tween::{SpanTweenBundle, SpanTweenPlayerBundle};
}

/// Default plugins in this crate
pub struct DefaultTweenPlugins;
impl PluginGroup for DefaultTweenPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let p = PluginGroupBuilder::start::<DefaultTweenPlugins>()
            .add(TweenCorePlugin)
            .add(lenses::DefaultTweenLensesPlugin)
            .add(interpolation::EaseFunctionPlugin);
        #[cfg(feature = "span_tween")]
        let p = p.add(span_tween::SpanTweenPlugin);
        p
    }
}

/// Core and basic types you need to get started with this plugin
pub struct TweenCorePlugin;
impl Plugin for TweenCorePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                TweenSystemSet::TickTweenPlayer,
                TweenSystemSet::TweenPlayer,
                TweenSystemSet::SampleInterpolator,
                TweenSystemSet::ApplyTween,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (tween_player::tick_tween_player_state_system,)
                .in_set(TweenSystemSet::TickTweenPlayer),
        )
        // .add_event::<tween_player::TweenPlayerEnded>()
        .register_type::<tween_player::TweenPlayerState>()
        .register_type::<tween_player::AnimationDirection>()
        .register_type::<tween_player::Repeat>()
        .register_type::<tween_player::RepeatStyle>()
        .register_type::<tween::TweenState>()
        .register_type::<tween::TweenInterpolationValue>();
    }
}

#[derive(Debug, SystemSet, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TweenSystemSet {
    TickTweenPlayer,
    TweenPlayer,
    SampleInterpolator,
    ApplyTween,
}
